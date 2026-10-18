//! VistaScan Coverage SLA — Per-/48 coverage %, anchor redundancy, auto-alert
//!
//! §3.6.2: "VistaScan achieves 97.95% coverage with 64–93% probe reduction."
//!
//! Monitors per-/48 coverage, anchor redundancy, and alerts if coverage <90%.
//! Zero external dependencies: self-contained metrics.

use std::collections::HashMap;

// ─────────────────────────────────────────────────────────
// §3.6.2 — Coverage SLA Types
// ─────────────────────────────────────────────────────────

/// Coverage status for a single /48 block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockCoverage {
    /// The /48 prefix (e.g. "2001:db8:abcd::/48").
    pub prefix: String,
    /// Total hosts in /48 (2^80 for full /48, but typically scanned subset).
    pub total_hosts: u64,
    /// Number of hosts successfully probed.
    pub probed_hosts: u64,
    /// Number of live hosts discovered.
    pub live_hosts: u64,
    /// Coverage percentage (probed / total).
    pub coverage_pct: f64,
    /// Number of anchors assigned to this block.
    pub anchor_count: usize,
    /// Redundancy score: average visibility of assigned anchors.
    pub anchor_redundancy: f64,
}

impl BlockCoverage {
    /// Create new block coverage.
    pub fn new(prefix: &str, total_hosts: u64) -> Self {
        Self {
            prefix: prefix.to_string(),
            total_hosts,
            probed_hosts: 0,
            live_hosts: 0,
            coverage_pct: 0.0,
            anchor_count: 0,
            anchor_redundancy: 0.0,
        }
    }

    /// Record a probe result.
    pub fn record_probe(&mut self, is_live: bool) {
        self.probed_hosts += 1;
        if is_live {
            self.live_hosts += 1;
        }
        if self.total_hosts > 0 {
            self.coverage_pct =
                (self.probed_hosts as f64 / self.total_hosts as f64 * 100.0).min(100.0);
        }
    }

    /// Update anchor information.
    pub fn set_anchors(&mut self, count: usize, avg_visibility: f64) {
        self.anchor_count = count;
        self.anchor_redundancy = avg_visibility;
    }

    /// Check if SLA is met (coverage >= threshold).
    pub fn sla_met(&self, threshold_pct: f64) -> bool {
        self.coverage_pct >= threshold_pct
    }
}

// ─────────────────────────────────────────────────────────
// §3.6.2 — SLA Alert
// ─────────────────────────────────────────────────────────

/// Alert types for SLA violations.
#[derive(Clone, Debug, PartialEq)]
pub enum SlaAlertType {
    /// Coverage below threshold.
    CoverageBelowThreshold,
    /// No anchors assigned.
    NoAnchors,
    /// Anchor redundancy too low.
    LowRedundancy,
}

/// An SLA alert for a specific block.
#[derive(Clone, Debug)]
pub struct SlaAlert {
    /// Type of alert.
    pub alert_type: SlaAlertType,
    /// Affected block prefix.
    pub prefix: String,
    /// Current value that triggered alert.
    pub current_value: f64,
    /// Threshold that was breached.
    pub threshold: f64,
    /// Alert message.
    pub message: String,
}

// ─────────────────────────────────────────────────────────
// §3.6.2 — Coverage SLA Monitor
// ─────────────────────────────────────────────────────────

/// Monitor coverage SLA across all /48 blocks.
pub struct CoverageSlaMonitor {
    /// Per-block coverage tracking.
    blocks: HashMap<String, BlockCoverage>,
    /// SLA threshold (default 90%).
    sla_threshold: f64,
    /// Minimum anchor redundancy threshold.
    min_redundancy: f64,
    /// Alert history.
    alerts: Vec<SlaAlert>,
}

impl CoverageSlaMonitor {
    /// Create new SLA monitor with default thresholds.
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            sla_threshold: 90.0,
            min_redundancy: 0.3,
            alerts: Vec::new(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(sla_threshold: f64, min_redundancy: f64) -> Self {
        Self {
            blocks: HashMap::new(),
            sla_threshold,
            min_redundancy,
            alerts: Vec::new(),
        }
    }

    /// Register a /48 block for monitoring.
    pub fn register_block(&mut self, prefix: &str, total_hosts: u64) {
        self.blocks
            .entry(prefix.to_string())
            .or_insert_with(|| BlockCoverage::new(prefix, total_hosts));
    }

    /// Record a probe result for a block.
    pub fn record_probe(&mut self, prefix: &str, is_live: bool) {
        if let Some(block) = self.blocks.get_mut(prefix) {
            block.record_probe(is_live);
        }
    }

    /// Update anchor info for a block.
    pub fn set_block_anchors(&mut self, prefix: &str, count: usize, avg_visibility: f64) {
        if let Some(block) = self.blocks.get_mut(prefix) {
            block.set_anchors(count, avg_visibility);
        }
    }

    /// Check SLA for all blocks and generate alerts.
    pub fn check_sla(&mut self) -> Vec<SlaAlert> {
        let mut new_alerts = Vec::new();

        for block in self.blocks.values() {
            // Check coverage threshold
            if !block.sla_met(self.sla_threshold) {
                let alert = SlaAlert {
                    alert_type: SlaAlertType::CoverageBelowThreshold,
                    prefix: block.prefix.clone(),
                    current_value: block.coverage_pct,
                    threshold: self.sla_threshold,
                    message: format!(
                        "Block {} coverage {:.1}% < SLA {:.1}%",
                        block.prefix, block.coverage_pct, self.sla_threshold
                    ),
                };
                new_alerts.push(alert);
            }

            // Check anchor count
            if block.anchor_count == 0 {
                let alert = SlaAlert {
                    alert_type: SlaAlertType::NoAnchors,
                    prefix: block.prefix.clone(),
                    current_value: 0.0,
                    threshold: 1.0,
                    message: format!("Block {} has no anchors assigned", block.prefix),
                };
                new_alerts.push(alert);
            }

            // Check redundancy
            if block.anchor_count > 0 && block.anchor_redundancy < self.min_redundancy {
                let alert = SlaAlert {
                    alert_type: SlaAlertType::LowRedundancy,
                    prefix: block.prefix.clone(),
                    current_value: block.anchor_redundancy,
                    threshold: self.min_redundancy,
                    message: format!(
                        "Block {} redundancy {:.2} < threshold {:.2}",
                        block.prefix, block.anchor_redundancy, self.min_redundancy
                    ),
                };
                new_alerts.push(alert);
            }
        }

        self.alerts.extend(new_alerts.clone());
        new_alerts
    }

    /// Get overall SLA status.
    pub fn get_sla_status(&self) -> SlaStatus {
        let total = self.blocks.len();
        let compliant = self
            .blocks
            .values()
            .filter(|b| b.sla_met(self.sla_threshold))
            .count();
        let avg_coverage = if total > 0 {
            self.blocks.values().map(|b| b.coverage_pct).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let avg_redundancy = if total > 0 {
            let with_anchors: Vec<_> = self
                .blocks
                .values()
                .filter(|b| b.anchor_count > 0)
                .collect();
            if with_anchors.is_empty() {
                0.0
            } else {
                with_anchors
                    .iter()
                    .map(|b| b.anchor_redundancy)
                    .sum::<f64>()
                    / with_anchors.len() as f64
            }
        } else {
            0.0
        };

        SlaStatus {
            total_blocks: total,
            compliant_blocks: compliant,
            compliance_pct: if total > 0 {
                compliant as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            avg_coverage_pct: avg_coverage,
            avg_redundancy,
            sla_threshold: self.sla_threshold,
            total_alerts: self.alerts.len(),
        }
    }

    /// Get all block coverages.
    pub fn get_block_coverages(&self) -> Vec<&BlockCoverage> {
        self.blocks.values().collect()
    }

    /// Get alert history.
    pub fn get_alerts(&self) -> &[SlaAlert] {
        &self.alerts
    }

    /// Clear alert history.
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Reset all blocks.
    pub fn reset(&mut self) {
        self.blocks.clear();
        self.alerts.clear();
    }
}

impl Default for CoverageSlaMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────
// §3.6.2 — SLA Status Summary
// ─────────────────────────────────────────────────────────

/// Summary of SLA compliance across all blocks.
#[derive(Clone, Debug)]
pub struct SlaStatus {
    pub total_blocks: usize,
    pub compliant_blocks: usize,
    pub compliance_pct: f64,
    pub avg_coverage_pct: f64,
    pub avg_redundancy: f64,
    pub sla_threshold: f64,
    pub total_alerts: usize,
}

impl SlaStatus {
    /// Check if overall SLA is met.
    pub fn is_healthy(&self) -> bool {
        self.compliance_pct >= 95.0 && self.avg_coverage_pct >= self.sla_threshold
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "SLA: {}/{} blocks compliant ({:.1}%), avg coverage {:.1}%, avg redundancy {:.2}, {} alerts",
            self.compliant_blocks, self.total_blocks, self.compliance_pct,
            self.avg_coverage_pct, self.avg_redundancy, self.total_alerts
        )
    }
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_coverage_basic() {
        let mut block = BlockCoverage::new("2001:db8:abcd::/48", 1000);

        // Probe 900 hosts, 800 live
        for _ in 0..900 {
            block.record_probe(true);
        }
        // Probe 100 more, all dead
        for _ in 0..100 {
            block.record_probe(false);
        }

        assert_eq!(block.probed_hosts, 1000);
        assert_eq!(block.live_hosts, 900);
        assert!((block.coverage_pct - 100.0).abs() < 0.01);
        assert!(block.sla_met(90.0));
        // 1000/1000 = 100% so met for 99%
        assert!(block.sla_met(99.0));
        assert!(!block.sla_met(100.1));
    }

    #[test]
    fn test_block_coverage_below_sla() {
        let mut block = BlockCoverage::new("2001:db8:1234::/48", 1000);

        for _ in 0..800 {
            block.record_probe(true);
        }

        assert!((block.coverage_pct - 80.0).abs() < 0.01);
        assert!(!block.sla_met(90.0));
    }

    #[test]
    fn test_block_anchors() {
        let mut block = BlockCoverage::new("2001:db8::/48", 500);
        block.set_anchors(3, 0.75);

        assert_eq!(block.anchor_count, 3);
        assert!((block.anchor_redundancy - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_sla_monitor_coverage_alert() {
        let mut monitor = CoverageSlaMonitor::new();
        monitor.register_block("2001:db8:a::/48", 1000);
        monitor.register_block("2001:db8:b::/48", 1000);

        // Block a: 95% coverage (OK)
        for _ in 0..950 {
            monitor.record_probe("2001:db8:a::/48", true);
        }
        // Block b: 80% coverage (FAIL)
        for _ in 0..800 {
            monitor.record_probe("2001:db8:b::/48", true);
        }

        let alerts = monitor.check_sla();
        // Block b has CoverageBelowThreshold + NoAnchors + LowRedundancy
        let coverage_alerts: Vec<_> = alerts
            .iter()
            .filter(|a| a.alert_type == SlaAlertType::CoverageBelowThreshold)
            .collect();
        assert_eq!(coverage_alerts.len(), 1);
        assert_eq!(coverage_alerts[0].prefix, "2001:db8:b::/48");
    }

    #[test]
    fn test_sla_monitor_no_anchors_alert() {
        let mut monitor = CoverageSlaMonitor::new();
        monitor.register_block("2001:db8::/48", 1000);

        for _ in 0..1000 {
            monitor.record_probe("2001:db8::/48", true);
        }
        // No anchors set

        let alerts = monitor.check_sla();
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == SlaAlertType::NoAnchors));
    }

    #[test]
    fn test_sla_monitor_low_redundancy_alert() {
        let mut monitor = CoverageSlaMonitor::with_thresholds(90.0, 0.5);
        monitor.register_block("2001:db8::/48", 1000);

        for _ in 0..1000 {
            monitor.record_probe("2001:db8::/48", true);
        }
        monitor.set_block_anchors("2001:db8::/48", 2, 0.2);

        let alerts = monitor.check_sla();
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == SlaAlertType::LowRedundancy));
    }

    #[test]
    fn test_sla_status_healthy() {
        let mut monitor = CoverageSlaMonitor::new();
        for i in 0..10 {
            let prefix = format!("2001:db8:{:x}::/48", i);
            monitor.register_block(&prefix, 1000);
            for _ in 0..950 {
                monitor.record_probe(&prefix, true);
            }
            monitor.set_block_anchors(&prefix, 2, 0.6);
        }

        let status = monitor.get_sla_status();
        assert!(status.is_healthy());
        assert!((status.compliance_pct - 100.0).abs() < 0.01);
        assert!((status.avg_coverage_pct - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_sla_status_summary() {
        let mut monitor = CoverageSlaMonitor::new();
        monitor.register_block("2001:db8::/48", 1000);
        for _ in 0..900 {
            monitor.record_probe("2001:db8::/48", true);
        }
        monitor.set_block_anchors("2001:db8::/48", 1, 0.5);

        let status = monitor.get_sla_status();
        let summary = status.summary();
        assert!(summary.contains("1/1"));
        assert!(summary.contains("90.0%"));
    }

    #[test]
    fn test_sla_monitor_reset() {
        let mut monitor = CoverageSlaMonitor::new();
        monitor.register_block("2001:db8::/48", 1000);
        for _ in 0..500 {
            monitor.record_probe("2001:db8::/48", true);
        }
        monitor.check_sla();

        monitor.reset();
        assert!(monitor.get_block_coverages().is_empty());
        assert!(monitor.get_alerts().is_empty());
    }

    #[test]
    fn test_default_trait() {
        let monitor = CoverageSlaMonitor::default();
        assert_eq!(monitor.sla_threshold, 90.0);
        assert_eq!(monitor.min_redundancy, 0.3);
    }
}
