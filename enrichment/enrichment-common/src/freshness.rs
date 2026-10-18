//! Data Freshness SLA Tracker
//!
//! Per-source (RPKI, MaxMind, NVD) freshness metric tracking with alerts
//! when data exceeds configured staleness thresholds (default: >24h)

use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use thiserror::Error;
use tracing::{info, warn};

static NEXT_ALERT_ID: AtomicU64 = AtomicU64::new(1);

/// Errors for freshness tracking
#[derive(Error, Debug)]
pub enum FreshnessError {
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Enrichment data source identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FreshnessSource {
    /// RPKI ROA/manifest data
    Rpki,
    /// MaxMind GeoIP database
    MaxMind,
    /// NVD CVE database
    Nvd,
    /// BGP routing tables
    Bgp,
    /// TLS certificate transparency logs
    CtLogs,
    /// DNS zone data
    Dns,
}

impl FreshnessSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            FreshnessSource::Rpki => "RPKI",
            FreshnessSource::MaxMind => "MaxMind",
            FreshnessSource::Nvd => "NVD",
            FreshnessSource::Bgp => "BGP",
            FreshnessSource::CtLogs => "CTLogs",
            FreshnessSource::Dns => "DNS",
        }
    }

    /// Default staleness threshold per source (hours)
    pub fn default_staleness_hours(&self) -> u32 {
        match self {
            FreshnessSource::Rpki => 24,
            FreshnessSource::MaxMind => 168, // weekly
            FreshnessSource::Nvd => 24,
            FreshnessSource::Bgp => 12,
            FreshnessSource::CtLogs => 6,
            FreshnessSource::Dns => 24,
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Freshness alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub source: FreshnessSource,
    pub level: AlertLevel,
    pub message: String,
    pub age_hours: f64,
    pub threshold_hours: u32,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
}

/// Per-source freshness state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFreshness {
    pub source: FreshnessSource,
    pub last_update: Option<DateTime<Utc>>,
    pub last_successful_update: Option<DateTime<Utc>>,
    pub update_count: u64,
    pub failure_count: u64,
    pub staleness_threshold: Duration,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
}

impl SourceFreshness {
    pub fn new(source: FreshnessSource, threshold_hours: u32) -> Self {
        Self {
            source,
            last_update: None,
            last_successful_update: None,
            update_count: 0,
            failure_count: 0,
            staleness_threshold: Duration::hours(threshold_hours as i64),
            last_error: None,
            last_error_at: None,
        }
    }

    pub fn age_hours(&self) -> Option<f64> {
        self.last_update
            .map(|last| (Utc::now() - last).num_seconds() as f64 / 3600.0)
    }

    pub fn is_stale(&self) -> bool {
        self.age_hours()
            .map(|age| age > self.staleness_threshold.num_seconds() as f64 / 3600.0)
            .unwrap_or(true)
    }

    pub fn is_critical(&self) -> bool {
        self.age_hours()
            .map(|age| age > self.staleness_threshold.num_seconds() as f64 / 3600.0 * 2.0)
            .unwrap_or(true)
    }

    pub fn record_update(&mut self, success: bool, error: Option<String>) {
        self.update_count += 1;
        self.last_update = Some(Utc::now());

        if success {
            self.last_successful_update = Some(Utc::now());
            self.last_error = None;
            self.last_error_at = None;
        } else {
            self.failure_count += 1;
            self.last_error = error;
            self.last_error_at = Some(Utc::now());
        }
    }
}

/// Configuration for freshness tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessConfig {
    pub sources: HashMap<FreshnessSource, u32>, // source -> staleness threshold hours
    pub check_interval_secs: u64,
    pub alert_on_stale: bool,
    pub alert_on_critical: bool,
    pub persist_path: Option<String>,
}

impl Default for FreshnessConfig {
    fn default() -> Self {
        let mut sources = HashMap::new();
        sources.insert(FreshnessSource::Rpki, 24);
        sources.insert(FreshnessSource::MaxMind, 168);
        sources.insert(FreshnessSource::Nvd, 24);
        sources.insert(FreshnessSource::Bgp, 12);
        sources.insert(FreshnessSource::CtLogs, 6);
        sources.insert(FreshnessSource::Dns, 24);

        Self {
            sources,
            check_interval_secs: 300, // 5 minutes
            alert_on_stale: true,
            alert_on_critical: true,
            persist_path: Some("enrichment_freshness.json".to_string()),
        }
    }
}

/// Tracks freshness of all enrichment data sources
pub struct FreshnessTracker {
    config: FreshnessConfig,
    sources: Arc<RwLock<HashMap<FreshnessSource, SourceFreshness>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
}

impl FreshnessTracker {
    pub fn new(config: FreshnessConfig) -> Self {
        let mut sources = HashMap::new();
        for (source, threshold) in &config.sources {
            sources.insert(*source, SourceFreshness::new(*source, *threshold));
        }

        Self {
            config,
            sources: Arc::new(RwLock::new(sources)),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a successful update for a source
    pub fn record_success(&self, source: FreshnessSource) {
        let mut sources = self.sources.write();
        if let Some(state) = sources.get_mut(&source) {
            state.record_update(true, None);
            info!(
                "Source {} updated successfully (total: {})",
                source.as_str(),
                state.update_count
            );
        }
    }

    /// Record a failed update for a source
    pub fn record_failure(&self, source: FreshnessSource, error: String) {
        let mut sources = self.sources.write();
        if let Some(state) = sources.get_mut(&source) {
            state.record_update(false, Some(error.clone()));
            warn!("Source {} update failed: {}", source.as_str(), error);
        }
    }

    /// Get freshness state for a source
    pub fn get_freshness(
        &self,
        source: FreshnessSource,
    ) -> Result<SourceFreshness, FreshnessError> {
        let sources = self.sources.read();
        sources
            .get(&source)
            .cloned()
            .ok_or_else(|| FreshnessError::SourceNotFound(source.as_str().to_string()))
    }

    /// Get all freshness states
    pub fn all_freshness(&self) -> Vec<SourceFreshness> {
        self.sources.read().values().cloned().collect()
    }

    /// Check if an unacknowledged alert already exists for the given source and level
    fn has_existing_alert(&self, source: FreshnessSource, level: AlertLevel) -> bool {
        self.alerts
            .read()
            .iter()
            .any(|a| a.source == source && a.level == level && !a.acknowledged)
    }

    /// Check all sources and generate alerts if needed
    pub fn check_and_alert(&self) -> Vec<Alert> {
        let mut new_alerts = Vec::new();
        let sources = self.sources.read();

        for (source, state) in sources.iter() {
            if let Some(age) = state.age_hours() {
                let threshold = self.config.sources.get(source).copied().unwrap_or(24) as f64;

                if age > threshold * 2.0 && self.config.alert_on_critical {
                    if !self.has_existing_alert(*source, AlertLevel::Critical) {
                        let alert = Alert {
                            id: NEXT_ALERT_ID.fetch_add(1, Ordering::Relaxed),
                            source: *source,
                            level: AlertLevel::Critical,
                            message: format!(
                                "{} data critically stale: {:.1}h old (threshold: {}h)",
                                source.as_str(),
                                age,
                                threshold
                            ),
                            age_hours: age,
                            threshold_hours: threshold as u32,
                            triggered_at: Utc::now(),
                            acknowledged: false,
                        };
                        new_alerts.push(alert);
                    }
                } else if age > threshold
                    && self.config.alert_on_stale
                    && !self.has_existing_alert(*source, AlertLevel::Warning)
                {
                    let alert = Alert {
                        id: NEXT_ALERT_ID.fetch_add(1, Ordering::Relaxed),
                        source: *source,
                        level: AlertLevel::Warning,
                        message: format!(
                            "{} data stale: {:.1}h old (threshold: {}h)",
                            source.as_str(),
                            age,
                            threshold
                        ),
                        age_hours: age,
                        threshold_hours: threshold as u32,
                        triggered_at: Utc::now(),
                        acknowledged: false,
                    };
                    new_alerts.push(alert);
                }
            } else {
                // Never updated - critical
                if self.config.alert_on_critical
                    && !self.has_existing_alert(*source, AlertLevel::Critical)
                {
                    let threshold = self.config.sources.get(source).copied().unwrap_or(24);
                    let alert = Alert {
                        id: NEXT_ALERT_ID.fetch_add(1, Ordering::Relaxed),
                        source: *source,
                        level: AlertLevel::Critical,
                        message: format!(
                            "{} data never updated (threshold: {}h)",
                            source.as_str(),
                            threshold
                        ),
                        age_hours: f64::INFINITY,
                        threshold_hours: threshold,
                        triggered_at: Utc::now(),
                        acknowledged: false,
                    };
                    new_alerts.push(alert);
                }
            }
        }

        if !new_alerts.is_empty() {
            let mut alerts = self.alerts.write();
            alerts.extend(new_alerts.clone());
        }

        new_alerts
    }

    /// Get unacknowledged alerts
    pub fn get_unacknowledged_alerts(&self) -> Vec<Alert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert by ID
    pub fn acknowledge_alert(&self, id: u64) -> Result<(), FreshnessError> {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(FreshnessError::Config(format!(
                "Alert with id {} not found",
                id
            )))
        }
    }

    /// Persist freshness state to disk
    pub fn persist(&self) -> Result<(), FreshnessError> {
        if let Some(path) = &self.config.persist_path {
            let data = self.all_freshness();
            let json = serde_json::to_string_pretty(&data)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// Load freshness state from disk, only if in-memory source is empty or older
    pub fn load(&self) -> Result<(), FreshnessError> {
        if let Some(path) = &self.config.persist_path {
            if std::path::Path::new(path).exists() {
                let json = std::fs::read_to_string(path)?;
                let data: Vec<SourceFreshness> = serde_json::from_str(&json)?;
                let mut sources = self.sources.write();
                for state in data {
                    if let Some(existing) = sources.get(&state.source) {
                        // Only overwrite if the in-memory source has never been updated
                        // or if the disk data is newer
                        let memory_newer = match (existing.last_update, state.last_update) {
                            (Some(mem), Some(disk)) => mem >= disk,
                            (Some(_), None) => true,
                            (None, _) => false,
                        };
                        if !memory_newer {
                            sources.insert(state.source, state);
                        }
                    } else {
                        sources.insert(state.source, state);
                    }
                }
            }
        }
        Ok(())
    }

    /// Start background freshness checker
    pub async fn start_background_checker(self: Arc<Self>) {
        let mut interval =
            tokio::time::interval(StdDuration::from_secs(self.config.check_interval_secs));

        loop {
            interval.tick().await;
            let alerts = self.check_and_alert();
            if !alerts.is_empty() {
                warn!("Freshness alerts generated: {} alerts", alerts.len());
                for alert in &alerts {
                    warn!("[{:?}] {}", alert.level, alert.message);
                }
            }
        }
    }
}

/// Aggregate freshness health score (0.0 - 1.0)
pub fn compute_freshness_health(tracker: &FreshnessTracker) -> f64 {
    let states = tracker.all_freshness();
    if states.is_empty() {
        return 0.0;
    }

    let healthy_count = states.iter().filter(|s| !s.is_stale()).count();

    healthy_count as f64 / states.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_freshness_new() {
        let freshness = SourceFreshness::new(FreshnessSource::Rpki, 24);
        assert_eq!(freshness.source, FreshnessSource::Rpki);
        assert_eq!(freshness.staleness_threshold, Duration::hours(24));
        assert!(freshness.is_stale()); // never updated
    }

    #[test]
    fn test_source_freshness_update() {
        let mut freshness = SourceFreshness::new(FreshnessSource::Nvd, 24);
        freshness.record_update(true, None);

        assert_eq!(freshness.update_count, 1);
        assert_eq!(freshness.failure_count, 0);
        assert!(!freshness.is_stale());
        assert!(!freshness.is_critical());
    }

    #[test]
    fn test_source_freshness_failure() {
        let mut freshness = SourceFreshness::new(FreshnessSource::MaxMind, 24);
        freshness.record_update(true, None);
        freshness.record_update(false, Some("network timeout".to_string()));

        assert_eq!(freshness.update_count, 2);
        assert_eq!(freshness.failure_count, 1);
        assert_eq!(freshness.last_error, Some("network timeout".to_string()));
    }

    #[test]
    fn test_staleness_threshold() {
        let mut freshness = SourceFreshness::new(FreshnessSource::Rpki, 24);
        freshness.last_update = Some(Utc::now() - Duration::hours(25));

        assert!(freshness.is_stale());
        assert!(!freshness.is_critical()); // 25h < 48h (2x threshold)
    }

    #[test]
    fn test_critical_staleness() {
        let mut freshness = SourceFreshness::new(FreshnessSource::Rpki, 24);
        freshness.last_update = Some(Utc::now() - Duration::hours(50));

        assert!(freshness.is_stale());
        assert!(freshness.is_critical()); // 50h > 48h (2x threshold)
    }

    #[tokio::test]
    async fn test_freshness_tracker() {
        let config = FreshnessConfig::default();
        let tracker = FreshnessTracker::new(config);

        tracker.record_success(FreshnessSource::Rpki);
        tracker.record_success(FreshnessSource::Nvd);
        tracker.record_failure(FreshnessSource::MaxMind, "timeout".to_string());

        let rpki = tracker.get_freshness(FreshnessSource::Rpki).unwrap();
        assert_eq!(rpki.update_count, 1);
        assert!(!rpki.is_stale());

        let maxmind = tracker.get_freshness(FreshnessSource::MaxMind).unwrap();
        assert_eq!(maxmind.failure_count, 1);
    }

    #[tokio::test]
    async fn test_freshness_alerts() {
        let config = FreshnessConfig {
            sources: {
                let mut m = HashMap::new();
                m.insert(FreshnessSource::Rpki, 24);
                m.insert(FreshnessSource::Nvd, 24);
                m
            },
            ..Default::default()
        };
        let tracker = FreshnessTracker::new(config);

        // Simulate stale data by manually setting last_update
        {
            let mut sources = tracker.sources.write();
            if let Some(rpki) = sources.get_mut(&FreshnessSource::Rpki) {
                rpki.last_update = Some(Utc::now() - Duration::hours(50));
            }
        }

        let alerts = tracker.check_and_alert();
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.level == AlertLevel::Critical));
    }

    #[test]
    fn test_compute_freshness_health() {
        let mut sources = HashMap::new();
        sources.insert(FreshnessSource::Rpki, 24);
        sources.insert(FreshnessSource::Nvd, 24);
        sources.insert(FreshnessSource::MaxMind, 168);
        let config = FreshnessConfig {
            sources,
            ..Default::default()
        };
        let tracker = FreshnessTracker::new(config);

        tracker.record_success(FreshnessSource::Rpki);
        tracker.record_success(FreshnessSource::Nvd);

        // One stale
        {
            let mut sources = tracker.sources.write();
            if let Some(maxmind) = sources.get_mut(&FreshnessSource::MaxMind) {
                maxmind.last_update = Some(Utc::now() - Duration::hours(200));
            }
        }

        let health = compute_freshness_health(&tracker);
        assert!((health - 2.0 / 3.0).abs() < 0.01); // 2 of 3 healthy
    }
}
