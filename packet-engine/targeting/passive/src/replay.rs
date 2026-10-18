use std::collections::HashMap;
use std::io::Read;
use std::net::Ipv4Addr;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BgpDumpParser, BgpPrefix, PassiveDnsAggregator, PassiveDnsRecord};

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("passive error: {0}")]
    Passive(#[from] crate::PassiveError),
    #[error("memory budget exceeded: {0}")]
    MemoryBudget(String),
    #[error("invalid IP address: {0}")]
    InvalidIp(String),
}

// ─── ScanCommand (lightweight, no protobuf dep for non-kafka builds) ────────

/// A scan command generated from passive intelligence replay.
///
/// This mirrors the proto ScanCommand but avoids requiring protobuf as a
/// non-optional dependency. When kafka feature is enabled, conversions to
/// `aegis_proto::ScanCommand` are provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCommand {
    pub command_id: u64,
    pub target_ip: [u8; 4],
    pub port: u16,
    pub transport: Transport,
    pub priority: f64,
    pub source: TargetSource,
    pub conditional_probability: f64,
    pub asn: u32,
    pub feature_tuple: Vec<u32>,
    pub generated_at: String,
    pub cidr_block: String,
    pub asn_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSource {
    PassiveCt,
    PassiveDns,
    BgpSeed,
}

// ─── Replay Event ───────────────────────────────────────────────────────────

/// A single event extracted during historical replay.
#[derive(Debug, Clone)]
pub struct ReplayEvent {
    /// IP address to scan.
    pub ip: [u8; 4],
    /// ASN that owns the prefix.
    pub asn: u32,
    /// CIDR block the IP belongs to.
    pub cidr_block: String,
    /// Source type of this event.
    pub source: TargetSource,
    /// Recency weight: 1.0 = today, 0.0 = 30 days ago.
    pub recency_weight: f64,
    /// Frequency weight: how many times this IP appeared in the dataset.
    pub frequency_weight: f64,
    /// Associated domain (from passive DNS, if any).
    pub domain: Option<String>,
}

// ─── Replay Configuration ───────────────────────────────────────────────────

/// Configuration for the historical replay engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Maximum memory budget in bytes (default: 512MB).
    pub memory_budget_bytes: usize,
    /// Number of parallel workers for processing.
    pub num_workers: usize,
    /// Batch size for chunked processing.
    pub batch_size: usize,
    /// Number of days to replay (default: 30).
    pub replay_days: u32,
    /// Minimum frequency weight to emit a scan command.
    pub min_frequency_threshold: f64,
    /// Maximum scan commands to emit per replay cycle.
    pub max_commands: usize,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            memory_budget_bytes: 512 * 1024 * 1024,
            num_workers: rayon::current_num_threads(),
            batch_size: 10000,
            replay_days: 30,
            min_frequency_threshold: 0.1,
            max_commands: 1_000_000,
        }
    }
}

// ─── Historical Replay Engine ───────────────────────────────────────────────

/// A parsed BGP route used for longest-prefix matching.
#[derive(Debug, Clone)]
struct PrefixRoute {
    /// Network address as u32.
    network: u32,
    /// Prefix length (e.g., 8, 16, 24).
    prefix_len: u8,
    /// Announcing ASN.
    asn: u32,
    /// Original CIDR string (normalized).
    prefix: String,
}

impl PrefixRoute {
    fn parse(cidr: &str, asn: u32) -> Result<Self, ReplayError> {
        let (ip_str, len_str) = cidr
            .split_once('/')
            .ok_or_else(|| ReplayError::InvalidIp(cidr.to_string()))?;
        let ip: Ipv4Addr = ip_str
            .parse()
            .map_err(|_| ReplayError::InvalidIp(ip_str.to_string()))?;
        let prefix_len: u8 = len_str
            .trim()
            .parse()
            .map_err(|_| ReplayError::InvalidIp(cidr.to_string()))?;
        if prefix_len > 32 {
            return Err(ReplayError::InvalidIp(cidr.to_string()));
        }

        let addr = u32::from_be_bytes(ip.octets());
        let mask: u32 = if prefix_len == 0 {
            0
        } else {
            (!0u32) << (32 - prefix_len)
        };
        let network = addr & mask;

        Ok(Self {
            network,
            prefix_len,
            asn,
            prefix: format!("{}/{}", Ipv4Addr::from(network), prefix_len),
        })
    }

    /// Netmask for this prefix length.
    fn mask(prefix_len: u8) -> u32 {
        if prefix_len == 0 {
            0
        } else {
            (!0u32) << (32 - prefix_len)
        }
    }
}

/// Longest-prefix-match routing table indexed by prefix length.
///
/// Each prefix length gets its own HashMap of network → route, so a lookup
/// probes at most 33 maps (32 → 0) instead of scanning all routes linearly.
#[derive(Debug, Default, Clone)]
struct RouteTable {
    /// prefix_len → (network → route).
    by_len: HashMap<u8, HashMap<u32, PrefixRoute>>,
    /// All routes for iteration (sorted longest-first).
    all: Vec<PrefixRoute>,
}

impl RouteTable {
    fn insert(&mut self, route: PrefixRoute) {
        let len = route.prefix_len;
        self.all.push(route.clone());
        self.by_len
            .entry(len)
            .or_default()
            .insert(route.network, route);
    }

    /// Longest-prefix match for an address.
    fn lookup(&self, addr: u32) -> Option<&PrefixRoute> {
        for len in (0..=32).rev() {
            let mask = PrefixRoute::mask(len);
            let network = addr & mask;
            if let Some(map) = self.by_len.get(&len)
                && let Some(route) = map.get(&network)
            {
                return Some(route);
            }
        }
        None
    }
}

/// Historical replay engine that processes 30-day passive DNS + BGP dumps
/// and generates prioritized ScanCommand targets.
pub struct HistoricalReplayEngine {
    config: ReplayConfig,
    dns_aggregator: PassiveDnsAggregator,
    bgp_prefixes: Vec<BgpPrefix>,
    /// Longest-prefix-match routing table built from BGP prefixes.
    routes: RouteTable,
    /// IP → occurrence count across all sources.
    ip_frequency: HashMap<[u8; 4], u64>,
}

impl HistoricalReplayEngine {
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            dns_aggregator: PassiveDnsAggregator::new(),
            bgp_prefixes: Vec::new(),
            routes: RouteTable::default(),
            ip_frequency: HashMap::new(),
        }
    }

    /// Load BGP prefixes from a CSV dump.
    pub fn load_bgp_dump<R: Read>(&mut self, reader: R) -> Result<usize, ReplayError> {
        let prefixes = BgpDumpParser::parse_csv(reader)?;
        let count = prefixes.len();
        self.add_bgp_prefixes(prefixes);
        Ok(count)
    }

    /// Ingest a batch of passive DNS records.
    pub fn ingest_dns_batch(&mut self, records: Vec<PassiveDnsRecord>) {
        for record in &records {
            if let Ok(ip) = record.ip_address.parse::<Ipv4Addr>() {
                let ip_u32: [u8; 4] = ip.octets();
                *self.ip_frequency.entry(ip_u32).or_insert(0) += 1;
            }
        }
        self.dns_aggregator.ingest_batch(records);
    }

    /// Ingest a single passive DNS record.
    pub fn ingest_dns(&mut self, record: PassiveDnsRecord) {
        if let Ok(ip) = record.ip_address.parse::<Ipv4Addr>() {
            let ip_u32: [u8; 4] = ip.octets();
            *self.ip_frequency.entry(ip_u32).or_insert(0) += 1;
        }
        self.dns_aggregator.ingest(record);
    }

    /// Ingest BGP prefixes from a CSV reader (for incremental updates).
    pub fn ingest_bgp_stream<R: Read>(&mut self, reader: R) -> Result<usize, ReplayError> {
        self.load_bgp_dump(reader)
    }

    /// Longest-prefix match: find the most specific route covering an IP.
    fn lookup_route(&self, ip: [u8; 4]) -> Option<&PrefixRoute> {
        self.routes.lookup(u32::from_be_bytes(ip))
    }

    /// ASN owning the prefix that covers this IP (0 if unknown).
    fn lookup_asn(&self, ip: [u8; 4]) -> u32 {
        self.lookup_route(ip).map(|r| r.asn).unwrap_or(0)
    }

    /// CIDR block covering this IP (empty if unknown).
    fn lookup_cidr(&self, ip: [u8; 4]) -> String {
        self.lookup_route(ip)
            .map(|r| r.prefix.clone())
            .unwrap_or_default()
    }

    fn add_bgp_prefixes(&mut self, prefixes: Vec<BgpPrefix>) {
        for prefix in &prefixes {
            if let Ok(route) = PrefixRoute::parse(&prefix.prefix, prefix.asn) {
                self.routes.insert(route);
            }
        }
        self.bgp_prefixes.extend(prefixes);
    }

    /// Replay all loaded data and generate prioritized ScanCommands.
    ///
    /// Deduplicates IPs, enriches with ASN/CIDR data, and scores by
    /// frequency × recency to prioritize the most valuable scan targets.
    pub fn replay(&self) -> Vec<ScanCommand> {
        let mut events: Vec<ReplayEvent> = Vec::new();

        // 1. Extract events from passive DNS: every unique IP gets an event.
        let all_domains = self.collect_all_domain_ip_pairs();
        for (ip, domain, freq) in all_domains {
            let asn = self.lookup_asn(ip);
            let cidr = self.lookup_cidr(ip);

            events.push(ReplayEvent {
                ip,
                asn,
                cidr_block: cidr,
                source: TargetSource::PassiveDns,
                recency_weight: 1.0,
                frequency_weight: freq as f64,
                domain: Some(domain),
            });
        }

        // 2. Extract events from BGP prefixes: base IPs of announced prefixes.
        let bgp_events = self.collect_bgp_events();
        events.extend(bgp_events);

        // 3. Deduplicate and merge: keep highest-scored event per IP.
        let mut merged: HashMap<[u8; 4], ReplayEvent> = HashMap::new();
        for event in events {
            let score = event.recency_weight * event.frequency_weight;
            merged
                .entry(event.ip)
                .and_modify(|existing| {
                    let existing_score = existing.recency_weight * existing.frequency_weight;
                    if score > existing_score {
                        *existing = event.clone();
                    }
                })
                .or_insert(event);
        }

        // 4. Apply frequency weight from aggregate counter.
        for event in merged.values_mut() {
            let agg_freq = self.ip_frequency.get(&event.ip).copied().unwrap_or(1);
            event.frequency_weight = (agg_freq as f64).sqrt();
        }

        // 5. Score and sort.
        let mut results: Vec<ReplayEvent> = merged.into_values().collect();
        results.sort_by(|a, b| {
            let sa = a.recency_weight * a.frequency_weight;
            let sb = b.recency_weight * b.frequency_weight;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 6. Truncate to max_commands and convert to ScanCommands.
        results.truncate(self.config.max_commands);

        let now = chrono_free_now();
        results
            .into_iter()
            .enumerate()
            .filter(|(_, e)| e.frequency_weight >= self.config.min_frequency_threshold)
            .map(|(i, event)| self.event_to_scan_command(event, i as u64, &now))
            .collect()
    }

    /// Replay in time-windowed batches, yielding commands incrementally.
    ///
    /// This is memory-efficient for very large datasets: it processes
    /// `batch_size` records at a time and streams commands via callback.
    pub fn replay_streaming<F>(&self, mut on_batch: F) -> Result<usize, ReplayError>
    where
        F: FnMut(Vec<ScanCommand>) -> Result<(), ReplayError>,
    {
        let all_domains = self.collect_all_domain_ip_pairs();
        let batch_size = self.config.batch_size;
        let now = chrono_free_now();
        let mut total_commands = 0usize;

        for chunk in all_domains.chunks(batch_size) {
            let cmds: Vec<ScanCommand> = chunk
                .par_iter()
                .map(|(ip, _domain, freq)| {
                    let asn = self.lookup_asn(*ip);
                    let cidr = self.lookup_cidr(*ip);

                    ScanCommand {
                        command_id: generate_command_id(ip, total_commands as u64),
                        target_ip: *ip,
                        port: 0,
                        transport: Transport::Tcp,
                        priority: (*freq as f64).sqrt(),
                        source: TargetSource::PassiveDns,
                        conditional_probability: 0.0,
                        asn,
                        feature_tuple: vec![],
                        generated_at: now.clone(),
                        cidr_block: cidr,
                        asn_name: String::new(),
                    }
                })
                .collect();

            total_commands += cmds.len();
            on_batch(cmds)?;
        }

        Ok(total_commands)
    }

    /// Total unique IPs observed.
    pub fn unique_ip_count(&self) -> usize {
        self.ip_frequency.len()
    }

    /// Total passive DNS records ingested.
    pub fn dns_record_count(&self) -> usize {
        self.dns_aggregator.record_count()
    }

    /// Total BGP prefixes loaded.
    pub fn bgp_prefix_count(&self) -> usize {
        self.bgp_prefixes.len()
    }

    // ─── Private helpers ──────────────────────────────────────────────

    fn collect_all_domain_ip_pairs(&self) -> Vec<([u8; 4], String, u64)> {
        let mut pairs = Vec::new();
        for (domain, ips) in self.dns_aggregator.domain_to_ips_iter() {
            for ip_str in ips {
                if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                    let octets = ip.octets();
                    let freq = self.ip_frequency.get(&octets).copied().unwrap_or(1);
                    pairs.push((octets, domain.to_string(), freq));
                }
            }
        }
        pairs
    }

    fn collect_bgp_events(&self) -> Vec<ReplayEvent> {
        self.routes
            .all
            .par_iter()
            .map(|route| {
                let base_ip = route.network.to_be_bytes();
                let freq = self.ip_frequency.get(&base_ip).copied().unwrap_or(1);
                ReplayEvent {
                    ip: base_ip,
                    asn: route.asn,
                    cidr_block: route.prefix.clone(),
                    source: TargetSource::BgpSeed,
                    recency_weight: 0.5,
                    frequency_weight: freq as f64,
                    domain: None,
                }
            })
            .collect()
    }

    fn event_to_scan_command(&self, event: ReplayEvent, seq: u64, now: &str) -> ScanCommand {
        let priority = event.recency_weight * event.frequency_weight;
        ScanCommand {
            command_id: generate_command_id(&event.ip, seq),
            target_ip: event.ip,
            port: 0,
            transport: Transport::Tcp,
            priority,
            source: event.source,
            conditional_probability: 0.0,
            asn: event.asn,
            feature_tuple: vec![],
            generated_at: now.to_string(),
            cidr_block: event.cidr_block,
            asn_name: String::new(),
        }
    }
}

// ─── Standalone replay from readers ─────────────────────────────────────────

/// Replay passive DNS CSV + BGP CSV into ScanCommands in a single call.
///
/// This is the simplest entry point: load both dumps, replay, return commands.
pub fn replay_from_readers<R1: Read, R2: Read>(
    dns_reader: R1,
    bgp_reader: R2,
    config: ReplayConfig,
) -> Result<Vec<ScanCommand>, ReplayError> {
    let mut engine = HistoricalReplayEngine::new(config);

    // Load BGP prefixes first (provides ASN/CIDR mapping).
    engine.load_bgp_dump(bgp_reader)?;

    // Parse passive DNS CSV records.
    let mut rdr = csv::Reader::from_reader(dns_reader);
    let mut batch = Vec::new();
    for result in rdr.deserialize() {
        let record: PassiveDnsRecord = result?;
        batch.push(record);
        if batch.len() >= 10000 {
            engine.ingest_dns_batch(std::mem::take(&mut batch));
        }
    }
    if !batch.is_empty() {
        engine.ingest_dns_batch(batch);
    }

    Ok(engine.replay())
}

// ─── Utility functions ──────────────────────────────────────────────────────

/// Parse the base IP from a CIDR prefix (e.g., "192.168.0.0/16" → [192,168,0,0]).
#[cfg(test)]
fn parse_prefix_base_ip(prefix: &str) -> Result<[u8; 4], ReplayError> {
    let ip_str = prefix
        .split('/')
        .next()
        .ok_or_else(|| ReplayError::InvalidIp(prefix.to_string()))?;
    let ip: Ipv4Addr = ip_str
        .parse()
        .map_err(|_| ReplayError::InvalidIp(ip_str.to_string()))?;
    Ok(ip.octets())
}

/// Generate a deterministic command ID from IP + sequence.
fn generate_command_id(ip: &[u8; 4], seq: u64) -> u64 {
    let ip_part =
        ((ip[0] as u64) << 24) | ((ip[1] as u64) << 16) | ((ip[2] as u64) << 8) | (ip[3] as u64);
    (ip_part << 16) | (seq & 0xFFFF)
}

/// Approximate current timestamp as ISO-8601 string (no external crate dep).
fn chrono_free_now() -> String {
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

    fn make_dns_record(domain: &str, ip: &str) -> PassiveDnsRecord {
        PassiveDnsRecord {
            domain: domain.to_string(),
            ip_address: ip.to_string(),
            record_type: "A".to_string(),
            first_seen: "2026-08-01T00:00:00Z".to_string(),
            last_seen: "2026-08-31T00:00:00Z".to_string(),
            ttl: Some(3600),
        }
    }

    #[test]
    fn test_parse_prefix_base_ip() {
        let ip = parse_prefix_base_ip("192.168.0.0/16").unwrap();
        assert_eq!(ip, [192, 168, 0, 0]);

        let ip = parse_prefix_base_ip("10.0.0.0/8").unwrap();
        assert_eq!(ip, [10, 0, 0, 0]);
    }

    #[test]
    fn test_parse_prefix_base_ip_invalid() {
        assert!(parse_prefix_base_ip("not-an-ip/24").is_err());
    }

    #[test]
    fn test_replay_dns_generates_commands() {
        let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());

        let bgp_csv = "prefix,asn,as_path,origin_asn\n\
                        192.168.0.0/16,64500,64500,64500\n\
                        10.0.0.0/8,64501,64501,64501\n";

        engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();

        engine.ingest_dns(make_dns_record("a.example.com", "192.168.0.1"));
        engine.ingest_dns(make_dns_record("b.example.com", "192.168.0.2"));
        engine.ingest_dns(make_dns_record("c.example.com", "10.0.0.1"));
        engine.ingest_dns(make_dns_record("d.example.com", "10.0.0.2"));

        let commands = engine.replay();

        // 4 DNS IPs + 2 BGP prefix base IPs = 6 total
        assert_eq!(commands.len(), 6);
        // All commands should have valid ASN
        assert!(commands.iter().all(|c| c.asn == 64500 || c.asn == 64501));
        // DNS IPs must be present
        assert!(commands.iter().any(|c| c.target_ip == [192, 168, 0, 1]));
        assert!(commands.iter().any(|c| c.target_ip == [10, 0, 0, 2]));
    }

    #[test]
    fn test_replay_deduplicates_ip() {
        let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());

        let bgp_csv = "prefix,asn,as_path,origin_asn\n192.168.0.0/16,64500,64500,64500\n";
        engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();

        // Same IP from multiple domains
        engine.ingest_dns(make_dns_record("a.example.com", "192.168.0.1"));
        engine.ingest_dns(make_dns_record("b.example.com", "192.168.0.1"));
        engine.ingest_dns(make_dns_record("c.example.com", "192.168.0.1"));

        let commands = engine.replay();

        // 1 deduplicated DNS command for 192.168.0.1 + 1 BGP prefix event = 2
        assert_eq!(commands.len(), 2);
        let dns_cmds: Vec<_> = commands
            .iter()
            .filter(|c| c.source == TargetSource::PassiveDns)
            .collect();
        assert_eq!(dns_cmds.len(), 1);
        assert_eq!(dns_cmds[0].target_ip, [192, 168, 0, 1]);
    }

    #[test]
    fn test_replay_frequency_boost() {
        let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());

        let bgp_csv = "prefix,asn,as_path,origin_asn\n192.168.0.0/16,64500,64500,64500\n";
        engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();

        // IP with high frequency (many domains)
        for i in 0..20 {
            engine.ingest_dns(make_dns_record(
                &format!("host{i}.example.com"),
                "192.168.0.1",
            ));
        }

        // IP with low frequency
        engine.ingest_dns(make_dns_record("low.example.com", "192.168.0.2"));

        let commands = engine.replay();

        // 2 DNS IPs + 1 BGP prefix event = 3
        assert_eq!(commands.len(), 3);
        let dns_cmds: Vec<_> = commands
            .iter()
            .filter(|c| c.source == TargetSource::PassiveDns)
            .collect();
        assert_eq!(dns_cmds.len(), 2);
        // High-frequency IP should have higher priority than low-frequency IP
        assert!(dns_cmds[0].priority >= dns_cmds[1].priority);
    }

    #[test]
    fn test_replay_empty_data() {
        let engine = HistoricalReplayEngine::new(ReplayConfig::default());
        let commands = engine.replay();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_replay_streaming() {
        let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());

        let bgp_csv = "prefix,asn,as_path,origin_asn\n192.168.0.0/16,64500,64500,64500\n";
        engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();

        for i in 0..100 {
            engine.ingest_dns(make_dns_record(
                &format!("host{i}.example.com"),
                &format!("192.168.{}.{}", (i / 256) + 1, i % 256),
            ));
        }

        let mut total = 0usize;
        engine
            .replay_streaming(|batch| {
                total += batch.len();
                Ok(())
            })
            .unwrap();

        assert_eq!(total, 100);
    }

    #[test]
    fn test_replay_from_readers() {
        let dns_csv = "domain,ip_address,record_type,first_seen,last_seen,ttl\n\
                       a.example.com,192.168.0.1,A,2026-08-01,2026-08-31,3600\n\
                       b.example.com,192.168.0.2,A,2026-08-01,2026-08-31,3600\n";
        let bgp_csv = "prefix,asn,as_path,origin_asn\n\
                       192.168.0.0/16,64500,64500,64500\n";

        let commands = replay_from_readers(
            dns_csv.as_bytes(),
            bgp_csv.as_bytes(),
            ReplayConfig::default(),
        )
        .unwrap();

        // 2 DNS IPs + 1 BGP prefix event = 3
        assert_eq!(commands.len(), 3);
        assert!(commands.iter().any(|c| c.target_ip == [192, 168, 0, 1]));
        assert!(commands.iter().any(|c| c.target_ip == [192, 168, 0, 2]));
    }

    #[test]
    fn test_generate_command_id_deterministic() {
        let ip1 = [192, 168, 0, 1];
        let ip2 = [192, 168, 0, 2];

        let id1 = generate_command_id(&ip1, 0);
        let id2 = generate_command_id(&ip2, 0);
        let id1_seq = generate_command_id(&ip1, 1);

        assert_ne!(id1, id2);
        assert_ne!(id1, id1_seq);
    }

    #[test]
    fn test_chrono_free_now_format() {
        let ts = chrono_free_now();
        // Should be valid ISO-8601: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20, "Timestamp should be 20 chars: {}", ts);
        assert!(ts.ends_with('Z'), "Should end with Z: {}", ts);
        assert_eq!(ts.chars().nth(4), Some('-'), "Should have dash after year");
        assert_eq!(ts.chars().nth(7), Some('-'), "Should have dash after month");
        assert_eq!(ts.chars().nth(10), Some('T'), "Should have T separator");
        assert_eq!(
            ts.chars().nth(13),
            Some(':'),
            "Should have colon after hours"
        );
        assert_eq!(
            ts.chars().nth(16),
            Some(':'),
            "Should have colon after minutes"
        );

        // Verify month is 01-12, not hardcoded to 01
        let month: u32 = ts[5..7].parse().unwrap();
        assert!(
            (1..=12).contains(&month),
            "Month should be 1-12, got {}",
            month
        );

        // Verify day is 01-31, not hardcoded to 01
        let day: u32 = ts[8..10].parse().unwrap();
        assert!((1..=31).contains(&day), "Day should be 1-31, got {}", day);
    }

    #[test]
    fn test_bgp_prefix_enrichment() {
        let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());

        let bgp_csv = "prefix,asn,as_path,origin_asn\n\
                        10.0.0.0/8,16509,16509,16509\n\
                        172.16.0.0/12,14618,14618,14618\n";
        engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();

        engine.ingest_dns(make_dns_record("aws.example.com", "10.0.0.1"));
        engine.ingest_dns(make_dns_record("gcp.example.com", "172.16.0.1"));

        let commands = engine.replay();

        // 2 DNS IPs + 2 BGP prefix base IPs = 4 total
        assert_eq!(commands.len(), 4);
        let aws_cmd = commands
            .iter()
            .find(|c| c.target_ip == [10, 0, 0, 1])
            .unwrap();
        assert_eq!(aws_cmd.asn, 16509);
        assert_eq!(aws_cmd.source, TargetSource::PassiveDns);

        let gcp_cmd = commands
            .iter()
            .find(|c| c.target_ip == [172, 16, 0, 1])
            .unwrap();
        assert_eq!(gcp_cmd.asn, 14618);
        assert_eq!(gcp_cmd.source, TargetSource::PassiveDns);

        // BGP events should carry the BgpSeed source
        assert_eq!(
            commands
                .iter()
                .filter(|c| c.source == TargetSource::BgpSeed)
                .count(),
            2
        );
    }

    #[test]
    fn test_memory_budget_config() {
        let config = ReplayConfig {
            memory_budget_bytes: 256 * 1024 * 1024,
            batch_size: 5000,
            max_commands: 500_000,
            ..Default::default()
        };

        let engine = HistoricalReplayEngine::new(config.clone());
        assert_eq!(engine.config.memory_budget_bytes, 256 * 1024 * 1024);
        assert_eq!(engine.config.batch_size, 5000);
        assert_eq!(engine.config.max_commands, 500_000);
    }
}
