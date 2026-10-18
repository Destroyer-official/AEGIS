//! Enrichment Quality Audit Log
//!
//! Structured JSONL logging for enrichment operations:
//! ip, tier_used, latency_ms, confidence, fallback_triggered

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::info;

/// Errors for audit logging
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Enrichment tier used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrichmentTier {
    /// Tier 1: Cryptographic (RPKI/ROA validated)
    Cryptographic,
    /// Tier 2: RTT-based triangulation
    Rtt,
    /// Tier 3: Passive registration (GeoIP, DNS, TLS)
    Passive,
    /// Unknown/failed
    Unknown,
}

impl EnrichmentTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrichmentTier::Cryptographic => "cryptographic",
            EnrichmentTier::Rtt => "rtt",
            EnrichmentTier::Passive => "passive",
            EnrichmentTier::Unknown => "unknown",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            EnrichmentTier::Cryptographic => 1,
            EnrichmentTier::Rtt => 2,
            EnrichmentTier::Passive => 3,
            EnrichmentTier::Unknown => 4,
        }
    }
}

/// Reason for fallback to lower tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackReason {
    /// No data available at this tier
    NoData,
    /// Validation failed (e.g., ROA invalid)
    ValidationFailed,
    /// Timeout occurred
    Timeout,
    /// Rate limited
    RateLimited,
    /// Source unavailable
    SourceUnavailable,
    /// Confidence below threshold
    LowConfidence,
    /// Internal error
    Error,
    /// No fallback needed (success at this tier)
    None,
}

/// Single audit entry for one enrichment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of enrichment
    pub timestamp: DateTime<Utc>,
    /// Target IP address
    pub ip: String,
    /// Enrichment tier actually used
    pub tier_used: EnrichmentTier,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Whether fallback was triggered
    pub fallback_triggered: bool,
    /// Reason for fallback (if any)
    pub fallback_reason: FallbackReason,
    /// Source PoP that performed enrichment
    pub source_pop: String,
    /// Enrichment event ID for correlation
    pub event_id: String,
    /// Additional context (service, port, etc.)
    #[serde(flatten)]
    pub context: HashMap<String, serde_json::Value>,
}

impl AuditEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ip: impl Into<String>,
        tier_used: EnrichmentTier,
        latency_ms: u64,
        confidence: f64,
        fallback_triggered: bool,
        fallback_reason: FallbackReason,
        source_pop: impl Into<String>,
        event_id: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            ip: ip.into(),
            tier_used,
            latency_ms,
            confidence: confidence.clamp(0.0, 1.0),
            fallback_triggered,
            fallback_reason,
            source_pop: source_pop.into(),
            event_id: event_id.into(),
            context: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Serialize as JSON line
    pub fn to_json_line(&self, include_context: bool) -> Result<String, AuditError> {
        let mut value = serde_json::to_value(self)?;
        if !include_context {
            if let serde_json::Value::Object(ref mut map) = value {
                map.remove("context");
            }
        }
        let mut line = serde_json::to_string(&value)?;
        line.push('\n');
        Ok(line)
    }
}

/// Configuration for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Output directory for log files
    pub log_dir: PathBuf,
    /// Base filename (date will be appended)
    pub base_filename: String,
    /// Filename prefix for querying log files
    pub filename_prefix: String,
    /// Rotate logs daily
    pub daily_rotation: bool,
    /// Maximum log file size before rotation (MB)
    pub max_file_size_mb: u64,
    /// Buffer size for batched writes
    pub buffer_size: usize,
    /// Flush interval (seconds)
    pub flush_interval_secs: u64,
    /// Include context fields
    pub include_context: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs/enrichment"),
            base_filename: "enrichment_audit".to_string(),
            filename_prefix: "enrichment_audit".to_string(),
            daily_rotation: true,
            max_file_size_mb: 100,
            buffer_size: 1000,
            flush_interval_secs: 10,
            include_context: true,
        }
    }
}

/// Thread-safe audit log writer with buffering and rotation
pub struct AuditLog {
    config: AuditConfig,
    writer: Arc<Mutex<BufWriter<std::fs::File>>>,
    current_date: Arc<Mutex<String>>,
    buffer: Arc<Mutex<Vec<String>>>,
    stats: Arc<Mutex<AuditStats>>,
    current_file_size: Arc<Mutex<u64>>,
}

/// Audit statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub by_tier: HashMap<String, u64>,
    pub fallback_count: u64,
    pub total_latency_ms: u64,
    pub errors: u64,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new(config: AuditConfig) -> Result<Self, AuditError> {
        std::fs::create_dir_all(&config.log_dir)?;

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let file_path = config
            .log_dir
            .join(format!("{}_{}.jsonl", config.base_filename, date));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        let initial_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        let buffer_size = config.buffer_size;
        let writer = BufWriter::with_capacity(buffer_size, file);

        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::with_capacity(buffer_size)));
        let writer = Arc::new(Mutex::new(writer));
        let current_file_size = Arc::new(Mutex::new(initial_size));

        let flush_interval = config.flush_interval_secs;
        let bg_buffer = Arc::clone(&buffer);
        let bg_writer = Arc::clone(&writer);
        let bg_file_size = Arc::clone(&current_file_size);
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(flush_interval));
            let mut buf = bg_buffer.lock().unwrap();
            if !buf.is_empty() {
                let lines: Vec<String> = buf.drain(..).collect();
                drop(buf);
                let mut w = bg_writer.lock().unwrap();
                let mut written = 0u64;
                for line in &lines {
                    let _ = w.write_all(line.as_bytes());
                    written += line.len() as u64;
                }
                let _ = w.flush();
                drop(w);
                let mut fs = bg_file_size.lock().unwrap();
                *fs += written;
            }
        });

        Ok(Self {
            config,
            writer,
            current_date: Arc::new(Mutex::new(date)),
            buffer,
            stats: Arc::new(Mutex::new(AuditStats::default())),
            current_file_size,
        })
    }

    /// Write an audit entry (buffered)
    pub fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        if self.config.daily_rotation || self.config.max_file_size_mb > 0 {
            self.rotate_if_needed()?;
        }

        let line = entry.to_json_line(self.config.include_context)?;

        let should_flush = {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(line);
            buffer.len() >= self.config.buffer_size
        };

        if should_flush {
            self.flush_buffer()?;
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_entries += 1;
            *stats
                .by_tier
                .entry(entry.tier_used.as_str().to_string())
                .or_default() += 1;
            if entry.fallback_triggered {
                stats.fallback_count += 1;
            }
            stats.total_latency_ms += entry.latency_ms;
        }

        Ok(())
    }

    /// Flush buffer to disk
    pub fn flush(&self) -> Result<(), AuditError> {
        self.flush_buffer()
    }

    fn flush_buffer(&self) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(());
        }

        let mut writer = self.writer.lock().unwrap();
        let mut written = 0u64;
        for line in buffer.drain(..) {
            written += line.len() as u64;
            writer.write_all(line.as_bytes())?;
        }
        writer.flush()?;
        drop(writer);

        let mut file_size = self.current_file_size.lock().unwrap();
        *file_size += written;

        Ok(())
    }

    fn rotate_if_needed(&self) -> Result<(), AuditError> {
        let now = Utc::now();
        let new_date = now.format("%Y-%m-%d").to_string();
        let mut current_date = self.current_date.lock().unwrap();

        let should_rotate_by_date = self.config.daily_rotation && *current_date != new_date;

        let should_rotate_by_size = if self.config.max_file_size_mb > 0 {
            let file_size = self.current_file_size.lock().unwrap();
            *file_size >= self.config.max_file_size_mb * 1024 * 1024
        } else {
            false
        };

        if should_rotate_by_date || should_rotate_by_size {
            self.flush_buffer()?;

            let rotate_date = if should_rotate_by_date {
                new_date.clone()
            } else {
                now.format("%Y-%m-%d_%H%M%S").to_string()
            };

            let file_path = self.config.log_dir.join(format!(
                "{}_{}.jsonl",
                self.config.base_filename, rotate_date
            ));

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;

            let initial_size = file.metadata().map(|m| m.len()).unwrap_or(0);

            let mut writer = self.writer.lock().unwrap();
            *writer = BufWriter::with_capacity(self.config.buffer_size, file);
            drop(writer);

            let mut file_size = self.current_file_size.lock().unwrap();
            *file_size = initial_size;

            if should_rotate_by_date {
                info!("Audit log rotated to {}", new_date);
                *current_date = new_date;
            } else {
                info!(
                    "Audit log rotated due to size limit ({} MB)",
                    self.config.max_file_size_mb
                );
            }
        }

        Ok(())
    }

    /// Get current statistics
    pub fn stats(&self) -> AuditStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get average latency
    pub fn avg_latency_ms(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        if stats.total_entries == 0 {
            0.0
        } else {
            stats.total_latency_ms as f64 / stats.total_entries as f64
        }
    }

    /// Get fallback rate
    pub fn fallback_rate(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        if stats.total_entries == 0 {
            0.0
        } else {
            stats.fallback_count as f64 / stats.total_entries as f64
        }
    }

    /// Get tier distribution
    pub fn tier_distribution(&self) -> HashMap<String, u64> {
        self.stats.lock().unwrap().by_tier.clone()
    }
}

/// Convenience function to create a default audit log
pub fn default_audit_log() -> Result<AuditLog, AuditError> {
    AuditLog::new(AuditConfig::default())
}

/// Query audit log files (reads and parses JSONL)
pub fn query_audit_logs(
    log_dir: &Path,
    date: Option<&str>,
    filter_ip: Option<&str>,
    filter_tier: Option<EnrichmentTier>,
    filename_prefix: Option<&str>,
) -> Result<Vec<AuditEntry>, AuditError> {
    let prefix = filename_prefix.unwrap_or("enrichment_audit");
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            let filename = path.file_name().unwrap().to_string_lossy();
            if !filename.starts_with(prefix) {
                continue;
            }

            // If date filter specified, check filename contains it
            if let Some(d) = date {
                if !filename.contains(d) {
                    continue;
                }
            }

            let content = std::fs::read_to_string(&path)?;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                let audit_entry: AuditEntry = serde_json::from_str(line)?;

                if let Some(ip) = filter_ip {
                    if audit_entry.ip != ip {
                        continue;
                    }
                }

                if let Some(tier) = filter_tier {
                    if audit_entry.tier_used != tier {
                        continue;
                    }
                }

                entries.push(audit_entry);
            }
        }
    }

    entries.sort_by_key(|a| a.timestamp);

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            "192.168.1.1",
            EnrichmentTier::Cryptographic,
            42,
            0.98,
            false,
            FallbackReason::None,
            "pop-us-east",
            "evt-123",
        );

        assert_eq!(entry.ip, "192.168.1.1");
        assert_eq!(entry.tier_used, EnrichmentTier::Cryptographic);
        assert_eq!(entry.latency_ms, 42);
        assert_eq!(entry.confidence, 0.98);
        assert!(!entry.fallback_triggered);
        assert_eq!(entry.fallback_reason, FallbackReason::None);
    }

    #[test]
    fn test_audit_entry_with_fallback() {
        let entry = AuditEntry::new(
            "10.0.0.1",
            EnrichmentTier::Rtt,
            150,
            0.75,
            true,
            FallbackReason::ValidationFailed,
            "pop-eu-west",
            "evt-456",
        )
        .with_context("port", serde_json::json!(443));

        assert_eq!(entry.tier_used, EnrichmentTier::Rtt);
        assert!(entry.fallback_triggered);
        assert_eq!(entry.fallback_reason, FallbackReason::ValidationFailed);
        assert_eq!(entry.context.get("port"), Some(&serde_json::json!(443)));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new(
            "192.168.1.1",
            EnrichmentTier::Passive,
            10,
            0.6,
            true,
            FallbackReason::NoData,
            "pop-test",
            "evt-789",
        );

        let json = entry.to_json_line(true).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('\n'));

        let parsed: AuditEntry = serde_json::from_str(json.trim()).unwrap();
        assert_eq!(parsed.ip, entry.ip);
        assert_eq!(parsed.tier_used, entry.tier_used);
    }

    #[test]
    fn test_audit_log_creation() {
        let dir = tempdir().unwrap();
        let config = AuditConfig {
            log_dir: dir.path().to_path_buf(),
            buffer_size: 10,
            flush_interval_secs: 1,
            ..Default::default()
        };

        let log = AuditLog::new(config).unwrap();

        let entry = AuditEntry::new(
            "192.168.1.1",
            EnrichmentTier::Cryptographic,
            50,
            0.95,
            false,
            FallbackReason::None,
            "pop-1",
            "evt-1",
        );

        log.write(&entry).unwrap();
        log.flush().unwrap();

        let stats = log.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.by_tier.get("cryptographic"), Some(&1));
    }

    #[test]
    fn test_audit_log_fallback_tracking() {
        let dir = tempdir().unwrap();
        let config = AuditConfig {
            log_dir: dir.path().to_path_buf(),
            buffer_size: 10,
            ..Default::default()
        };

        let log = AuditLog::new(config).unwrap();

        let entry = AuditEntry::new(
            "10.0.0.1",
            EnrichmentTier::Rtt,
            200,
            0.7,
            true,
            FallbackReason::NoData,
            "pop-2",
            "evt-2",
        );

        log.write(&entry).unwrap();
        log.flush().unwrap();

        let stats = log.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.fallback_count, 1);
        assert_eq!(log.fallback_rate(), 1.0);
    }

    #[test]
    fn test_audit_log_buffer_flush() {
        let dir = tempdir().unwrap();
        let config = AuditConfig {
            log_dir: dir.path().to_path_buf(),
            buffer_size: 2,
            ..Default::default()
        };

        let log = AuditLog::new(config).unwrap();

        for i in 0..3 {
            let entry = AuditEntry::new(
                format!("192.168.1.{}", i),
                EnrichmentTier::Passive,
                100,
                0.5,
                false,
                FallbackReason::None,
                "pop-3",
                format!("evt-{}", i),
            );
            log.write(&entry).unwrap();
        }

        log.flush().unwrap();

        let stats = log.stats();
        assert_eq!(stats.total_entries, 3);
    }

    #[test]
    fn test_query_audit_logs() {
        let dir = tempdir().unwrap();
        let config = AuditConfig {
            log_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let log = AuditLog::new(config).unwrap();

        for i in 0..3 {
            let entry = AuditEntry::new(
                format!("192.168.1.{}", i),
                if i == 0 {
                    EnrichmentTier::Cryptographic
                } else {
                    EnrichmentTier::Passive
                },
                100 + i * 10,
                0.9 - i as f64 * 0.1,
                i > 0,
                if i > 0 {
                    FallbackReason::NoData
                } else {
                    FallbackReason::None
                },
                "pop-test",
                format!("evt-{}", i),
            );
            log.write(&entry).unwrap();
        }
        log.flush().unwrap();

        // Query all
        let results = query_audit_logs(dir.path(), None, None, None, None).unwrap();
        assert_eq!(results.len(), 3);

        // Query by IP
        let results = query_audit_logs(dir.path(), None, Some("192.168.1.1"), None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ip, "192.168.1.1");

        // Query by tier
        let results = query_audit_logs(
            dir.path(),
            None,
            None,
            Some(EnrichmentTier::Cryptographic),
            None,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tier_used, EnrichmentTier::Cryptographic);
    }

    #[test]
    fn test_tier_priority() {
        assert_eq!(EnrichmentTier::Cryptographic.priority(), 1);
        assert_eq!(EnrichmentTier::Rtt.priority(), 2);
        assert_eq!(EnrichmentTier::Passive.priority(), 3);
        assert_eq!(EnrichmentTier::Unknown.priority(), 4);
    }

    #[test]
    fn test_avg_latency() {
        let dir = tempdir().unwrap();
        let config = AuditConfig {
            log_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let log = AuditLog::new(config).unwrap();

        for i in 0..3 {
            let entry = AuditEntry::new(
                format!("10.0.0.{}", i),
                EnrichmentTier::Passive,
                (i + 1) * 100,
                0.5,
                false,
                FallbackReason::None,
                "pop",
                format!("e{}", i),
            );
            log.write(&entry).unwrap();
        }

        let avg = log.avg_latency_ms();
        assert!((avg - 200.0).abs() < 0.01); // (100+200+300)/3 = 200
    }
}
