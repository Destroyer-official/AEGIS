use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{CrtShEntry, CtCertificate, PassiveDnsRecord};

#[derive(Debug, Error)]
pub enum CtMonitorError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("monitor shutdown: {0}")]
    Shutdown(String),
}

// ─── CT Monitor ScanCommand ─────────────────────────────────────────────────

/// A scan command emitted by the CT monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtScanCommand {
    pub command_id: u64,
    pub target_ip: Option<[u8; 4]>,
    pub domain: String,
    pub port: u16,
    pub source: String,
    pub priority: f64,
    pub generated_at: String,
    pub cert_issuer: String,
    pub cert_sans: Vec<String>,
}

// ─── CT Monitor Configuration ───────────────────────────────────────────────

/// Configuration for the CT log incremental monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtMonitorConfig {
    /// Domains to monitor for new certificates.
    pub watch_domains: Vec<String>,
    /// Poll interval in seconds (default: 30, target <60s end-to-end).
    pub poll_interval_secs: u64,
    /// Maximum concurrent HTTP requests.
    pub max_concurrent_requests: usize,
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Deduplication window in seconds (same cert not re-emitted within).
    pub dedup_window_secs: u64,
    /// Batch size for DNS resolution.
    pub dns_batch_size: usize,
    /// crt.sh base URL.
    pub crt_sh_url: String,
    /// User agent for crt.sh requests.
    pub user_agent: String,
}

impl Default for CtMonitorConfig {
    fn default() -> Self {
        Self {
            watch_domains: Vec::new(),
            poll_interval_secs: 30,
            max_concurrent_requests: 4,
            request_timeout_secs: 15,
            dedup_window_secs: 3600,
            dns_batch_size: 100,
            crt_sh_url: "https://crt.sh".to_string(),
            user_agent: "AEGIS-CT-Monitor/1.0 (+https://aegis.example.com/robots)".to_string(),
        }
    }
}

// ─── Certificate Seen Record ────────────────────────────────────────────────

/// Tracks when a certificate was first seen to avoid re-emitting.
#[derive(Debug, Clone)]
struct CertSeenRecord {
    first_seen: Instant,
}

// ─── Domain Discovery Record ────────────────────────────────────────────────

/// A domain discovered via CT log monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDiscovery {
    pub domain: String,
    pub discovered_via: String,
    pub discovered_at: String,
    pub issuer_ca: String,
    pub common_name: String,
    pub san_count: usize,
}

// ─── CT Log Incremental Monitor ─────────────────────────────────────────────

/// Real-time CT log monitor that polls crt.sh for new certificates
/// and emits scan commands within <60s latency.
pub struct CtLogMonitor {
    config: CtMonitorConfig,
    client: reqwest::Client,
    /// Certificate deduplication: cert_fingerprint → SeenRecord.
    seen_certs: HashMap<String, CertSeenRecord>,
    /// All domains discovered through CT monitoring.
    discovered_domains: Vec<DomainDiscovery>,
    /// Known domains (prevents re-discovery of existing domains).
    known_domains: HashSet<String>,
    /// Command ID counter.
    command_counter: u64,
    /// Start time for latency tracking.
    started_at: Instant,
}

impl CtLogMonitor {
    pub fn new(config: CtMonitorConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build HTTP client");

        let known_domains: HashSet<String> = config.watch_domains.iter().cloned().collect();

        Self {
            config,
            client,
            seen_certs: HashMap::new(),
            discovered_domains: Vec::new(),
            known_domains,
            command_counter: 0,
            started_at: Instant::now(),
        }
    }

    /// Add a domain to the watch list.
    pub fn watch_domain(&mut self, domain: String) {
        self.known_domains.insert(domain.clone());
        self.config.watch_domains.push(domain);
    }

    /// Add multiple domains to the watch list.
    pub fn watch_domains(&mut self, domains: Vec<String>) {
        for d in domains {
            self.watch_domain(d);
        }
    }

    /// Poll crt.sh once for all watched domains and return new scan commands.
    ///
    /// This is the core polling loop: it queries crt.sh for each domain,
    /// deduplicates certificates, and emits commands for new discoveries.
    pub async fn poll_once(&mut self) -> Result<Vec<CtScanCommand>, CtMonitorError> {
        let mut all_commands = Vec::new();
        let now = now_iso8601();

        // Poll each domain (sequential to respect rate limits).
        for domain in &self.config.watch_domains.clone() {
            let entries = self.query_crt_sh(domain).await?;

            for entry in entries {
                // Dedup: skip if we've already seen this cert recently.
                let cert_key = format!("{}:{}", entry.common_name, entry.issuer_ca);

                if let Some(seen) = self.seen_certs.get(&cert_key)
                    && seen.first_seen.elapsed()
                        < Duration::from_secs(self.config.dedup_window_secs)
                {
                    continue;
                }

                self.command_counter += 1;
                let cmd = CtScanCommand {
                    command_id: self.command_counter,
                    target_ip: None,
                    domain: entry.common_name.clone(),
                    port: 443,
                    source: "passive_ct".to_string(),
                    priority: self.calculate_cert_priority(&entry),
                    generated_at: now.clone(),
                    cert_issuer: entry.issuer_ca.clone(),
                    cert_sans: entry
                        .name_value
                        .split('\n')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                };

                self.seen_certs.insert(
                    cert_key,
                    CertSeenRecord {
                        first_seen: Instant::now(),
                    },
                );

                // Track new domains discovered via SANs.
                self.track_discovered_domains(&entry, domain);

                all_commands.push(cmd);
            }
        }

        Ok(all_commands)
    }

    /// Run the monitor in a loop, calling `on_commands` for each batch.
    ///
    /// This is the main event loop for the CT monitor. It polls crt.sh
    /// at the configured interval and invokes the callback with new commands.
    pub async fn run<F>(&mut self, mut on_commands: F) -> Result<(), CtMonitorError>
    where
        F: FnMut(Vec<CtScanCommand>),
    {
        self.started_at = Instant::now();

        loop {
            let start = Instant::now();
            let commands = self.poll_once().await?;

            if !commands.is_empty() {
                on_commands(commands);
            }

            // Sleep for the remaining interval.
            let elapsed = start.elapsed();
            let interval = Duration::from_secs(self.config.poll_interval_secs);
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }
        }
    }

    /// Resolve domains to IPs via DNS (stub implementation).
    ///
    /// In production, this would use a real DNS resolver. For now, it
    /// returns None for target_ip, letting downstream resolve.
    pub fn resolve_domains(&self, domains: &[String]) -> Vec<(String, Option<[u8; 4]>)> {
        domains
            .iter()
            .map(|d| {
                let ip = d.parse::<Ipv4Addr>().ok().map(|addr| addr.octets());
                (d.clone(), ip)
            })
            .collect()
    }

    /// Generate DNS records from discovered domains for passive DNS ingestion.
    pub fn discovered_dns_records(&self) -> Vec<PassiveDnsRecord> {
        self.discovered_domains
            .iter()
            .map(|d| PassiveDnsRecord {
                domain: d.domain.clone(),
                ip_address: String::new(),
                record_type: "A".to_string(),
                first_seen: d.discovered_at.clone(),
                last_seen: d.discovered_at.clone(),
                ttl: None,
            })
            .collect()
    }

    /// All domains discovered so far.
    pub fn discovered_domains(&self) -> &[DomainDiscovery] {
        &self.discovered_domains
    }

    /// Total certificates seen.
    pub fn total_certs_seen(&self) -> usize {
        self.seen_certs.len()
    }

    /// Monitor uptime in seconds.
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Clean up expired dedup entries.
    pub fn cleanup_dedup(&mut self) {
        let window = Duration::from_secs(self.config.dedup_window_secs);
        self.seen_certs
            .retain(|_, record| record.first_seen.elapsed() < window);
    }

    // ─── Private helpers ──────────────────────────────────────────────

    async fn query_crt_sh(&self, domain: &str) -> Result<Vec<CtCertificate>, CtMonitorError> {
        let pattern = format!("%.{}", domain);
        let url = format!("{}/?q={}&output=json", self.config.crt_sh_url, pattern);

        let resp = self.client.get(&url).send().await?;
        let entries: Vec<CrtShEntry> = resp.json().await?;

        Ok(entries
            .into_iter()
            .filter_map(|e| {
                let issuer = e.issuer_name.unwrap_or_default();
                let cn = e.common_name.unwrap_or_default();
                let sans = e.name_value.unwrap_or_default();
                if cn.is_empty() && sans.is_empty() {
                    None
                } else {
                    Some(CtCertificate {
                        issuer_ca: issuer,
                        common_name: cn,
                        name_value: sans,
                    })
                }
            })
            .collect())
    }

    fn calculate_cert_priority(&self, cert: &CtCertificate) -> f64 {
        let mut priority: f64 = 0.5;

        // Boost priority for well-known CAs.
        let issuer_lower = cert.issuer_ca.to_lowercase();
        if issuer_lower.contains("letsencrypt") || issuer_lower.contains("let's encrypt") {
            priority += 0.1;
        }
        if issuer_lower.contains("digicert") || issuer_lower.contains("comodo") {
            priority += 0.15;
        }

        // Boost for wildcard certs (indicates infrastructure).
        if cert.common_name.starts_with("*.") {
            priority += 0.2;
        }

        // Boost for many SANs (indicates CDN/load balancer).
        let san_count = cert.name_value.lines().count();
        if san_count > 10 {
            priority += 0.1;
        }

        priority.min(1.0)
    }

    fn track_discovered_domains(&mut self, cert: &CtCertificate, source_domain: &str) {
        // Extract domains from SANs.
        for san in cert.name_value.lines() {
            let san = san.trim().to_string();
            if san.is_empty() || self.known_domains.contains(&san) {
                continue;
            }

            // Skip IPs and wildcards in discovery.
            if san.parse::<Ipv4Addr>().is_ok() {
                continue;
            }

            self.known_domains.insert(san.clone());
            self.discovered_domains.push(DomainDiscovery {
                domain: san,
                discovered_via: source_domain.to_string(),
                discovered_at: now_iso8601(),
                issuer_ca: cert.issuer_ca.clone(),
                common_name: cert.common_name.clone(),
                san_count: cert.name_value.lines().count(),
            });
        }
    }
}

// ─── Utility ────────────────────────────────────────────────────────────────

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut remaining = secs;
    let mut y = 1970u32;
    loop {
        let days_in_year =
            if (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400) {
                366
            } else {
                365
            };
        let secs_in_year = days_in_year * 86400;
        if remaining < secs_in_year {
            break;
        }
        remaining -= secs_in_year;
        y += 1;
    }
    let day_of_year = (remaining / 86400) as u32;
    let is_leap = (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400);
    let month_days = [
        31u32,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0u32;
    let mut d = day_of_year;
    while m < 12 && d >= month_days[m as usize] {
        d -= month_days[m as usize];
        m += 1;
    }
    let secs_in_day = remaining % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;
    let seconds = secs_in_day % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d + 1,
        hours,
        minutes,
        seconds
    )
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_monitor_config_defaults() {
        let config = CtMonitorConfig::default();
        assert_eq!(config.poll_interval_secs, 30);
        assert_eq!(config.max_concurrent_requests, 4);
        assert_eq!(config.dedup_window_secs, 3600);
        assert!(config.watch_domains.is_empty());
    }

    #[test]
    fn test_ct_monitor_watch_domain() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());
        monitor.watch_domain("example.com".to_string());
        assert!(monitor.known_domains.contains("example.com"));
        assert!(
            monitor
                .config
                .watch_domains
                .contains(&"example.com".to_string())
        );
    }

    #[test]
    fn test_ct_monitor_watch_domains_batch() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());
        monitor.watch_domains(vec![
            "a.com".to_string(),
            "b.com".to_string(),
            "c.com".to_string(),
        ]);
        assert!(monitor.known_domains.contains("a.com"));
        assert!(monitor.known_domains.contains("b.com"));
        assert!(monitor.known_domains.contains("c.com"));
    }

    #[test]
    fn test_calculate_cert_priority() {
        let monitor = CtLogMonitor::new(CtMonitorConfig::default());

        // Basic cert.
        let cert = CtCertificate {
            issuer_ca: "Unknown CA".to_string(),
            common_name: "example.com".to_string(),
            name_value: "example.com".to_string(),
        };
        let p1 = monitor.calculate_cert_priority(&cert);
        assert!((0.5..=1.0).contains(&p1));

        // Let's Encrypt cert.
        let cert_letsencrypt = CtCertificate {
            issuer_ca: "Let's Encrypt Authority X3".to_string(),
            common_name: "example.com".to_string(),
            name_value: "example.com".to_string(),
        };
        let p2 = monitor.calculate_cert_priority(&cert_letsencrypt);
        assert!(p2 > p1);

        // Wildcard cert.
        let cert_wildcard = CtCertificate {
            issuer_ca: "DigiCert SHA2".to_string(),
            common_name: "*.example.com".to_string(),
            name_value: "*.example.com\nexample.com".to_string(),
        };
        let p3 = monitor.calculate_cert_priority(&cert_wildcard);
        assert!(p3 >= p2);
    }

    #[test]
    fn test_track_discovered_domains() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());

        let cert = CtCertificate {
            issuer_ca: "Let's Encrypt".to_string(),
            common_name: "*.newdomain.com".to_string(),
            name_value: "*.newdomain.com\napi.newdomain.com\nadmin.newdomain.com".to_string(),
        };

        monitor.track_discovered_domains(&cert, "example.com");

        assert!(monitor.known_domains.contains("*.newdomain.com"));
        assert!(monitor.known_domains.contains("api.newdomain.com"));
        assert!(monitor.known_domains.contains("admin.newdomain.com"));
        assert_eq!(monitor.discovered_domains.len(), 3);
    }

    #[test]
    fn test_track_discovered_domains_skips_known() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig {
            watch_domains: vec!["existing.com".to_string()],
            ..Default::default()
        });

        let cert = CtCertificate {
            issuer_ca: "Let's Encrypt".to_string(),
            common_name: "existing.com".to_string(),
            name_value: "existing.com\nnew.com".to_string(),
        };

        monitor.track_discovered_domains(&cert, "source.com");

        // existing.com is known, only new.com discovered
        assert_eq!(monitor.discovered_domains.len(), 1);
        assert_eq!(monitor.discovered_domains[0].domain, "new.com");
    }

    #[test]
    fn test_track_discovered_domains_skips_ips() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());

        let cert = CtCertificate {
            issuer_ca: "Let's Encrypt".to_string(),
            common_name: "ip-cert.com".to_string(),
            name_value: "ip-cert.com\n192.168.1.1".to_string(),
        };

        monitor.track_discovered_domains(&cert, "source.com");

        // IP address should be skipped
        assert_eq!(monitor.discovered_domains.len(), 1);
        assert_eq!(monitor.discovered_domains[0].domain, "ip-cert.com");
    }

    #[test]
    fn test_resolve_domains() {
        let monitor = CtLogMonitor::new(CtMonitorConfig::default());
        let results =
            monitor.resolve_domains(&["example.com".to_string(), "192.168.1.1".to_string()]);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "example.com");
        assert!(results[0].1.is_none());
        assert_eq!(results[1].0, "192.168.1.1");
        assert_eq!(results[1].1, Some([192, 168, 1, 1]));
    }

    #[test]
    fn test_cleanup_dedup() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig {
            dedup_window_secs: 0, // Immediate expiry
            ..Default::default()
        });

        monitor.seen_certs.insert(
            "test.com:Issuer".to_string(),
            CertSeenRecord {
                first_seen: Instant::now(),
            },
        );

        assert_eq!(monitor.seen_certs.len(), 1);
        monitor.cleanup_dedup();
        assert_eq!(monitor.seen_certs.len(), 0);
    }

    #[test]
    fn test_discovered_dns_records() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());

        monitor.discovered_domains.push(DomainDiscovery {
            domain: "new.com".to_string(),
            discovered_via: "example.com".to_string(),
            discovered_at: "2026-09-01T00:00:00Z".to_string(),
            issuer_ca: "Let's Encrypt".to_string(),
            common_name: "new.com".to_string(),
            san_count: 1,
        });

        let records = monitor.discovered_dns_records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].domain, "new.com");
    }

    #[test]
    fn test_monitor_statistics() {
        let mut monitor = CtLogMonitor::new(CtMonitorConfig::default());

        monitor.seen_certs.insert(
            "a.com:Issuer1".to_string(),
            CertSeenRecord {
                first_seen: Instant::now(),
            },
        );
        monitor.seen_certs.insert(
            "b.com:Issuer2".to_string(),
            CertSeenRecord {
                first_seen: Instant::now(),
            },
        );

        assert_eq!(monitor.total_certs_seen(), 2);
        assert!(monitor.uptime_secs() < 10);
    }
}
