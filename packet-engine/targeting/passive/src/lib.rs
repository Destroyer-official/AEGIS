use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod ct_monitor;
pub mod cve_feed;
pub mod quality;
pub mod replay;

#[derive(Debug, Error)]
pub enum PassiveError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("BGP dump format error: {0}")]
    BgpFormat(String),
    #[error("CT log query failed: {0}")]
    CtLogError(String),
}

// ─── Certificate Transparency ────────────────────────────────────

/// A certificate observed in CT logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CtCertificate {
    /// The issuer CA name.
    pub issuer_ca: String,
    /// Common Name (CN) from the certificate subject.
    pub common_name: String,
    /// All Subject Alternative Names (SANs).
    pub name_value: String,
}

/// Response from crt.sh CT log API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CrtShEntry {
    pub issuer_ca_id: Option<u64>,
    pub issuer_name: Option<String>,
    pub common_name: Option<String>,
    pub name_value: Option<String>,
    pub id: Option<u64>,
    pub entry_timestamp: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
}

/// CT log fetcher querying crt.sh for certificate transparency data.
pub struct CtLogFetcher {
    client: reqwest::Client,
    base_url: String,
}

impl CtLogFetcher {
    /// Create a new CT log fetcher with the default crt.sh endpoint.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("AEGIS-Scanner/1.0 (+https://aegis.example.com/robots)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build HTTP client"),
            base_url: "https://crt.sh".to_string(),
        }
    }

    /// Create a CT log fetcher with a custom base URL.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("AEGIS-Scanner/1.0 (+https://aegis.example.com/robots)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build HTTP client"),
            base_url: base_url.into(),
        }
    }

    /// Query crt.sh for certificates matching a domain name.
    ///
    /// Returns all certificate entries where the domain appears in
    /// the Subject Alternative Names (SANs).
    pub async fn query_domain(&self, domain: &str) -> Result<Vec<CtCertificate>, PassiveError> {
        let url = format!("{}/?q={}&output=json", self.base_url, domain);
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

    /// Query crt.sh for certificates matching a wildcard pattern.
    pub async fn query_wildcard(&self, domain: &str) -> Result<Vec<CtCertificate>, PassiveError> {
        let pattern = format!("%.{}", domain);
        self.query_domain(&pattern).await
    }
}

impl Default for CtLogFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a CT log CSV line (for offline/testing usage).
pub fn parse_ct_csv(reader: impl std::io::Read) -> Result<Vec<CtCertificate>, PassiveError> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut certs = Vec::new();

    for result in rdr.deserialize() {
        let entry: CrtShEntry = result?;
        let issuer = entry.issuer_name.unwrap_or_default();
        let cn = entry.common_name.unwrap_or_default();
        let sans = entry.name_value.unwrap_or_default();

        if !cn.is_empty() || !sans.is_empty() {
            certs.push(CtCertificate {
                issuer_ca: issuer,
                common_name: cn,
                name_value: sans,
            });
        }
    }

    Ok(certs)
}

// ─── Passive DNS ─────────────────────────────────────────────────

/// A passive DNS resolution record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassiveDnsRecord {
    /// The domain name that was resolved.
    pub domain: String,
    /// The IP address it resolved to.
    pub ip_address: String,
    /// Record type (A, AAAA, CNAME, MX, etc.).
    pub record_type: String,
    /// First time this resolution was observed.
    pub first_seen: String,
    /// Last time this resolution was observed.
    pub last_seen: String,
    /// TTL value if available.
    pub ttl: Option<u32>,
}

/// Passive DNS aggregator collecting domain-to-IP mappings.
pub struct PassiveDnsAggregator {
    /// Map from IP address to list of domains that have resolved to it.
    ip_to_domains: HashMap<String, Vec<String>>,
    /// Map from domain to list of IPs it has resolved to.
    domain_to_ips: HashMap<String, Vec<String>>,
    /// Raw records indexed by (domain, ip).
    records: HashMap<(String, String), PassiveDnsRecord>,
}

impl PassiveDnsAggregator {
    pub fn new() -> Self {
        Self {
            ip_to_domains: HashMap::new(),
            domain_to_ips: HashMap::new(),
            records: HashMap::new(),
        }
    }

    /// Ingest a passive DNS record.
    pub fn ingest(&mut self, record: PassiveDnsRecord) {
        let key = (record.domain.clone(), record.ip_address.clone());

        self.ip_to_domains
            .entry(record.ip_address.clone())
            .or_default()
            .push(record.domain.clone());

        self.domain_to_ips
            .entry(record.domain.clone())
            .or_default()
            .push(record.ip_address.clone());

        self.records.insert(key, record);
    }

    /// Ingest a batch of passive DNS records.
    pub fn ingest_batch(&mut self, records: Vec<PassiveDnsRecord>) {
        for record in records {
            self.ingest(record);
        }
    }

    /// Look up all domains that have historically resolved to an IP.
    pub fn domains_for_ip(&self, ip: &str) -> &[String] {
        self.ip_to_domains
            .get(ip)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Look up all IPs a domain has resolved to.
    pub fn ips_for_domain(&self, domain: &str) -> &[String] {
        self.domain_to_ips
            .get(domain)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the raw record for a specific (domain, ip) pair.
    pub fn get_record(&self, domain: &str, ip: &str) -> Option<&PassiveDnsRecord> {
        self.records.get(&(domain.to_string(), ip.to_string()))
    }

    /// Iterate over all (domain, ips) pairs.
    pub fn domain_to_ips_iter(&self) -> impl Iterator<Item = (&str, &Vec<String>)> {
        self.domain_to_ips.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Total records ingested.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Total unique IPs observed.
    pub fn unique_ip_count(&self) -> usize {
        self.ip_to_domains.len()
    }

    /// Total unique domains observed.
    pub fn unique_domain_count(&self) -> usize {
        self.domain_to_ips.len()
    }
}

impl Default for PassiveDnsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── BGP Dump Parser ─────────────────────────────────────────────

/// A BGP prefix announcement extracted from a routing table dump.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BgpPrefix {
    /// The IP prefix (CIDR notation, e.g., "192.168.0.0/16").
    pub prefix: String,
    /// The Autonomous System Number that announced this prefix.
    pub asn: u32,
    /// AS path (list of ASNs from origin to collector).
    pub as_path: Vec<u32>,
    /// Origin AS (the AS that originated the route).
    pub origin_asn: u32,
    /// Whether the prefix has a valid Route Origin Authorization.
    pub roa_valid: Option<bool>,
}

/// BGP dump parser supporting MRT/RIB format and simplified CSV.
pub struct BgpDumpParser;

impl BgpDumpParser {
    /// Parse a simplified BGP dump CSV with columns:
    /// prefix,asn,as_path,origin_asn
    ///
    /// Example line: `192.168.0.0/16,15169,15169 3356 1299,15169`
    pub fn parse_csv(reader: impl std::io::Read) -> Result<Vec<BgpPrefix>, PassiveError> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut prefixes = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if record.len() < 4 {
                continue;
            }

            let prefix = record[0].trim().to_string();
            let asn: u32 = record[1]
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| {
                    PassiveError::BgpFormat(format!("invalid ASN '{}': {}", record[1].trim(), e))
                })?;
            let as_path: Vec<u32> = record[2]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            let origin_asn: u32 =
                record[3]
                    .trim()
                    .parse()
                    .map_err(|e: std::num::ParseIntError| {
                        PassiveError::BgpFormat(format!(
                            "invalid origin ASN '{}': {}",
                            record[3].trim(),
                            e
                        ))
                    })?;

            prefixes.push(BgpPrefix {
                prefix,
                asn,
                as_path,
                origin_asn,
                roa_valid: None,
            });
        }

        Ok(prefixes)
    }

    /// Parse a simplified BGP dump from a string (for testing).
    pub fn parse_csv_str(input: &str) -> Result<Vec<BgpPrefix>, PassiveError> {
        Self::parse_csv(input.as_bytes())
    }

    /// Extract all IPv6 /48 or shorter prefixes from a BGP dump,
    /// suitable as seed subnets for 6Hit/6SENSE targeting.
    pub fn extract_ipv6_seed_prefixes(prefixes: &[BgpPrefix], min_len: u8) -> Vec<&BgpPrefix> {
        prefixes
            .iter()
            .filter(|p| {
                // Check if it's an IPv6 prefix
                p.prefix.contains(':')
                    && p.prefix
                        .split('/')
                        .nth(1)
                        .and_then(|l| l.parse::<u8>().ok())
                        .map(|len| len <= min_len)
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Extract prefixes belonging to a specific ASN.
    pub fn prefixes_for_asn(prefixes: &[BgpPrefix], asn: u32) -> Vec<&BgpPrefix> {
        prefixes
            .iter()
            .filter(|p| p.asn == asn || p.origin_asn == asn)
            .collect()
    }
}

// ─── Passive Intel Aggregator (unified) ──────────────────────────

/// Unified passive intelligence aggregator combining CT, DNS, and BGP.
pub struct PassiveAggregator {
    pub ct: CtLogFetcher,
    pub dns: PassiveDnsAggregator,
    pub bgp_prefixes: Vec<BgpPrefix>,
}

impl PassiveAggregator {
    pub fn new() -> Self {
        Self {
            ct: CtLogFetcher::new(),
            dns: PassiveDnsAggregator::new(),
            bgp_prefixes: Vec::new(),
        }
    }

    /// Load BGP prefixes from a CSV dump.
    pub fn load_bgp_dump(&mut self, reader: impl std::io::Read) -> Result<(), PassiveError> {
        self.bgp_prefixes = BgpDumpParser::parse_csv(reader)?;
        Ok(())
    }

    /// Get all unique ASNs observed across BGP prefixes.
    pub fn unique_asns(&self) -> Vec<u32> {
        let mut asns: Vec<u32> = self.bgp_prefixes.iter().map(|p| p.asn).collect();
        asns.sort();
        asns.dedup();
        asns
    }

    /// Count total passive intelligence records across all sources.
    pub fn total_records(&self) -> usize {
        self.dns.record_count() + self.bgp_prefixes.len()
    }
}

impl Default for PassiveAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CT Log Tests ──

    #[test]
    fn test_parse_ct_csv() {
        let csv_data = "issuer_ca_id,issuer_name,common_name,name_value\n\
            12345,\"Let's Encrypt\",example.com,\"example.com\nwww.example.com\"\n\
            67890,\"DigiCert\",test.example.com,\"test.example.com\"\n";

        let certs = parse_ct_csv(csv_data.as_bytes()).unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0].common_name, "example.com");
        assert!(certs[0].name_value.contains("example.com"));
    }

    // ── Passive DNS Tests ──

    #[test]
    fn test_passive_dns_ingest() {
        let mut agg = PassiveDnsAggregator::new();

        agg.ingest(PassiveDnsRecord {
            domain: "example.com".to_string(),
            ip_address: "93.184.216.34".to_string(),
            record_type: "A".to_string(),
            first_seen: "2024-01-01T00:00:00Z".to_string(),
            last_seen: "2026-08-29T00:00:00Z".to_string(),
            ttl: Some(3600),
        });

        agg.ingest(PassiveDnsRecord {
            domain: "www.example.com".to_string(),
            ip_address: "93.184.216.34".to_string(),
            record_type: "A".to_string(),
            first_seen: "2024-01-01T00:00:00Z".to_string(),
            last_seen: "2026-08-29T00:00:00Z".to_string(),
            ttl: Some(3600),
        });

        assert_eq!(agg.unique_ip_count(), 1);
        assert_eq!(agg.unique_domain_count(), 2);

        let domains = agg.domains_for_ip("93.184.216.34");
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"example.com".to_string()));
        assert!(domains.contains(&"www.example.com".to_string()));
    }

    #[test]
    fn test_passive_dns_bidirectional() {
        let mut agg = PassiveDnsAggregator::new();

        agg.ingest(PassiveDnsRecord {
            domain: "a.com".to_string(),
            ip_address: "1.2.3.4".to_string(),
            record_type: "A".to_string(),
            first_seen: "2024-01-01T00:00:00Z".to_string(),
            last_seen: "2026-08-29T00:00:00Z".to_string(),
            ttl: None,
        });

        agg.ingest(PassiveDnsRecord {
            domain: "b.com".to_string(),
            ip_address: "1.2.3.4".to_string(),
            record_type: "A".to_string(),
            first_seen: "2024-01-01T00:00:00Z".to_string(),
            last_seen: "2026-08-29T00:00:00Z".to_string(),
            ttl: None,
        });

        let ips = agg.ips_for_domain("a.com");
        assert_eq!(ips, &["1.2.3.4".to_string()]);
    }

    // ── BGP Dump Tests ──

    #[test]
    fn test_parse_bgp_csv() {
        let csv_data = "prefix,asn,as_path,origin_asn\n\
            192.168.0.0/16,15169,15169 3356 1299,15169\n\
            10.0.0.0/8,16509,16509 1299,16509\n\
            2001:db8::/32,6939,6939 1299,6939\n";

        let prefixes = BgpDumpParser::parse_csv_str(csv_data).unwrap();
        assert_eq!(prefixes.len(), 3);

        assert_eq!(prefixes[0].prefix, "192.168.0.0/16");
        assert_eq!(prefixes[0].asn, 15169);
        assert_eq!(prefixes[0].as_path, vec![15169, 3356, 1299]);

        assert_eq!(prefixes[2].prefix, "2001:db8::/32");
        assert_eq!(prefixes[2].asn, 6939);
    }

    #[test]
    fn test_extract_ipv6_seeds() {
        let csv_data = "prefix,asn,as_path,origin_asn\n\
            192.168.0.0/16,15169,15169,15169\n\
            2001:db8::/32,6939,6939,6939\n\
            2001:db8:abcd::/48,6939,6939,6939\n\
            2001:db8:1234:5678::/64,6939,6939,6939\n";

        let prefixes = BgpDumpParser::parse_csv_str(csv_data).unwrap();
        let ipv6_seeds = BgpDumpParser::extract_ipv6_seed_prefixes(&prefixes, 48);

        // Should get /32 and /48, not /64 (which is > 48)
        assert_eq!(ipv6_seeds.len(), 2);
        assert!(ipv6_seeds.iter().all(|p| p.prefix.contains(':')));
    }

    #[test]
    fn test_prefixes_for_asn() {
        let csv_data = "prefix,asn,as_path,origin_asn\n\
            10.0.0.0/8,16509,16509,16509\n\
            172.16.0.0/12,16509,16509,12345\n\
            192.168.0.0/16,15169,15169,15169\n";

        let prefixes = BgpDumpParser::parse_csv_str(csv_data).unwrap();
        let aws = BgpDumpParser::prefixes_for_asn(&prefixes, 16509);
        assert_eq!(aws.len(), 2);
    }

    // ── Unified Aggregator Tests ──

    #[test]
    fn test_unified_aggregator() {
        let mut agg = PassiveAggregator::new();

        let csv_data = "prefix,asn,as_path,origin_asn\n\
            10.0.0.0/8,16509,16509,16509\n\
            192.168.0.0/16,15169,15169,15169\n";

        agg.load_bgp_dump(csv_data.as_bytes()).unwrap();

        assert_eq!(agg.bgp_prefixes.len(), 2);
        assert_eq!(agg.unique_asns(), vec![15169, 16509]);
    }
}
