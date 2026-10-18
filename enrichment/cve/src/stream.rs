//! CVE Streaming Correlation
//!
//! Real-time CVE correlation pipeline: NVD feed → CPE match → Asset graph edge update.
//! Target: New CVE → asset edge in <10s.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::{CveError, NvdCache};

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("CVE processing error: {0}")]
    Processing(String),
    #[error("Asset graph error: {0}")]
    AssetGraph(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("CVE error: {0}")]
    Cve(#[from] CveError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvdCveEvent {
    pub cve_id: String,
    pub published_date: Option<DateTime<Utc>>,
    pub last_modified_date: Option<DateTime<Utc>>,
    pub cvss_score: f64,
    pub severity: crate::CvssSeverity,
    pub cvss_vector: Option<String>,
    pub cpe_matches: Vec<String>,
    pub description: Option<String>,
    pub exploit_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveStreamConfig {
    pub kafka_brokers: String,
    pub nvd_topic: String,
    pub asset_update_topic: String,
    pub consumer_group: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub max_retries: u32,
}

impl Default for CveStreamConfig {
    fn default() -> Self {
        Self {
            kafka_brokers: "aegis-kafka:9092".to_string(),
            nvd_topic: "nvd-cve-feed".to_string(),
            asset_update_topic: "asset-cve-updates".to_string(),
            consumer_group: "aegis-cve-stream".to_string(),
            batch_size: 100,
            flush_interval_ms: 1000,
            max_retries: 3,
        }
    }
}

/// Trait for asset graph proxy to avoid circular dependencies
#[async_trait]
pub trait AssetGraphProxy: Send + Sync {
    async fn get_asset_by_cpe(&self, cpe: &crate::CpeString) -> Option<String>;
    async fn add_cve_edge(
        &self,
        source: String,
        target: String,
        cve_id: String,
        cvss: f64,
    ) -> Result<(), String>;
}

pub struct CveStreamProcessor {
    config: CveStreamConfig,
    cache: Arc<RwLock<NvdCache>>,
    asset_graph: Arc<RwLock<dyn AssetGraphProxy + Send + Sync>>,
    producer: FutureProducer,
}

impl CveStreamProcessor {
    pub async fn new(
        config: CveStreamConfig,
        cache: Arc<RwLock<NvdCache>>,
        asset_graph: Arc<RwLock<dyn AssetGraphProxy + Send + Sync>>,
    ) -> Result<Self, StreamError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka_brokers)
            .set("message.timeout.ms", "30000")
            .set("enable.idempotence", "true")
            .set("acks", "all")
            .create()?;

        Ok(Self {
            config,
            cache,
            asset_graph,
            producer,
        })
    }

    pub async fn run(&self) -> Result<(), StreamError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.config.kafka_brokers)
            .set("group.id", &self.config.consumer_group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "latest")
            .create()?;

        consumer.subscribe(&[&self.config.nvd_topic])?;

        info!(
            "CVE stream processor started, consuming from {}",
            self.config.nvd_topic
        );

        let mut message_stream = consumer.stream();

        while let Some(message) = message_stream.next().await {
            match message {
                Err(e) => warn!("Kafka error: {}", e),
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        if let Err(e) = self.process_nvd_message(payload).await {
                            warn!("Failed to process NVD message: {}", e);
                        }
                    }
                    if let Err(e) =
                        consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async)
                    {
                        warn!("Failed to commit offset: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_nvd_message(&self, payload: &[u8]) -> Result<(), StreamError> {
        let event: NvdCveEvent = serde_json::from_slice(payload)?;
        debug!("Processing CVE: {}", event.cve_id);

        // Update NVD cache
        {
            let cache = self.cache.write().await;
            let cve_record = self.convert_to_cve_record(&event);
            cache.insert(cve_record)?;
        }

        // Find affected assets via CPE correlation
        let affected_assets = self.find_affected_assets(&event).await?;

        // Generate asset graph edges
        if !affected_assets.is_empty() {
            self.emit_asset_edges(&event, &affected_assets).await?;
        }

        Ok(())
    }

    fn convert_to_cve_record(&self, event: &NvdCveEvent) -> crate::CveRecord {
        crate::CveRecord {
            cve_id: event.cve_id.clone(),
            cvss_score: event.cvss_score,
            severity: event.severity,
            cvss_vector: event.cvss_vector.clone(),
            description: event.description.clone(),
            affected_cpes: event
                .cpe_matches
                .iter()
                .map(|s| crate::CpeString::parse(s).unwrap_or_else(|_| crate::CpeString::empty()))
                .collect(),
            exploit_available: event.exploit_available,
            published: event.published_date,
            last_modified: event.last_modified_date,
        }
    }

    async fn find_affected_assets(&self, event: &NvdCveEvent) -> Result<Vec<String>, StreamError> {
        let mut assets = Vec::new();

        for cpe_str in &event.cpe_matches {
            let graph = self.asset_graph.read().await;
            if let Some(asset_id) = graph
                .get_asset_by_cpe(
                    &crate::CpeString::parse(cpe_str).unwrap_or_else(|_| crate::CpeString::empty()),
                )
                .await
            {
                assets.push(asset_id);
            }
        }

        assets.dedup();
        Ok(assets)
    }

    async fn emit_asset_edges(
        &self,
        event: &NvdCveEvent,
        assets: &[String],
    ) -> Result<(), StreamError> {
        let edge_update = AssetCveEdgeUpdate {
            cve_id: event.cve_id.clone(),
            cvss_score: event.cvss_score,
            severity: event.severity,
            affected_assets: assets.to_vec(),
            timestamp: Utc::now(),
            description: event.description.clone().unwrap_or_default(),
        };

        let key = event.cve_id.as_bytes();
        let value = serde_json::to_vec(&edge_update)?;

        let record = FutureRecord::to(&self.config.asset_update_topic)
            .key(key)
            .payload(&value);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| StreamError::Kafka(e))?;

        info!(
            "Emitted CVE edge update for {} affecting {} assets",
            event.cve_id,
            assets.len()
        );

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetCveEdgeUpdate {
    pub cve_id: String,
    pub cvss_score: f64,
    pub severity: crate::CvssSeverity,
    pub affected_assets: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

/// NVD Feed Fetcher - polls NVD API for new CVEs and publishes to Kafka
pub struct NvdFeedFetcher {
    config: CveStreamConfig,
    producer: FutureProducer,
    http_client: reqwest::Client,
    last_modified: Arc<RwLock<DateTime<Utc>>>,
}

impl NvdFeedFetcher {
    pub async fn new(config: CveStreamConfig) -> Result<Self, StreamError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka_brokers)
            .set("enable.idempotence", "true")
            .set("acks", "all")
            .create()?;

        Ok(Self {
            config,
            producer,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("AEGIS-NVD-Fetcher/1.0")
                .build()?,
            last_modified: Arc::new(RwLock::new(Utc::now() - chrono::Duration::days(1))),
        })
    }

    pub async fn run(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), StreamError> {
        let mut interval = interval(Duration::from_secs(300));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.fetch_and_publish().await {
                        warn!("NVD fetch failed: {}", e);
                    }
                }
                _ = shutdown.changed() => {
                    info!("Shutdown signal received, stopping NVD fetcher");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn fetch_and_publish(&self) -> Result<(), StreamError> {
        let last_modified = *self.last_modified.read().await;
        let now = Utc::now();
        let url = format!(
            "https://services.nvd.nist.gov/rest/json/cves/2.0/?lastModStartDate={}&lastModEndDate={}",
            last_modified.format("%Y-%m-%dT%H:%M:%S.000"),
            now.format("%Y-%m-%dT%H:%M:%S.000")
        );

        let response = self.http_client.get(&url).send().await?;
        let data: NvdApiResponse = response.json().await?;

        let mut new_last_modified = last_modified;

        for vuln in &data.vulnerabilities {
            let cve = &vuln.cve;
            let _published = cve
                .published
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now());
            let modified = cve
                .last_modified
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now());

            if modified > new_last_modified {
                new_last_modified = modified;
            }

            let event = self.convert_nvd_to_event(cve)?;
            let payload = serde_json::to_vec(&event)?;

            let record = FutureRecord::to(&self.config.nvd_topic)
                .key(event.cve_id.as_bytes())
                .payload(&payload);

            self.producer
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(e, _)| StreamError::Kafka(e))?;
        }

        *self.last_modified.write().await = new_last_modified;
        info!(
            "Published {} CVE events to Kafka",
            data.vulnerabilities.len()
        );

        Ok(())
    }

    fn convert_nvd_to_event(&self, cve: &NvdCveItem) -> Result<NvdCveEvent, StreamError> {
        let cvss_data = cve
            .metrics
            .cvss_metric_v31
            .first()
            .or_else(|| cve.metrics.cvss_metric_v30.first())
            .map(|m| NvdCvssData {
                vector_string: m.cvss_data.vector_string.clone(),
                base_score: m.cvss_data.base_score,
            });

        let cpe_matches = cve
            .configurations
            .iter()
            .flat_map(|c| &c.nodes)
            .flat_map(|n| &n.cpe_match)
            .filter(|m| m.vulnerable)
            .map(|m| m.criteria.clone())
            .collect();

        let description = cve.descriptions.first().map(|d| d.value.clone());

        Ok(NvdCveEvent {
            cve_id: cve.id.clone(),
            published_date: cve.published.parse().ok(),
            last_modified_date: cve.last_modified.parse().ok(),
            cvss_score: cvss_data.as_ref().map(|d| d.base_score).unwrap_or(0.0),
            severity: crate::CvssSeverity::from_score(
                cvss_data.as_ref().map(|d| d.base_score).unwrap_or(0.0),
            ),
            cvss_vector: cvss_data.map(|d| d.vector_string),
            cpe_matches,
            description,
            exploit_available: false,
        })
    }
}

#[derive(Deserialize)]
struct NvdApiResponse {
    vulnerabilities: Vec<NvdVulnerability>,
}

#[derive(Deserialize)]
struct NvdVulnerability {
    cve: NvdCveItem,
}

#[derive(Deserialize, Clone)]
struct NvdCveItem {
    id: String,
    published: String,
    #[serde(rename = "lastModified")]
    last_modified: String,
    #[serde(default)]
    descriptions: Vec<NvdDescription>,
    #[serde(default)]
    metrics: NvdMetrics,
    #[serde(default)]
    configurations: Vec<NvdConfiguration>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct NvdDescription {
    #[serde(rename = "lang")]
    lang: String,
    #[serde(rename = "value")]
    value: String,
}

#[derive(Deserialize, Clone, Default)]
struct NvdMetrics {
    #[serde(rename = "cvssMetricV31", default)]
    cvss_metric_v31: Vec<NvdCvssMetric>,
    #[serde(rename = "cvssMetricV30", default)]
    cvss_metric_v30: Vec<NvdCvssMetric>,
}

#[derive(Deserialize, Clone)]
struct NvdCvssMetric {
    #[serde(rename = "cvssData")]
    cvss_data: NvdCvssData,
}

#[derive(Deserialize, Clone)]
struct NvdCvssData {
    #[serde(rename = "vectorString")]
    vector_string: String,
    #[serde(rename = "baseScore")]
    base_score: f64,
}

#[derive(Deserialize, Clone)]
struct NvdConfiguration {
    #[serde(rename = "nodes")]
    nodes: Vec<NvdNode>,
}

#[derive(Deserialize, Clone)]
struct NvdNode {
    #[serde(rename = "cpeMatch")]
    cpe_match: Vec<NvdCpeMatch>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct NvdCpeMatch {
    #[serde(rename = "criteria")]
    criteria: String,
    #[serde(rename = "vulnerable")]
    vulnerable: bool,
    #[serde(rename = "versionStartIncluding")]
    version_start_including: Option<String>,
    #[serde(rename = "versionEndIncluding")]
    version_end_including: Option<String>,
}

/// Background processor that consumes NVD events and updates asset graph
pub async fn run_cve_stream_pipeline(config: CveStreamConfig) -> Result<(), StreamError> {
    let processor = CveStreamProcessor::new(
        config.clone(),
        Arc::new(RwLock::new(NvdCache::new(3600))),
        Arc::new(RwLock::new(MockAssetGraphProxy::new())),
    )
    .await?;

    let fetcher = NvdFeedFetcher::new(config.clone()).await?;

    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let processor_handle = tokio::spawn(async move {
        if let Err(e) = processor.run().await {
            warn!("CVE stream processor error: {}", e);
        }
    });

    let fetcher_handle = tokio::spawn(async move {
        if let Err(e) = fetcher.run(shutdown_rx).await {
            warn!("NVD fetcher error: {}", e);
        }
    });

    tokio::try_join!(processor_handle, fetcher_handle)?;

    Ok(())
}

/// Mock asset graph proxy for testing
struct MockAssetGraphProxy {
    cpe_to_asset: HashMap<String, String>,
}

impl MockAssetGraphProxy {
    fn new() -> Self {
        let mut proxy = Self {
            cpe_to_asset: HashMap::new(),
        };
        proxy.cpe_to_asset.insert(
            "cpe:2.3:a:apache:http_server:2.4.41:*:*:*:*:*:*:*".to_string(),
            "asset-1".to_string(),
        );
        proxy.cpe_to_asset.insert(
            "cpe:2.3:a:nginx:nginx:1.20.0:*:*:*:*:*:*:*".to_string(),
            "asset-2".to_string(),
        );
        proxy.cpe_to_asset.insert(
            "cpe:2.3:a:openbsd:openssh:8.6:*:*:*:*:*:*:*".to_string(),
            "asset-3".to_string(),
        );
        proxy
    }
}

#[async_trait]
impl AssetGraphProxy for MockAssetGraphProxy {
    async fn get_asset_by_cpe(&self, cpe: &crate::CpeString) -> Option<String> {
        self.cpe_to_asset.get(&cpe.0).cloned()
    }

    async fn add_cve_edge(
        &self,
        source: String,
        target: String,
        cve_id: String,
        cvss: f64,
    ) -> Result<(), String> {
        info!(
            "Adding CVE edge: {} --{} (CVSS:{})--> {}",
            source, cve_id, cvss, target
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nvd_event_conversion() {
        let json = r#"{
            "vulnerabilities": [{
                "cve": {
                    "id": "CVE-2023-12345",
                    "published": "2023-01-15T10:30:00.000",
                    "lastModified": "2023-01-16T14:20:00.000",
                    "descriptions": [{"lang": "en", "value": "Test vulnerability"}],
                    "metrics": {
                        "cvssMetricV31": [{
                            "cvssData": {
                                "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
                                "baseScore": 9.8,
                                "attackVector": "NETWORK",
                                "attackComplexity": "LOW",
                                "privilegesRequired": "NONE",
                                "userInteraction": "NONE",
                                "scope": "UNCHANGED",
                                "confidentialityImpact": "HIGH",
                                "integrityImpact": "HIGH",
                                "availabilityImpact": "HIGH"
                            }
                        }]
                    },
                    "configurations": [{
                        "nodes": [{
                            "cpeMatch": [{
                                "criteria": "cpe:2.3:a:test:product:1.0:*:*:*:*:*:*:*",
                                "vulnerable": true
                            }]
                        }]
                    }]
                }
            }]
        }"#;

        let response: NvdApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.vulnerabilities.len(), 1);

        let cve = &response.vulnerabilities[0].cve;
        assert_eq!(cve.id, "CVE-2023-12345");
        assert!(!cve.metrics.cvss_metric_v31.is_empty());
    }
}
