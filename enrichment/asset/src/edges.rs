//! Edge utilities for the asset graph.
use super::{AssetEdge, AssetGraph, AssetId, EdgeType};
use petgraph::{Directed, Graph};
use serde::Serialize;
use std::collections::HashMap;

/// Convert AssetGraph to a petgraph Graph with full edge metadata (`AssetEdge`).
///
/// This version preserves all edges without deduplication and includes complete
/// edge metadata (confidence, correlation_value, discovered_at). Use this when
/// you need the full edge information for analysis or when edge count matters.
pub fn to_petgraph(graph: &AssetGraph) -> Graph<AssetId, AssetEdge, Directed> {
    let mut pg = Graph::new();
    let mut node_map = HashMap::new();

    // Add nodes
    for asset_id in graph.nodes.keys() {
        let idx = pg.add_node(asset_id.clone());
        node_map.insert(asset_id.clone(), idx);
    }

    // Add edges
    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (node_map.get(&edge.source), node_map.get(&edge.target)) {
            pg.add_edge(src, tgt, edge.clone());
        }
    }

    pg
}

/// Filter edges by type.
pub fn filter_edges_by_type(
    graph: &AssetGraph,
    edge_type: EdgeType,
) -> impl Iterator<Item = &AssetEdge> + '_ {
    graph.edges.iter().filter(move |e| e.edge_type == edge_type)
}

/// Get all CVE severity edges.
pub fn get_cve_edges(graph: &AssetGraph) -> Vec<&AssetEdge> {
    filter_edges_by_type(graph, EdgeType::CveSeverity).collect()
}

/// Get all assets connected via CVE edges.
pub fn get_cve_connected_assets(graph: &AssetGraph) -> std::collections::HashSet<&AssetId> {
    let mut assets = std::collections::HashSet::new();
    for edge in get_cve_edges(graph) {
        assets.insert(&edge.source);
        assets.insert(&edge.target);
    }
    assets
}

/// Find assets with critical CVEs (CVSS >= 9.0).
pub fn find_critical_cve_assets(graph: &AssetGraph) -> Vec<&AssetId> {
    graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::CveSeverity)
        .filter(|e| {
            // Extract CVSS from correlation_value format "CVE-XXXX-XXXX:CVSS=X.X"
            e.correlation_value
                .split("CVSS=")
                .nth(1)
                .and_then(|s| s.parse::<f64>().ok())
                .is_some_and(|v| v >= 9.0)
        })
        .flat_map(|e| [&e.source, &e.target])
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Find the most connected assets by CVE edges.
pub fn top_cve_connected_assets(graph: &AssetGraph, limit: usize) -> Vec<(&AssetId, usize)> {
    let mut counts: HashMap<&AssetId, usize> = HashMap::new();
    for edge in get_cve_edges(graph) {
        *counts.entry(&edge.source).or_default() += 1;
        *counts.entry(&edge.target).or_default() += 1;
    }
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    v.into_iter().take(limit).collect()
}

/// CVE severity distribution summary.
#[derive(Debug, Clone, Serialize)]
pub struct CveSeveritySummary {
    pub critical: usize, // CVSS >= 9.0
    pub high: usize,     // CVSS 7.0-8.9
    pub medium: usize,   // CVSS 4.0-6.9
    pub low: usize,      // CVSS 0.1-3.9
    pub total: usize,
}

impl CveSeveritySummary {
    pub fn from_graph(graph: &AssetGraph) -> Self {
        let mut summary = Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            total: 0,
        };

        for edge in get_cve_edges(graph) {
            // Parse CVSS from correlation_value if available
            let cvss = edge
                .correlation_value
                .split("CVSS=")
                .nth(1)
                .and_then(|s| s.parse::<f64>().ok());

            if let Some(score) = cvss {
                summary.total += 1;
                if score >= 9.0 {
                    summary.critical += 1;
                } else if score >= 7.0 {
                    summary.high += 1;
                } else if score >= 4.0 {
                    summary.medium += 1;
                } else {
                    summary.low += 1;
                }
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetGraph, AssetId, AssetNodeDefaults, ServiceInfo};
    use std::net::IpAddr;

    fn test_ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn make_service_with_cves(cves: Vec<String>, max_cvss: Option<f64>) -> ServiceInfo {
        ServiceInfo {
            port: 80,
            protocol: "tcp".into(),
            service_name: "http".into(),
            banner_hash: None,
            ja4_fingerprint: None,
            jarm_fingerprint: None,
            tls_cert_serial: None,
            ssh_host_key: None,
            cve_ids: cves,
            max_cvss_score: max_cvss,
            cpe: Some("cpe:2.3:a:apache:http_server:2.4.41".into()),
        }
    }

    #[test]
    fn test_cve_severity_edges() {
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

        // Asset 1 has CVE-2021-44228 (Log4Shell, CVSS 10.0)
        let svc1 = make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0));
        graph.ingest_service(a1.clone(), svc1, defaults.clone());
        graph.ingest_cves(a1.clone(), &["CVE-2021-44228".into()], Some(10.0));

        // Asset 2 shares the same CVE
        let svc2 = make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0));
        graph.ingest_service(a2.clone(), svc2, defaults.clone());
        graph.ingest_cves(a2.clone(), &["CVE-2021-44228".into()], Some(10.0));

        // Asset 3 has different CVE
        let svc3 = make_service_with_cves(vec!["CVE-2020-1938".into()], Some(7.5));
        graph.ingest_service(a3.clone(), svc3, defaults);
        graph.ingest_cves(a3, &["CVE-2020-1938".into()], Some(7.5));

        // Check CVE edges exist
        let cve_edges = get_cve_edges(&graph);
        assert_eq!(cve_edges.len(), 1); // Only a1-a2 share CVE-2021-44228

        let edge = &cve_edges[0];
        assert_eq!(edge.edge_type, EdgeType::CveSeverity);
        assert!(edge.correlation_value.starts_with("CVE-2021-44228"));
        assert!(edge.correlation_value.contains("CVSS=10.0"));
        assert!(edge.confidence > 0.9); // High confidence for critical CVE

        // Check CVE connected assets
        let connected = get_cve_connected_assets(&graph);
        assert_eq!(connected.len(), 2); // a1 and a2
    }

    #[test]
    fn test_cve_severity_summary() {
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
        let a4 = AssetId {
            ip: test_ip("10.0.0.4"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();

        // Critical CVE (10.0)
        graph.ingest_service(
            a1.clone(),
            make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0)),
            defaults.clone(),
        );
        graph.ingest_cves(a1.clone(), &["CVE-2021-44228".into()], Some(10.0));
        graph.ingest_service(
            a2.clone(),
            make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0)),
            defaults.clone(),
        );
        graph.ingest_cves(a2, &["CVE-2021-44228".into()], Some(10.0));

        // High CVE (7.5)
        graph.ingest_service(
            a3.clone(),
            make_service_with_cves(vec!["CVE-2020-1938".into()], Some(7.5)),
            defaults.clone(),
        );
        graph.ingest_cves(a3.clone(), &["CVE-2020-1938".into()], Some(7.5));
        graph.ingest_service(
            a4.clone(),
            make_service_with_cves(vec!["CVE-2020-1938".into()], Some(7.5)),
            defaults,
        );
        graph.ingest_cves(a4, &["CVE-2020-1938".into()], Some(7.5));

        let summary = CveSeveritySummary::from_graph(&graph);
        assert_eq!(summary.total, 2); // Two unique CVE pairs
        assert_eq!(summary.critical, 1); // CVE-2021-44228
        assert_eq!(summary.high, 1); // CVE-2020-1938
    }

    #[test]
    fn test_find_critical_cve_assets() {
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

        graph.ingest_service(
            a1.clone(),
            make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0)),
            defaults.clone(),
        );
        graph.ingest_cves(a1.clone(), &["CVE-2021-44228".into()], Some(10.0));
        graph.ingest_service(
            a2.clone(),
            make_service_with_cves(vec!["CVE-2021-44228".into()], Some(10.0)),
            defaults,
        );
        graph.ingest_cves(a2, &["CVE-2021-44228".into()], Some(10.0));

        let critical = find_critical_cve_assets(&graph);
        assert_eq!(critical.len(), 2);
    }

    #[test]
    fn test_top_cve_connected_assets() {
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
        let a4 = AssetId {
            ip: test_ip("10.0.0.4"),
            port: None,
        };

        let defaults = AssetNodeDefaults::default();
        let assets = [a1.clone(), a2.clone(), a3.clone(), a4.clone()];

        // Each asset has a unique CVE initially (no edges)
        for (i, asset) in assets.iter().enumerate() {
            let svc = make_service_with_cves(vec![format!("CVE-2021-{}", 44228 + i)], Some(10.0));
            graph.ingest_service(asset.clone(), svc, defaults.clone());
            graph.ingest_cves(
                asset.clone(),
                &[format!("CVE-2021-{}", 44228 + i)],
                Some(10.0),
            );
        }

        // Now create shared CVEs: a1 shares with a2, a3, a4
        graph.ingest_cves(a1.clone(), &["CVE-2021-99999".into()], Some(10.0));
        graph.ingest_cves(a2.clone(), &["CVE-2021-99999".into()], Some(10.0));

        graph.ingest_cves(a1.clone(), &["CVE-2021-99998".into()], Some(10.0));
        graph.ingest_cves(a3.clone(), &["CVE-2021-99998".into()], Some(10.0));

        graph.ingest_cves(a1.clone(), &["CVE-2021-99997".into()], Some(10.0));
        graph.ingest_cves(a4.clone(), &["CVE-2021-99997".into()], Some(10.0));

        let top = top_cve_connected_assets(&graph, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].1, 3); // a1 has 3 connections (to a2, a3, a4)
    }
}
