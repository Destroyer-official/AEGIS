//! Asset relationship graphing for AEGIS intelligence enrichment.
//! §3.5.3 — Asset-Relationship Graphing
//!
//! Correlates shared infrastructure characteristics across distinct IP addresses:
//! - TLS certificate serial numbers
//! - SSH host keys
//! - JARM TLS fingerprints
//! - BGP AS membership with similar service profiles
//! - DNS name resolution (shared rDNS/forward DNS)
//!
//! Produces a directed edge list suitable for downstream graph storage
//! (Neo4j/JanusGraph) or visualization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

mod edges;
mod neo4j_export;
pub use edges::{
    filter_edges_by_type, find_critical_cve_assets, get_cve_connected_assets, get_cve_edges,
    to_petgraph, top_cve_connected_assets, CveSeveritySummary,
};
pub use neo4j_export::{
    CentralityResult, Community, CveBlastRadius, ExportRunner, ExportStats, Neo4jConfig,
    Neo4jExporter,
};

// ───────── Core data model ─────────

/// Unique identifier for an asset (IP address + optional service context).
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetId {
    pub ip: IpAddr,
    /// Optional port to scope the edge to a specific service.
    pub port: Option<u16>,
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.port {
            Some(p) => write!(f, "{}:{}", self.ip, p),
            None => write!(f, "{}", self.ip),
        }
    }
}

/// Type of correlation that links two assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Both assets serve the same TLS certificate (by serial number).
    SharedTlsCert,
    /// Both assets serve the same SSH host key.
    SharedSshHostKey,
    /// Both assets produce matching JARM TLS fingerprints.
    MatchingJarm,
    /// Both assets are in the same BGP AS with similar service profiles.
    SameBgpAs,
    /// Both assets resolve to the same DNS name.
    SharedDnsName,
    /// Corporate infrastructure boundary (heuristic from cert/WHOIS overlap).
    CorporateBoundary,
    /// Both assets share known CVE vulnerabilities (CPE correlation → NVD → CVSS).
    CveSeverity,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::SharedTlsCert => write!(f, "shared_tls_cert"),
            EdgeType::SharedSshHostKey => write!(f, "shared_ssh_host_key"),
            EdgeType::MatchingJarm => write!(f, "matching_jarm"),
            EdgeType::SameBgpAs => write!(f, "same_bgp_as"),
            EdgeType::SharedDnsName => write!(f, "shared_dns_name"),
            EdgeType::CorporateBoundary => write!(f, "corporate_boundary"),
            EdgeType::CveSeverity => write!(f, "cve_severity"),
        }
    }
}

/// A directed edge in the asset relationship graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEdge {
    pub source: AssetId,
    pub target: AssetId,
    pub edge_type: EdgeType,
    /// Confidence in this correlation (0.0–1.0).
    pub confidence: f64,
    /// The raw value that produced this edge (e.g., cert serial, JARM hash).
    pub correlation_value: String,
    /// When this edge was discovered.
    pub discovered_at: DateTime<Utc>,
}

/// An asset node in the graph with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetNode {
    pub asset_id: AssetId,
    pub asn: Option<u32>,
    pub as_org: Option<String>,
    pub country: Option<String>,
    pub rdomains: Vec<String>,
    pub services: Vec<ServiceInfo>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Service metadata attached to an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub port: u16,
    pub protocol: String,
    pub service_name: String,
    pub banner_hash: Option<String>,
    pub ja4_fingerprint: Option<String>,
    pub jarm_fingerprint: Option<String>,
    pub tls_cert_serial: Option<String>,
    pub ssh_host_key: Option<String>,
    /// CVE IDs associated with this service (from CPE correlation).
    pub cve_ids: Vec<String>,
    /// Maximum CVSS base score among associated CVEs.
    pub max_cvss_score: Option<f64>,
    /// CPE string that matched (for traceability).
    pub cpe: Option<String>,
}

/// The complete asset relationship graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetGraph {
    /// All known asset nodes keyed by AssetId.
    pub nodes: HashMap<AssetId, AssetNode>,
    /// All edges in the graph.
    pub edges: Vec<AssetEdge>,
    /// Correlation value → set of AssetIds (for dedup of shared secrets).
    cert_index: HashMap<String, HashSet<AssetId>>,
    ssh_key_index: HashMap<String, HashSet<AssetId>>,
    jarm_index: HashMap<String, HashSet<AssetId>>,
    dns_index: HashMap<String, HashSet<AssetId>>,
    bgp_as_index: HashMap<u32, HashSet<AssetId>>,
    /// CVE ID → set of AssetIds (for CVE severity correlation).
    cve_index: HashMap<String, HashSet<AssetId>>,
}

impl AssetGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an asset node.
    pub fn add_node(&mut self, node: AssetNode) {
        self.nodes.insert(node.asset_id.clone(), node);
    }

    /// Ingest a service record and generate correlation edges.
    ///
    /// This is the primary entry point for streaming enrichment.
    /// Each call checks the new service against all existing services
    /// for shared secrets and produces edges.
    pub fn ingest_service(
        &mut self,
        asset_id: AssetId,
        service: ServiceInfo,
        node_defaults: AssetNodeDefaults,
    ) -> Vec<AssetEdge> {
        let mut new_edges = Vec::new();

        // Ensure node exists
        self.nodes
            .entry(asset_id.clone())
            .or_insert_with(|| AssetNode {
                asset_id: asset_id.clone(),
                asn: node_defaults.asn,
                as_org: node_defaults.as_org,
                country: node_defaults.country,
                rdomains: Vec::new(),
                services: Vec::new(),
                first_seen: Utc::now(),
                last_seen: Utc::now(),
            });

        // Update services list
        if let Some(node) = self.nodes.get_mut(&asset_id) {
            node.services.push(service.clone());
            node.last_seen = Utc::now();
        }

        // 1. TLS certificate serial correlation
        if let Some(ref serial) = service.tls_cert_serial {
            let entry = self.cert_index.entry(serial.clone()).or_default();
            for peer in entry.iter() {
                if peer != &asset_id {
                    let edge = AssetEdge {
                        source: asset_id.clone(),
                        target: peer.clone(),
                        edge_type: EdgeType::SharedTlsCert,
                        confidence: 0.95,
                        correlation_value: serial.clone(),
                        discovered_at: Utc::now(),
                    };
                    new_edges.push(edge.clone());
                    self.edges.push(edge);
                }
            }
            entry.insert(asset_id.clone());
        }

        // 2. SSH host key correlation
        if let Some(ref key) = service.ssh_host_key {
            let entry = self.ssh_key_index.entry(key.clone()).or_default();
            for peer in entry.iter() {
                if peer != &asset_id {
                    let edge = AssetEdge {
                        source: asset_id.clone(),
                        target: peer.clone(),
                        edge_type: EdgeType::SharedSshHostKey,
                        confidence: 0.98,
                        correlation_value: key.clone(),
                        discovered_at: Utc::now(),
                    };
                    new_edges.push(edge.clone());
                    self.edges.push(edge);
                }
            }
            entry.insert(asset_id.clone());
        }

        // 3. JARM fingerprint correlation
        if let Some(ref jarm) = service.jarm_fingerprint {
            let entry = self.jarm_index.entry(jarm.clone()).or_default();
            for peer in entry.iter() {
                if peer != &asset_id {
                    let edge = AssetEdge {
                        source: asset_id.clone(),
                        target: peer.clone(),
                        edge_type: EdgeType::MatchingJarm,
                        confidence: 0.80,
                        correlation_value: jarm.clone(),
                        discovered_at: Utc::now(),
                    };
                    new_edges.push(edge.clone());
                    self.edges.push(edge);
                }
            }
            entry.insert(asset_id.clone());
        }

        new_edges
    }

    /// Ingest CVE correlations and generate CveSeverity edges.
    ///
    /// Links assets that share the same CVE ID, with edge weight based on CVSS severity.
    pub fn ingest_cves(
        &mut self,
        asset_id: AssetId,
        cve_ids: &[String],
        max_cvss: Option<f64>,
    ) -> Vec<AssetEdge> {
        let mut new_edges = Vec::new();

        for cve_id in cve_ids {
            let entry = self.cve_index.entry(cve_id.clone()).or_default();

            for peer in entry.iter() {
                if peer != &asset_id {
                    // Confidence based on CVSS score (higher severity = higher confidence)
                    let confidence = max_cvss.map(|s| (s / 10.0).clamp(0.5, 0.95)).unwrap_or(0.6);
                    // Include CVSS in correlation_value for downstream analysis
                    let corr_value = max_cvss
                        .map(|s| format!("{} CVSS={:.1}", cve_id, s))
                        .unwrap_or(cve_id.clone());
                    let edge = AssetEdge {
                        source: asset_id.clone(),
                        target: peer.clone(),
                        edge_type: EdgeType::CveSeverity,
                        confidence,
                        correlation_value: corr_value,
                        discovered_at: Utc::now(),
                    };
                    new_edges.push(edge.clone());
                    self.edges.push(edge);
                }
            }

            entry.insert(asset_id.clone());
        }

        new_edges
    }

    /// Ingest a DNS resolution and check for shared names.
    pub fn ingest_dns(&mut self, asset_id: AssetId, hostname: &str) -> Vec<AssetEdge> {
        let mut new_edges = Vec::new();
        let entry = self.dns_index.entry(hostname.to_string()).or_default();

        for peer in entry.iter() {
            if peer != &asset_id {
                let edge = AssetEdge {
                    source: asset_id.clone(),
                    target: peer.clone(),
                    edge_type: EdgeType::SharedDnsName,
                    confidence: 0.70,
                    correlation_value: hostname.to_string(),
                    discovered_at: Utc::now(),
                };
                new_edges.push(edge.clone());
                self.edges.push(edge);
            }
        }

        // Update rdomains on the node
        if let Some(node) = self.nodes.get_mut(&asset_id) {
            if !node.rdomains.contains(&hostname.to_string()) {
                node.rdomains.push(hostname.to_string());
            }
        }

        entry.insert(asset_id);
        new_edges
    }

    /// Ingest BGP AS membership and check for co-located services.
    pub fn ingest_bgp_as(
        &mut self,
        asset_id: AssetId,
        asn: u32,
        as_org: &str,
        service_profile: &str,
    ) -> Vec<AssetEdge> {
        let mut new_edges = Vec::new();

        // Update node metadata
        if let Some(node) = self.nodes.get_mut(&asset_id) {
            node.asn = Some(asn);
            node.as_org = Some(as_org.to_string());
        }

        let entry = self.bgp_as_index.entry(asn).or_default();

        // Check for matching service profiles within same AS
        let profile_clone = service_profile.to_string();
        for peer in entry.iter() {
            if peer != &asset_id {
                if let Some(peer_node) = self.nodes.get(peer) {
                    // Heuristic: same ASN + at least one matching service port
                    let peer_ports: HashSet<u16> =
                        peer_node.services.iter().map(|s| s.port).collect();
                    let our_ports: HashSet<u16> = self
                        .nodes
                        .get(&asset_id)
                        .map(|n| n.services.iter().map(|s| s.port).collect())
                        .unwrap_or_default();
                    let overlap = our_ports.intersection(&peer_ports).count();

                    if overlap > 0 {
                        let edge = AssetEdge {
                            source: asset_id.clone(),
                            target: peer.clone(),
                            edge_type: EdgeType::SameBgpAs,
                            confidence: 0.50 + (overlap as f64 * 0.10).min(0.30),
                            correlation_value: format!("AS{}:{}", asn, profile_clone),
                            discovered_at: Utc::now(),
                        };
                        new_edges.push(edge.clone());
                        self.edges.push(edge);
                    }
                }
            }
        }

        entry.insert(asset_id);
        new_edges
    }

    /// Generate corporate boundary edges by clustering assets
    /// that share multiple correlation types.
    pub fn infer_corporate_boundaries(&mut self) -> Vec<AssetEdge> {
        let mut new_edges = Vec::new();

        // Build a co-occurrence matrix: (AssetId, AssetId) → count of shared edges
        let mut co_occurrence: HashMap<(AssetId, AssetId), usize> = HashMap::new();

        for edge in &self.edges {
            // Skip corporate boundary edges (we're building those)
            if edge.edge_type == EdgeType::CorporateBoundary {
                continue;
            }
            let key = if edge.source < edge.target {
                (edge.source.clone(), edge.target.clone())
            } else {
                (edge.target.clone(), edge.source.clone())
            };
            *co_occurrence.entry(key).or_insert(0) += 1;
        }

        // Assets sharing 3+ correlation types are likely corporate infrastructure
        for ((a, b), count) in &co_occurrence {
            if *count >= 3 {
                let edge = AssetEdge {
                    source: a.clone(),
                    target: b.clone(),
                    edge_type: EdgeType::CorporateBoundary,
                    confidence: (*count as f64 / 5.0).min(0.95),
                    correlation_value: format!("co_occurrence:{}", count),
                    discovered_at: Utc::now(),
                };
                new_edges.push(edge.clone());
                self.edges.push(edge);
            }
        }

        new_edges
    }

    /// Export the graph as a flat edge list (source, target, edge_type, confidence).
    pub fn to_edge_list(&self) -> Vec<EdgeListEntry> {
        self.edges
            .iter()
            .map(|e| EdgeListEntry {
                source: e.source.to_string(),
                target: e.target.to_string(),
                edge_type: e.edge_type.to_string(),
                confidence: e.confidence,
                correlation_value: e.correlation_value.clone(),
            })
            .collect()
    }

    /// Export the graph as JSON for downstream graph databases (Neo4j/JanusGraph).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let export = GraphExport {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.clone(),
        };
        serde_json::to_string_pretty(&export)
    }

    /// Get all edges for a specific asset.
    pub fn edges_for(&self, asset: &AssetId) -> Vec<&AssetEdge> {
        self.edges
            .iter()
            .filter(|e| &e.source == asset || &e.target == asset)
            .collect()
    }

    /// Get all neighbors of an asset (regardless of edge direction).
    pub fn neighbors(&self, asset: &AssetId) -> HashSet<AssetId> {
        let mut neighbors = HashSet::new();
        for edge in &self.edges {
            if &edge.source == asset {
                neighbors.insert(edge.target.clone());
            } else if &edge.target == asset {
                neighbors.insert(edge.source.clone());
            }
        }
        neighbors
    }

    /// Get the subgraph reachable from an asset within `max_hops` (inclusive).
    ///
    /// `max_hops=0` returns only the start node.
    /// `max_hops=1` returns the start node and its direct neighbors.
    pub fn subgraph_from(
        &self,
        start: &AssetId,
        max_hops: usize,
    ) -> (HashSet<AssetId>, Vec<&AssetEdge>) {
        let mut visited = HashSet::new();
        let mut frontier = HashSet::new();
        frontier.insert(start.clone());
        let mut edge_set = Vec::new();

        for _ in 0..=max_hops {
            let mut next_frontier = HashSet::new();
            for asset in &frontier {
                if visited.contains(asset) {
                    continue;
                }
                visited.insert(asset.clone());
                for edge in &self.edges {
                    if &edge.source == asset && !visited.contains(&edge.target) {
                        edge_set.push(edge);
                        next_frontier.insert(edge.target.clone());
                    } else if &edge.target == asset && !visited.contains(&edge.source) {
                        edge_set.push(edge);
                        next_frontier.insert(edge.source.clone());
                    }
                }
            }
            frontier = next_frontier;
            if frontier.is_empty() {
                break;
            }
        }

        (visited, edge_set)
    }

    /// Summary statistics for the graph.
    pub fn stats(&self) -> GraphStats {
        let mut edge_type_counts: HashMap<EdgeType, usize> = HashMap::new();
        for edge in &self.edges {
            *edge_type_counts.entry(edge.edge_type).or_insert(0) += 1;
        }

        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            edge_type_counts,
        }
    }
}

/// Defaults for new asset nodes (set during ingestion).
#[derive(Debug, Clone, Default)]
pub struct AssetNodeDefaults {
    pub asn: Option<u32>,
    pub as_org: Option<String>,
    pub country: Option<String>,
}

/// Flat edge list entry for CSV/JSON export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeListEntry {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub confidence: f64,
    pub correlation_value: String,
}

/// Graph export format for JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExport {
    pub nodes: Vec<AssetNode>,
    pub edges: Vec<AssetEdge>,
}

/// Summary statistics for the asset graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub edge_type_counts: HashMap<EdgeType, usize>,
}

// ───────── petgraph integration ─────────

impl AssetGraph {
    /// Convert to a petgraph `Graph` with simplified edge data (`EdgeType`).
    ///
    /// This version deduplicates edges by keeping only the highest confidence edge
    /// for each (source, target, edge_type) triple. Use this when you need a simple
    /// graph structure for algorithms that don't require full edge metadata.
    pub fn to_petgraph(&self) -> petgraph::graph::Graph<AssetId, EdgeType> {
        let mut g = petgraph::graph::Graph::new();
        let mut node_indices: HashMap<AssetId, petgraph::graph::NodeIndex> = HashMap::new();

        // Add all nodes
        for asset_id in self.nodes.keys() {
            let idx = g.add_node(asset_id.clone());
            node_indices.insert(asset_id.clone(), idx);
        }

        // Add edges (deduplicate by keeping highest confidence)
        let mut best_edge: HashMap<(AssetId, AssetId, EdgeType), &AssetEdge> = HashMap::new();
        for edge in &self.edges {
            let key = (edge.source.clone(), edge.target.clone(), edge.edge_type);
            match best_edge.get(&key) {
                Some(existing) if existing.confidence >= edge.confidence => {}
                _ => {
                    best_edge.insert(key, edge);
                }
            }
        }

        for edge in best_edge.values() {
            if let (Some(&src), Some(&tgt)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                g.add_edge(src, tgt, edge.edge_type);
            }
        }

        g
    }
}

// ───────── Tests ─────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn make_cert_service(serial: &str) -> ServiceInfo {
        ServiceInfo {
            port: 443,
            protocol: "tcp".into(),
            service_name: "https".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: None,
            tls_cert_serial: Some(serial.into()),
            ssh_host_key: None,
            cve_ids: Vec::new(),
            max_cvss_score: None,
            cpe: None,
        }
    }

    fn make_ssh_service(key: &str) -> ServiceInfo {
        ServiceInfo {
            port: 22,
            protocol: "tcp".into(),
            service_name: "ssh".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: None,
            tls_cert_serial: None,
            ssh_host_key: Some(key.into()),
            cve_ids: Vec::new(),
            max_cvss_score: None,
            cpe: None,
        }
    }

    fn make_jarm_service(jarm: &str) -> ServiceInfo {
        ServiceInfo {
            port: 443,
            protocol: "tcp".into(),
            service_name: "https".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: Some(jarm.into()),
            tls_cert_serial: None,
            ssh_host_key: None,
            cve_ids: Vec::new(),
            max_cvss_score: None,
            cpe: None,
        }
    }

    #[test]
    fn test_shared_cert_creates_edge() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let svc = make_cert_service("SERIAL-123");
        let defaults = AssetNodeDefaults::default();

        let edges1 = graph.ingest_service(a1.clone(), svc.clone(), defaults.clone());
        assert!(edges1.is_empty());

        let edges2 = graph.ingest_service(a2.clone(), svc, defaults);
        assert_eq!(edges2.len(), 1);
        assert_eq!(edges2[0].edge_type, EdgeType::SharedTlsCert);
        assert_eq!(edges2[0].confidence, 0.95);
    }

    #[test]
    fn test_shared_ssh_key_creates_edge() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let svc = make_ssh_service("ssh-rsa AAAA...");
        let defaults = AssetNodeDefaults::default();

        graph.ingest_service(a1.clone(), svc.clone(), defaults.clone());
        let edges = graph.ingest_service(a2, svc, defaults);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, EdgeType::SharedSshHostKey);
        assert_eq!(edges[0].confidence, 0.98);
    }

    #[test]
    fn test_matching_jarm_creates_edge() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let svc = make_jarm_service("jarm-hash-abc123");
        let defaults = AssetNodeDefaults::default();

        graph.ingest_service(a1.clone(), svc.clone(), defaults.clone());
        let edges = graph.ingest_service(a2, svc, defaults);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, EdgeType::MatchingJarm);
    }

    #[test]
    fn test_shared_dns_creates_edge() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        // Need nodes first
        graph.add_node(AssetNode {
            asset_id: a1.clone(),
            asn: None,
            as_org: None,
            country: None,
            rdomains: Vec::new(),
            services: Vec::new(),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        });
        graph.add_node(AssetNode {
            asset_id: a2.clone(),
            asn: None,
            as_org: None,
            country: None,
            rdomains: Vec::new(),
            services: Vec::new(),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        });

        graph.ingest_dns(a1.clone(), "example.com");
        let edges = graph.ingest_dns(a2, "example.com");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, EdgeType::SharedDnsName);
    }

    #[test]
    fn test_same_bgp_as_creates_edge_on_overlap() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();
        let svc1 = ServiceInfo {
            port: 443,
            protocol: "tcp".into(),
            service_name: "https".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: None,
            tls_cert_serial: None,
            ssh_host_key: None,
            cve_ids: Vec::new(),
            max_cvss_score: None,
            cpe: None,
        };
        let svc2 = ServiceInfo {
            port: 443,
            protocol: "tcp".into(),
            service_name: "https".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: None,
            tls_cert_serial: None,
            ssh_host_key: None,
            cve_ids: Vec::new(),
            max_cvss_score: None,
            cpe: None,
        };

        graph.ingest_service(a1.clone(), svc1, defaults.clone());
        graph.ingest_service(a2.clone(), svc2, defaults);

        // Ingest BGP AS with overlapping services
        let edges = graph.ingest_bgp_as(a1, 15169, "Google LLC", "web");
        // No peer yet for a1
        assert!(edges.is_empty());

        let edges = graph.ingest_bgp_as(a2, 15169, "Google LLC", "web");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, EdgeType::SameBgpAs);
    }

    #[test]
    fn test_corporate_boundary_inference() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        // Manually add 3 different edge types between a1 and a2
        let make_edge = |et: EdgeType, conf: f64| AssetEdge {
            source: a1.clone(),
            target: a2.clone(),
            edge_type: et,
            confidence: conf,
            correlation_value: "test".into(),
            discovered_at: Utc::now(),
        };

        graph.edges.push(make_edge(EdgeType::SharedTlsCert, 0.95));
        graph.edges.push(make_edge(EdgeType::MatchingJarm, 0.80));
        graph.edges.push(make_edge(EdgeType::SharedDnsName, 0.70));

        let boundaries = graph.infer_corporate_boundaries();
        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].edge_type, EdgeType::CorporateBoundary);
    }

    #[test]
    fn test_to_edge_list() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let svc = make_cert_service("SERIAL-456");
        let defaults = AssetNodeDefaults::default();

        graph.ingest_service(a1, svc.clone(), defaults.clone());
        graph.ingest_service(a2, svc, defaults);

        let edge_list = graph.to_edge_list();
        assert_eq!(edge_list.len(), 1);
        assert_eq!(edge_list[0].edge_type, "shared_tls_cert");
    }

    #[test]
    fn test_neighbors() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };
        let a3 = AssetId {
            ip: test_ip("10.0.0.3"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();
        graph.ingest_service(a1.clone(), make_cert_service("S1"), defaults.clone());
        graph.ingest_service(a2.clone(), make_cert_service("S1"), defaults.clone());
        graph.ingest_service(a3, make_cert_service("S2"), defaults);

        let neighbors = graph.neighbors(&a1);
        assert!(neighbors.contains(&a2));
        assert!(!neighbors.contains(&a1));
    }

    #[test]
    fn test_subgraph_from() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };
        let a3 = AssetId {
            ip: test_ip("10.0.0.3"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();
        graph.ingest_service(a1.clone(), make_cert_service("S1"), defaults.clone());
        graph.ingest_service(a2.clone(), make_cert_service("S1"), defaults.clone());
        graph.ingest_service(a3.clone(), make_cert_service("S2"), defaults);

        let (reachable, _edges) = graph.subgraph_from(&a1, 1);
        assert!(reachable.contains(&a1));
        assert!(reachable.contains(&a2));
        // a3 is not connected to a1
        assert!(!reachable.contains(&a3));
    }

    #[test]
    fn test_stats() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let a2 = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();
        graph.ingest_service(a1, make_cert_service("S1"), defaults.clone());
        graph.ingest_service(a2, make_cert_service("S1"), defaults);

        let stats = graph.stats();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 1);
        assert_eq!(stats.edge_type_counts[&EdgeType::SharedTlsCert], 1);
    }

    #[test]
    fn test_asset_id_display() {
        let a = AssetId {
            ip: test_ip("10.0.0.1"),
            port: Some(443),
        };
        assert_eq!(a.to_string(), "10.0.0.1:443");

        let b = AssetId {
            ip: test_ip("10.0.0.2"),
            port: None,
        };
        assert_eq!(b.to_string(), "10.0.0.2");
    }

    #[test]
    fn test_edge_type_display() {
        assert_eq!(EdgeType::SharedTlsCert.to_string(), "shared_tls_cert");
        assert_eq!(
            EdgeType::SharedSshHostKey.to_string(),
            "shared_ssh_host_key"
        );
        assert_eq!(EdgeType::MatchingJarm.to_string(), "matching_jarm");
        assert_eq!(EdgeType::SameBgpAs.to_string(), "same_bgp_as");
        assert_eq!(EdgeType::SharedDnsName.to_string(), "shared_dns_name");
        assert_eq!(
            EdgeType::CorporateBoundary.to_string(),
            "corporate_boundary"
        );
    }

    #[test]
    fn test_to_json() {
        let mut graph = AssetGraph::new();
        let a1 = AssetId {
            ip: test_ip("10.0.0.1"),
            port: None,
        };
        let defaults = AssetNodeDefaults::default();
        graph.ingest_service(a1, make_cert_service("S1"), defaults);

        let json = graph.to_json();
        assert!(json.is_ok());
    }
}
