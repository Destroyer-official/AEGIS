//! Asset Graph Neo4j Export
//!
//! Periodic export of asset relationship graph to Neo4j for graph queries
//! (shortest path, communities, centrality, pattern matching).
//!
//! Uses neo4rs 0.8 (Bolt protocol, tokio 1.x based).

use chrono::{DateTime, Utc};
use neo4rs::{query, BoltType, ConfigBuilder, Graph};
use petgraph::{Directed, Graph as PetGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use crate::{AssetEdge, AssetGraph, AssetId, AssetNode};

#[derive(Error, Debug)]
pub enum Neo4jExportError {
    #[error("Neo4j connection error: {0}")]
    Connection(#[from] neo4rs::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] neo4rs::DeError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Graph export error: {0}")]
    Export(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub batch_size: usize,
    pub export_interval_secs: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://neo4j:7687".to_string(),
            username: "neo4j".to_string(),
            password: String::new(), // Must be configured before use
            database: "aegis".to_string(),
            batch_size: 1000,
            export_interval_secs: 300, // 5 minutes
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub nodes_exported: usize,
    pub edges_exported: usize,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub errors: Vec<String>,
}

pub struct Neo4jExporter {
    config: Neo4jConfig,
    graph: Option<Graph>,
    asset_graph: Arc<RwLock<AssetGraph>>,
    stats_tx: Option<mpsc::Sender<ExportStats>>,
}

impl Neo4jExporter {
    pub fn new(config: Neo4jConfig, asset_graph: Arc<RwLock<AssetGraph>>) -> Self {
        Self {
            config,
            graph: None,
            asset_graph,
            stats_tx: None,
        }
    }

    pub fn with_stats_channel(mut self, tx: mpsc::Sender<ExportStats>) -> Self {
        self.stats_tx = Some(tx);
        self
    }

    /// Connect to Neo4j with retry logic
    pub async fn connect(&mut self) -> Result<(), Neo4jExportError> {
        let mut retries = 0;
        while retries <= self.config.max_retries {
            match self.try_connect().await {
                Ok(graph) => {
                    self.graph = Some(graph);
                    info!("Connected to Neo4j at {}", self.config.uri);
                    return Ok(());
                }
                Err(e) => {
                    retries += 1;
                    if retries > self.config.max_retries {
                        return Err(Neo4jExportError::Connection(e));
                    }
                    warn!(
                        "Neo4j connection attempt {} failed: {}. Retrying in {}ms...",
                        retries, e, self.config.retry_delay_ms
                    );
                    tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                }
            }
        }
        Err(Neo4jExportError::Config("Max retries exceeded".to_string()))
    }

    async fn try_connect(&self) -> Result<Graph, neo4rs::Error> {
        let config = ConfigBuilder::default()
            .uri(&self.config.uri)
            .user(&self.config.username)
            .password(&self.config.password)
            .db(self.config.database.as_str())
            .build()?;

        Graph::connect(config).await
    }

    /// Export the entire asset graph to Neo4j
    pub async fn export_full(&self) -> Result<ExportStats, Neo4jExportError> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| Neo4jExportError::Export("Not connected to Neo4j".to_string()))?;

        let start = std::time::Instant::now();
        let asset_graph = self.asset_graph.read().await;

        // Create constraints and indexes first
        self.setup_schema(graph).await?;

        // Export nodes
        let nodes_exported = self.export_nodes(graph, &asset_graph).await?;

        // Export edges
        let edges_exported = self.export_edges(graph, &asset_graph).await?;

        let duration = start.elapsed().as_millis() as u64;

        let stats = ExportStats {
            nodes_exported,
            edges_exported,
            duration_ms: duration,
            timestamp: Utc::now(),
            errors: Vec::new(),
        };

        info!(
            "Neo4j export complete: {} nodes, {} edges in {}ms",
            nodes_exported, edges_exported, duration
        );

        if let Some(tx) = &self.stats_tx {
            let _ = tx.send(stats.clone()).await;
        }

        Ok(stats)
    }

    /// Setup Neo4j schema (constraints, indexes)
    async fn setup_schema(&self, graph: &Graph) -> Result<(), Neo4jExportError> {
        // Unique constraint on Asset nodes
        graph
            .run(query(
                "CREATE CONSTRAINT asset_id_unique IF NOT EXISTS FOR (a:Asset) REQUIRE a.id IS UNIQUE",
            ))
            .await?;

        // Index on ASN
        graph
            .run(query(
                "CREATE INDEX asset_asn_idx IF NOT EXISTS FOR (a:Asset) ON (a.asn)",
            ))
            .await?;

        // Index on country
        graph
            .run(query(
                "CREATE INDEX asset_country_idx IF NOT EXISTS FOR (a:Asset) ON (a.country)",
            ))
            .await?;

        // Index on edge type
        graph
            .run(query(
                "CREATE INDEX edge_type_idx IF NOT EXISTS FOR ()-[r:CONNECTS]-() ON (r.type)",
            ))
            .await?;

        // Index on CVE edges
        graph
            .run(query(
                "CREATE INDEX cve_edge_idx IF NOT EXISTS FOR ()-[r:CVE_SEVERITY]-() ON (r.cve_id)",
            ))
            .await?;

        // Fulltext index for banner search
        graph
            .run(query(
                "CREATE FULLTEXT INDEX banner_fulltext IF NOT EXISTS FOR (a:Asset) ON EACH [a.banner_hash, a.ja4_fingerprint]",
            ))
            .await?;

        debug!("Neo4j schema created/verified");
        Ok(())
    }

    /// Export asset nodes
    async fn export_nodes(
        &self,
        graph: &Graph,
        asset_graph: &AssetGraph,
    ) -> Result<usize, Neo4jExportError> {
        let mut count = 0;
        let mut batch = Vec::new();

        for node in asset_graph.nodes.values() {
            batch.push(self.node_to_properties(node));

            if batch.len() >= self.config.batch_size {
                self.write_node_batch(graph, &batch).await?;
                count += batch.len();
                batch.clear();
            }
        }

        if !batch.is_empty() {
            self.write_node_batch(graph, &batch).await?;
            count += batch.len();
        }

        Ok(count)
    }

    fn node_to_properties(&self, node: &AssetNode) -> serde_json::Value {
        let mut props = serde_json::json!({
            "id": node.asset_id.to_string(),
            "asn": node.asn,
            "as_org": node.as_org,
            "country": node.country,
            "rdomains": node.rdomains,
            "first_seen": node.first_seen.to_rfc3339(),
            "last_seen": node.last_seen.to_rfc3339(),
        });

        // Export all services as an array
        let services: Vec<serde_json::Value> = node
            .services
            .iter()
            .map(|service| {
                serde_json::json!({
                    "port": service.port,
                    "protocol": service.protocol,
                    "service_name": service.service_name,
                    "banner_hash": service.banner_hash.clone().unwrap_or_default(),
                    "ja4_fingerprint": service.ja4_fingerprint.clone().unwrap_or_default(),
                    "jarm_fingerprint": service.jarm_fingerprint.clone().unwrap_or_default(),
                    "tls_cert_serial": service.tls_cert_serial.clone().unwrap_or_default(),
                    "ssh_host_key": service.ssh_host_key.clone().unwrap_or_default(),
                    "cve_ids": service.cve_ids,
                    "max_cvss_score": service.max_cvss_score,
                    "cpe": service.cpe.clone().unwrap_or_default(),
                })
            })
            .collect();
        props["services"] = serde_json::json!(services);

        props
    }

    async fn write_node_batch(
        &self,
        graph: &Graph,
        batch: &[serde_json::Value],
    ) -> Result<(), Neo4jExportError> {
        let bolt_batch = json_batch_to_bolt(batch)?;

        let q =
            query("UNWIND $batch AS row MERGE (a:Asset {id: row.id}) SET a += row RETURN count(*)")
                .param("batch", bolt_batch);

        let mut result = graph.execute(q).await?;
        result.next().await?;

        Ok(())
    }

    /// Export asset edges (relationships)
    async fn export_edges(
        &self,
        graph: &Graph,
        asset_graph: &AssetGraph,
    ) -> Result<usize, Neo4jExportError> {
        let mut count = 0;
        let mut batch = Vec::new();

        for edge in &asset_graph.edges {
            batch.push(self.edge_to_properties(edge));

            if batch.len() >= self.config.batch_size {
                self.write_edge_batch(graph, &batch).await?;
                count += batch.len();
                batch.clear();
            }
        }

        if !batch.is_empty() {
            self.write_edge_batch(graph, &batch).await?;
            count += batch.len();
        }

        Ok(count)
    }

    fn edge_to_properties(&self, edge: &AssetEdge) -> serde_json::Value {
        serde_json::json!({
            "source": edge.source.to_string(),
            "target": edge.target.to_string(),
            "type": edge.edge_type.to_string(),
            "confidence": edge.confidence,
            "correlation_value": edge.correlation_value,
            "discovered_at": edge.discovered_at.to_rfc3339(),
        })
    }

    async fn write_edge_batch(
        &self,
        graph: &Graph,
        batch: &[serde_json::Value],
    ) -> Result<(), Neo4jExportError> {
        // Group by relationship type since Cypher can't parameterize rel types
        let mut by_type: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        for item in batch {
            let edge_type = item["type"].as_str().unwrap_or("CONNECTS").to_string();
            by_type.entry(edge_type).or_default().push(item.clone());
        }

        for (edge_type, edges) in by_type {
            let rel_type = edge_type_to_rel_type(&edge_type);
            let bolt_batch = json_batch_to_bolt(&edges)?;

            let q = query(&format!(
                "UNWIND $batch AS row MATCH (s:Asset {{id: row.source}}), (t:Asset {{id: row.target}}) \
                 MERGE (s)-[r:{} {{id: row.source + '_' + row.target + '_' + row.type}}]->(t) \
                 SET r += row RETURN count(*)",
                rel_type
            ))
            .param("batch", bolt_batch);

            let mut result = graph.execute(q).await?;
            result.next().await?;
        }

        Ok(())
    }

    /// Run periodic export loop
    pub async fn run_periodic(
        &mut self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), Neo4jExportError> {
        self.connect().await?;

        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.export_interval_secs));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.export_full().await {
                        warn!("Neo4j export failed: {}", e);
                    }
                }
                _ = shutdown.changed() => {
                    info!("Shutdown signal received, stopping Neo4j exporter");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Query utilities for graph analysis
    pub async fn query_shortest_path(
        &self,
        source_ip: &str,
        target_ip: &str,
    ) -> Result<Vec<String>, Neo4jExportError> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| Neo4jExportError::Export("Not connected".to_string()))?;

        let q = query(
            "MATCH (s:Asset {id: $source}), (t:Asset {id: $target}) \
             MATCH p = shortestPath((s)-[*]-(t)) \
             RETURN [n IN nodes(p) | n.id] AS path",
        )
        .param("source", source_ip)
        .param("target", target_ip);

        let mut result = graph.execute(q).await?;
        if let Some(row) = result.next().await? {
            let path: Vec<String> = row.get("path")?;
            Ok(path)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn query_communities(
        &self,
        min_size: usize,
    ) -> Result<Vec<Community>, Neo4jExportError> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| Neo4jExportError::Export("Not connected".to_string()))?;

        let q = query(
            "CALL gds.louvain.stream('aegis-graph', {relationshipTypes: ['CONNECTS', 'SHARED_TLS_CERT', 'SHARED_SSH_HOST_KEY', 'MATCHING_JARM', 'SAME_BGP_AS', 'SHARED_DNS_NAME', 'CORPORATE_BOUNDARY', 'CVE_SEVERITY']}) \
             YIELD nodeId, communityId \
             WITH communityId, collect(nodeId) AS nodes \
             WHERE size(nodes) >= $min_size \
             RETURN communityId, nodes",
        )
        .param("min_size", min_size as i64);

        let mut result = graph.execute(q).await?;
        let mut communities = Vec::new();

        while let Some(row) = result.next().await? {
            let id: i64 = row.get("communityId")?;
            let nodes: Vec<i64> = row.get("nodes")?;
            communities.push(Community {
                id: id as usize,
                nodes: nodes.into_iter().map(|n| n as usize).collect(),
            });
        }

        Ok(communities)
    }

    pub async fn query_centrality(
        &self,
        limit: usize,
    ) -> Result<Vec<CentralityResult>, Neo4jExportError> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| Neo4jExportError::Export("Not connected".to_string()))?;

        let q = query(
            "CALL gds.pageRank.stream('aegis-graph', {maxIterations: 20, dampingFactor: 0.85}) \
             YIELD nodeId, score \
             WITH gds.util.asNode(nodeId) AS node, score \
             ORDER BY score DESC \
             LIMIT $limit \
             RETURN node.id AS asset_id, score",
        )
        .param("limit", limit as i64);

        let mut result = graph.execute(q).await?;
        let mut centrality = Vec::new();

        while let Some(row) = result.next().await? {
            let asset_id: String = row.get("asset_id")?;
            let score: f64 = row.get("score")?;
            centrality.push(CentralityResult { asset_id, score });
        }

        Ok(centrality)
    }

    pub async fn query_cve_blast_radius(
        &self,
        cve_id: &str,
    ) -> Result<CveBlastRadius, Neo4jExportError> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| Neo4jExportError::Export("Not connected".to_string()))?;

        let q = query(
            "MATCH (a:Asset)-[r:CVE_SEVERITY {cve_id: $cve}]->(b:Asset) \
             RETURN collect(DISTINCT a.id) AS sources, collect(DISTINCT b.id) AS targets, count(r) AS edges",
        )
        .param("cve", cve_id);

        let mut result = graph.execute(q).await?;
        if let Some(row) = result.next().await? {
            let sources: Vec<String> = row.get("sources")?;
            let targets: Vec<String> = row.get("targets")?;
            let edges: i64 = row.get("edges")?;

            // Get all unique affected assets
            let mut all_assets = HashSet::new();
            all_assets.extend(sources.iter().cloned());
            all_assets.extend(targets.iter().cloned());

            Ok(CveBlastRadius {
                cve_id: cve_id.to_string(),
                affected_assets: all_assets.into_iter().collect(),
                edge_count: edges as usize,
            })
        } else {
            Ok(CveBlastRadius {
                cve_id: cve_id.to_string(),
                affected_assets: Vec::new(),
                edge_count: 0,
            })
        }
    }

    /// Convert to petgraph for local algorithms
    pub fn to_petgraph(&self) -> PetGraph<AssetId, AssetEdge, Directed> {
        let asset_graph = self.asset_graph.blocking_read();
        crate::edges::to_petgraph(&asset_graph)
    }
}

/// Convert an edge type string to a Neo4j relationship type.
fn edge_type_to_rel_type(edge_type: &str) -> &'static str {
    match edge_type {
        "shared_tls_cert" => "SHARED_TLS_CERT",
        "shared_ssh_host_key" => "SHARED_SSH_HOST_KEY",
        "matching_jarm" => "MATCHING_JARM",
        "same_bgp_as" => "SAME_BGP_AS",
        "shared_dns_name" => "SHARED_DNS_NAME",
        "corporate_boundary" => "CORPORATE_BOUNDARY",
        "cve_severity" => "CVE_SEVERITY",
        _ => "CONNECTS",
    }
}

/// Convert a batch of JSON values into a Bolt list parameter.
fn json_batch_to_bolt(batch: &[serde_json::Value]) -> Result<BoltType, Neo4jExportError> {
    let items: Vec<BoltType> = batch
        .iter()
        .cloned()
        .map(|v| {
            v.try_into()
                .map_err(|e| Neo4jExportError::Export(format!("JSON→Bolt conversion failed: {e}")))
        })
        .collect::<Result<_, _>>()?;
    Ok(items.into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub nodes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityResult {
    pub asset_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveBlastRadius {
    pub cve_id: String,
    pub affected_assets: Vec<String>,
    pub edge_count: usize,
}

/// Export runner that manages the Neo4j exporter lifecycle
pub struct ExportRunner {
    exporter: Neo4jExporter,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl ExportRunner {
    pub fn new(config: Neo4jConfig, asset_graph: Arc<RwLock<AssetGraph>>) -> Self {
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        let exporter = Neo4jExporter::new(config, asset_graph);

        Self {
            exporter,
            shutdown_tx,
        }
    }

    pub async fn start(&mut self) -> Result<(), Neo4jExportError> {
        let shutdown_rx = self.shutdown_tx.subscribe();
        self.exporter.run_periodic(shutdown_rx).await
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neo4j_config_default() {
        let config = Neo4jConfig::default();
        assert_eq!(config.uri, "bolt://neo4j:7687");
        assert_eq!(config.username, "neo4j");
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.export_interval_secs, 300);
    }

    #[test]
    fn test_edge_type_to_rel_type() {
        assert_eq!(edge_type_to_rel_type("shared_tls_cert"), "SHARED_TLS_CERT");
        assert_eq!(
            edge_type_to_rel_type("shared_ssh_host_key"),
            "SHARED_SSH_HOST_KEY"
        );
        assert_eq!(edge_type_to_rel_type("matching_jarm"), "MATCHING_JARM");
        assert_eq!(edge_type_to_rel_type("same_bgp_as"), "SAME_BGP_AS");
        assert_eq!(edge_type_to_rel_type("shared_dns_name"), "SHARED_DNS_NAME");
        assert_eq!(
            edge_type_to_rel_type("corporate_boundary"),
            "CORPORATE_BOUNDARY"
        );
        assert_eq!(edge_type_to_rel_type("cve_severity"), "CVE_SEVERITY");
        assert_eq!(edge_type_to_rel_type("unknown"), "CONNECTS");
    }

    #[test]
    fn test_export_stats() {
        let stats = ExportStats {
            nodes_exported: 1000,
            edges_exported: 5000,
            duration_ms: 1500,
            timestamp: Utc::now(),
            errors: Vec::new(),
        };

        assert_eq!(stats.nodes_exported, 1000);
        assert_eq!(stats.edges_exported, 5000);
        assert!(stats.duration_ms > 0);
    }

    #[test]
    fn test_json_batch_to_bolt() {
        let batch = vec![
            serde_json::json!({"id": "192.168.1.1", "asn": 64512}),
            serde_json::json!({"id": "192.168.1.2", "asn": 64513}),
        ];

        let bolt = json_batch_to_bolt(&batch).unwrap();
        match bolt {
            BoltType::List(list) => assert_eq!(list.value.len(), 2),
            _ => panic!("expected BoltType::List"),
        }
    }

    #[test]
    fn test_json_batch_to_bolt_nested() {
        let batch = vec![serde_json::json!({
            "id": "10.0.0.1",
            "services": [{"port": 443, "tls": true}],
            "cves": [],
        })];

        assert!(json_batch_to_bolt(&batch).is_ok());
    }
}
