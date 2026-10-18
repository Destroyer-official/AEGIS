use std::collections::HashMap;

use quick_xml::de::from_reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CveFeedError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CVE feed validation error: {0}")]
    Validation(String),
}

// NVD XML structures (simplified for feed parsing)

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdFeed {
    #[serde(rename = "entry")]
    pub entries: Vec<NvdEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdEntry {
    #[serde(rename = "vuln:vulnerable-software-list")]
    pub vulnerable_software_list: Option<VulnerableSoftwareList>,
    #[serde(rename = "vuln:cve-id")]
    pub cve_id: String,
    #[serde(rename = "vuln:cvss")]
    pub cvss: Option<Cvss>,
    #[serde(rename = "vuln:summary")]
    pub summary: Option<String>,
    #[serde(rename = "vuln:published-datetime")]
    pub published_datetime: Option<String>,
    #[serde(rename = "vuln:last-modified-datetime")]
    pub last_modified_datetime: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VulnerableSoftwareList {
    #[serde(rename = "vuln:product")]
    pub products: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cvss {
    #[serde(rename = "cvss:base_metrics")]
    pub base_metrics: Option<BaseMetrics>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseMetrics {
    #[serde(rename = "cvss:score")]
    pub score: Option<f64>,
    #[serde(rename = "cvss:access-vector")]
    pub access_vector: Option<String>,
    #[serde(rename = "cvss:access-complexity")]
    pub access_complexity: Option<String>,
    #[serde(rename = "cvss:authentication")]
    pub authentication: Option<String>,
    #[serde(rename = "cvss:confidentiality-impact")]
    pub confidentiality_impact: Option<String>,
    #[serde(rename = "cvss:integrity-impact")]
    pub integrity_impact: Option<String>,
    #[serde(rename = "cvss:availability-impact")]
    pub availability_impact: Option<String>,
}

// Parsed CVE record for internal use

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveRecord {
    pub cve_id: String,
    pub cpe_matches: Vec<String>,
    pub cvss_score: Option<f64>,
    pub cvss_vector: Option<String>,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub last_modified: Option<String>,
}

impl CveRecord {
    pub fn from_entry(entry: NvdEntry) -> Self {
        let cpe_matches = entry
            .vulnerable_software_list
            .map(|vsl| vsl.products)
            .unwrap_or_default();

        let (cvss_score, cvss_vector) = entry
            .cvss
            .and_then(|c| c.base_metrics)
            .map(|bm| {
                let score = bm.score;
                let vector = build_cvss_vector(&bm);
                (score, Some(vector))
            })
            .unwrap_or((None, None));

        Self {
            cve_id: entry.cve_id,
            cpe_matches,
            cvss_score,
            cvss_vector,
            summary: entry.summary,
            published: entry.published_datetime,
            last_modified: entry.last_modified_datetime,
        }
    }
}

fn build_cvss_vector(bm: &BaseMetrics) -> String {
    let mut parts = Vec::new();
    if let Some(av) = &bm.access_vector {
        parts.push(format!(
            "AV:{}",
            av.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    if let Some(ac) = &bm.access_complexity {
        parts.push(format!(
            "AC:{}",
            ac.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    if let Some(au) = &bm.authentication {
        parts.push(format!(
            "Au:{}",
            au.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    if let Some(c) = &bm.confidentiality_impact {
        parts.push(format!(
            "C:{}",
            c.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    if let Some(i) = &bm.integrity_impact {
        parts.push(format!(
            "I:{}",
            i.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    if let Some(a) = &bm.availability_impact {
        parts.push(format!(
            "A:{}",
            a.chars().next().unwrap_or('N').to_uppercase()
        ));
    }
    parts.join("/")
}

// CVE Feed Aggregator

pub struct CveFeedAggregator {
    client: reqwest::Client,
    cve_index: HashMap<String, CveRecord>,
    cpe_to_cves: HashMap<String, Vec<String>>,
}

impl CveFeedAggregator {
    pub fn new() -> Result<Self, CveFeedError> {
        let client = reqwest::Client::builder()
            .user_agent("AEGIS-CVE-Aggregator/1.0")
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(CveFeedError::Http)?;
        Ok(Self {
            client,
            cve_index: HashMap::new(),
            cpe_to_cves: HashMap::new(),
        })
    }

    /// Download and parse NVD yearly XML feed (e.g., 2024)
    pub async fn fetch_nvd_year(&mut self, year: u16) -> Result<usize, CveFeedError> {
        let url = format!(
            "https://nvd.nist.gov/feeds/xml/cve/nvdcve-2.0-{}.xml.gz",
            year
        );
        let resp = self.client.get(&url).send().await?;
        let bytes = resp.bytes().await?;

        // Decompress gzip
        let decoder = flate2::read::GzDecoder::new(bytes.as_ref());
        let mut reader = std::io::BufReader::new(decoder);

        self.parse_nvd_xml(&mut reader)
    }

    /// Parse NVD XML from a reader
    pub fn parse_nvd_xml<R: std::io::BufRead>(
        &mut self,
        reader: &mut R,
    ) -> Result<usize, CveFeedError> {
        let feed: NvdFeed = from_reader(reader)?;
        let count = feed.entries.len();

        for entry in feed.entries {
            let record = CveRecord::from_entry(entry);

            // Index by CVE ID
            self.cve_index.insert(record.cve_id.clone(), record.clone());

            // Index by CPE
            for cpe in &record.cpe_matches {
                self.cpe_to_cves
                    .entry(cpe.clone())
                    .or_default()
                    .push(record.cve_id.clone());
            }
        }

        Ok(count)
    }

    /// Parse from local XML file (for testing)
    pub fn parse_nvd_file(&mut self, path: &str) -> Result<usize, CveFeedError> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        self.parse_nvd_xml(&mut reader)
    }

    /// Get CVEs for a specific CPE
    pub fn cves_for_cpe(&self, cpe: &str) -> Option<&Vec<String>> {
        self.cpe_to_cves.get(cpe)
    }

    /// Get CVE record by ID
    pub fn get_cve(&self, cve_id: &str) -> Option<&CveRecord> {
        self.cve_index.get(cve_id)
    }

    /// Get all CPE->CVSS mappings for a list of CPEs
    pub fn cpe_cvss_mappings(&self, cpes: &[String]) -> HashMap<String, Vec<CveRecord>> {
        let mut result = HashMap::new();
        for cpe in cpes {
            if let Some(cve_ids) = self.cpe_to_cves.get(cpe) {
                let records: Vec<CveRecord> = cve_ids
                    .iter()
                    .filter_map(|id| self.cve_index.get(id).cloned())
                    .collect();
                if !records.is_empty() {
                    result.insert(cpe.clone(), records);
                }
            }
        }
        result
    }

    /// Total CVEs indexed
    pub fn total_cves(&self) -> usize {
        self.cve_index.len()
    }

    /// Total unique CPEs indexed
    pub fn total_cpes(&self) -> usize {
        self.cpe_to_cves.len()
    }
}

// CPE parsing utilities

pub fn parse_cpe_uri(cpe: &str) -> Option<CpeParts> {
    // cpe:2.3:a:vendor:product:version:update:edition:language:sw_edition:target_sw:target_hw:other
    let parts: Vec<&str> = cpe.split(':').collect();
    if parts.len() < 6 || parts[0] != "cpe" || parts[1] != "2.3" {
        return None;
    }
    Some(CpeParts {
        part: parts[2].to_string(),
        vendor: parts[3].to_string(),
        product: parts[4].to_string(),
        version: parts.get(5).copied().unwrap_or("*").to_string(),
        update: parts.get(6).copied().unwrap_or("*").to_string(),
        edition: parts.get(7).copied().unwrap_or("*").to_string(),
        language: parts.get(8).copied().unwrap_or("*").to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpeParts {
    pub part: String,
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub update: String,
    pub edition: String,
    pub language: String,
}

impl CpeParts {
    pub fn matches(&self, other: &CpeParts) -> bool {
        (self.part == other.part || other.part == "*")
            && (self.vendor == other.vendor || other.vendor == "*")
            && (self.product == other.product || other.product == "*")
            && (self.version == other.version || other.version == "*")
    }
}

// Integration hook for Dev 8's CVE crate
pub mod dev8_integration {
    use super::*;
    use serde_json::Value;

    /// Convert CveRecord to Dev 8's expected JSON format
    pub fn to_dev8_format(record: &CveRecord) -> Value {
        serde_json::json!({
            "cve_id": record.cve_id,
            "cpe_list": record.cpe_matches,
            "cvss_score": record.cvss_score,
            "cvss_vector": record.cvss_vector,
            "summary": record.summary,
            "published": record.published,
            "modified": record.last_modified,
        })
    }

    /// Batch convert for Dev 8 ingestion
    pub fn batch_to_dev8(records: &[CveRecord]) -> Vec<Value> {
        records.iter().map(to_dev8_format).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cve_record_from_entry() {
        let entry = NvdEntry {
            vulnerable_software_list: Some(VulnerableSoftwareList {
                products: vec![
                    "cpe:2.3:a:apache:http_server:2.4.41:*:*:*:*:*:*:*".to_string(),
                    "cpe:2.3:a:nginx:nginx:1.18.0:*:*:*:*:*:*:*".to_string(),
                ],
            }),
            cve_id: "CVE-2021-44228".to_string(),
            cvss: Some(Cvss {
                base_metrics: Some(BaseMetrics {
                    score: Some(10.0),
                    access_vector: Some("NETWORK".to_string()),
                    access_complexity: Some("LOW".to_string()),
                    authentication: Some("NONE".to_string()),
                    confidentiality_impact: Some("COMPLETE".to_string()),
                    integrity_impact: Some("COMPLETE".to_string()),
                    availability_impact: Some("COMPLETE".to_string()),
                }),
            }),
            summary: Some("Log4j RCE".to_string()),
            published_datetime: Some("2021-12-10T00:00:00".to_string()),
            last_modified_datetime: Some("2021-12-13T00:00:00".to_string()),
        };

        let record = CveRecord::from_entry(entry);
        assert_eq!(record.cve_id, "CVE-2021-44228");
        assert_eq!(record.cpe_matches.len(), 2);
        assert_eq!(record.cvss_score, Some(10.0));
        assert!(record.cvss_vector.as_ref().unwrap().contains("AV:N"));
    }

    #[test]
    fn test_parse_cpe_uri() {
        let cpe = "cpe:2.3:a:apache:http_server:2.4.41:*:*:*:*:*:*:*";
        let parts = parse_cpe_uri(cpe).unwrap();
        assert_eq!(parts.part, "a");
        assert_eq!(parts.vendor, "apache");
        assert_eq!(parts.product, "http_server");
        assert_eq!(parts.version, "2.4.41");
    }

    #[test]
    fn test_cpe_matching() {
        let cpe1 = parse_cpe_uri("cpe:2.3:a:apache:http_server:2.4.41:*:*:*:*:*:*:*").unwrap();
        let cpe2 = parse_cpe_uri("cpe:2.3:a:apache:http_server:*:*:*:*:*:*:*").unwrap();
        let cpe3 = parse_cpe_uri("cpe:2.3:a:nginx:nginx:1.18.0:*:*:*:*:*:*:*").unwrap();

        assert!(cpe1.matches(&cpe2));
        assert!(!cpe1.matches(&cpe3));
    }

    #[test]
    fn test_aggregator_indexing() {
        let mut agg = CveFeedAggregator::new().unwrap();

        let entry = NvdEntry {
            vulnerable_software_list: Some(VulnerableSoftwareList {
                products: vec!["cpe:2.3:a:test:product:1.0:*:*:*:*:*:*:*".to_string()],
            }),
            cve_id: "CVE-2024-1234".to_string(),
            cvss: Some(Cvss {
                base_metrics: Some(BaseMetrics {
                    score: Some(7.5),
                    access_vector: Some("NETWORK".to_string()),
                    access_complexity: Some("LOW".to_string()),
                    authentication: Some("NONE".to_string()),
                    confidentiality_impact: Some("PARTIAL".to_string()),
                    integrity_impact: Some("PARTIAL".to_string()),
                    availability_impact: Some("PARTIAL".to_string()),
                }),
            }),
            summary: Some("Test CVE".to_string()),
            published_datetime: Some("2024-01-01T00:00:00".to_string()),
            last_modified_datetime: Some("2024-01-02T00:00:00".to_string()),
        };

        let record = CveRecord::from_entry(entry);
        agg.cve_index.insert(record.cve_id.clone(), record.clone());
        for cpe in &record.cpe_matches {
            agg.cpe_to_cves
                .entry(cpe.clone())
                .or_default()
                .push(record.cve_id.clone());
        }

        assert_eq!(agg.total_cves(), 1);
        assert_eq!(agg.total_cpes(), 1);
        assert!(
            agg.cves_for_cpe("cpe:2.3:a:test:product:1.0:*:*:*:*:*:*:*")
                .is_some()
        );
    }
}
