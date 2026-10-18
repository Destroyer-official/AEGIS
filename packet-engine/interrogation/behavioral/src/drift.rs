//! SSH Behavioral Drift Detection — Streaming KS-test on KEX feature distributions.
//!
//! Monitors KEX algorithm preference distributions over time and alerts
//! on statistically significant distribution shifts using the
//! Kolmogorov-Smirnov two-sample test.

use std::collections::VecDeque;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Drift detection errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DriftError {
    #[error("insufficient samples: need {min}, got {actual}")]
    InsufficientSamples { min: usize, actual: usize },

    #[error("window too small: need at least 2")]
    WindowTooSmall,
}

// ─── KS-Test ─────────────────────────────────────────────────────────────────

/// Result of a two-sample Kolmogorov-Smirnov test.
#[derive(Clone, Debug)]
pub struct KsTestResult {
    /// KS statistic (max absolute difference between CDFs)
    pub statistic: f64,
    /// p-value (approximation)
    pub p_value: f64,
    /// Whether the null hypothesis is rejected at alpha level
    pub significant: bool,
}

/// Compute the two-sample Kolmogorov-Smirnov statistic.
///
/// Given two samples, compute the maximum absolute difference between
/// their empirical CDFs.
pub fn ks_two_sample(sample_a: &[f64], sample_b: &[f64]) -> KsTestResult {
    if sample_a.is_empty() || sample_b.is_empty() {
        return KsTestResult {
            statistic: 0.0,
            p_value: 1.0,
            significant: false,
        };
    }

    let n_a = sample_a.len() as f64;
    let n_b = sample_b.len() as f64;

    // Merge and sort all values
    let mut all_values: Vec<(f64, bool)> = Vec::with_capacity(sample_a.len() + sample_b.len());
    for &v in sample_a {
        all_values.push((v, true)); // true = from sample A
    }
    for &v in sample_b {
        all_values.push((v, false)); // false = from sample B
    }
    all_values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Compute maximum difference between empirical CDFs
    let mut cdf_a = 0.0;
    let mut cdf_b = 0.0;
    let mut max_diff = 0.0;

    for &(_, from_a) in &all_values {
        if from_a {
            cdf_a += 1.0 / n_a;
        } else {
            cdf_b += 1.0 / n_b;
        }
        let diff = (cdf_a - cdf_b).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    // Approximate p-value using asymptotic distribution
    // For large samples, sqrt(n) * D converges to the KS distribution
    let n_eff = (n_a * n_b / (n_a + n_b)).sqrt();
    let lambda = (n_eff + 0.12 + 0.11 / n_eff) * max_diff;

    // Series expansion for KS p-value (Abramowitz & Stegun)
    let p_value = if lambda < 0.001 {
        1.0
    } else if lambda < 1.18 {
        let q = (-2.0 * lambda * lambda).exp();
        1.0 - q
            * (2.0 / 3.0 * (-0.3316 * lambda * lambda).exp()
                + 0.343 * (-1.216 * lambda * lambda).exp()
                + 0.0042 * (-5.24 * lambda * lambda).exp()
                - 0.0003 * (-16.3 * lambda * lambda).exp())
    } else {
        // Large lambda approximation
        let q = (-2.0 * (lambda - 0.12 + 0.11 / lambda).powi(2)).exp();
        q * (1.0 - 2.0 / (9.0 * lambda) + 2.0 / (81.0 * lambda * lambda)).sqrt()
    };

    KsTestResult {
        statistic: max_diff,
        p_value: p_value.clamp(0.0, 1.0),
        significant: p_value < 0.05,
    }
}

// ─── Drift Detector ──────────────────────────────────────────────────────────

/// Configuration for drift detection.
#[derive(Clone, Debug)]
pub struct DriftConfig {
    /// Minimum number of samples before drift can be detected
    pub min_samples: usize,
    /// Window size for baseline (number of recent observations)
    pub baseline_window: usize,
    /// Window size for comparison (number of recent observations)
    pub comparison_window: usize,
    /// Significance level (alpha) for KS-test
    pub alpha: f64,
    /// Feature index to monitor (0-7 for KEX feature vector)
    pub feature_index: usize,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            min_samples: 50,
            baseline_window: 1000,
            comparison_window: 200,
            alpha: 0.05,
            feature_index: 0, // KEX algorithm preference
        }
    }
}

/// Alert generated when drift is detected.
#[derive(Clone, Debug)]
pub struct DriftAlert {
    /// Feature that drifted
    pub feature_name: String,
    /// KS-test result
    pub ks_result: KsTestResult,
    /// Baseline statistics
    pub baseline_mean: f64,
    pub baseline_std: f64,
    /// Comparison statistics
    pub comparison_mean: f64,
    pub comparison_std: f64,
    /// Timestamp
    pub timestamp: u64,
}

impl std::fmt::Display for DriftAlert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DRIFT ALERT: {} | KS={:.4} p={:.4} | baseline={:.2}±{:.2} -> comparison={:.2}±{:.2}",
            self.feature_name,
            self.ks_result.statistic,
            self.ks_result.p_value,
            self.baseline_mean,
            self.baseline_std,
            self.comparison_mean,
            self.comparison_std,
        )
    }
}

/// Streaming drift detector for SSH behavioral features.
pub struct DriftDetector {
    /// Configuration
    config: DriftConfig,
    /// Baseline buffer (circular)
    baseline: VecDeque<f64>,
    /// Comparison buffer (circular)
    comparison: VecDeque<f64>,
    /// Total observations received
    total_observations: usize,
}

impl DriftDetector {
    /// Create a new drift detector with the given configuration.
    pub fn new(config: DriftConfig) -> Self {
        Self {
            config,
            baseline: VecDeque::new(),
            comparison: VecDeque::new(),
            total_observations: 0,
        }
    }

    /// Create a detector with default configuration monitoring KEX preference.
    pub fn for_kex_preference() -> Self {
        Self::new(DriftConfig {
            feature_index: 0,
            ..Default::default()
        })
    }

    /// Create a detector with default configuration monitoring host key type.
    pub fn for_host_key_type() -> Self {
        Self::new(DriftConfig {
            feature_index: 1,
            ..Default::default()
        })
    }

    /// Add a feature value to the detector.
    pub fn add_observation(&mut self, value: f64) {
        self.comparison.push_back(value);
        self.total_observations += 1;

        // Trim comparison window
        while self.comparison.len() > self.config.comparison_window {
            self.comparison.pop_front();
        }
    }

    /// Check for drift by comparing baseline vs comparison windows.
    ///
    /// Returns `Some(DriftAlert)` if drift is detected, `None` otherwise.
    pub fn check_drift(&mut self) -> Option<DriftAlert> {
        // Need minimum samples
        if self.total_observations < self.config.min_samples {
            return None;
        }

        // Need both windows to have data
        if self.baseline.len() < 2 || self.comparison.len() < 2 {
            return None;
        }

        // Run KS-test
        let baseline_vec: Vec<f64> = self.baseline.iter().copied().collect();
        let comparison_vec: Vec<f64> = self.comparison.iter().copied().collect();

        let ks_result = ks_two_sample(&baseline_vec, &comparison_vec);

        if !ks_result.significant {
            return None;
        }

        // Compute statistics
        let baseline_mean = mean(&baseline_vec);
        let baseline_std = std_dev(&baseline_vec, baseline_mean);
        let comparison_mean = mean(&comparison_vec);
        let comparison_std = std_dev(&comparison_vec, comparison_mean);

        let feature_name = match self.config.feature_index {
            0 => "kex_preference",
            1 => "host_key_type",
            2 => "kex_algo_count",
            3 => "host_key_algo_count",
            4 => "cipher_count",
            5 => "supports_ed25519",
            6 => "offers_dh_group14_sha1",
            7 => "unimpl_total_length",
            _ => "unknown",
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Some(DriftAlert {
            feature_name: feature_name.to_string(),
            ks_result,
            baseline_mean,
            baseline_std,
            comparison_mean,
            comparison_std,
            timestamp,
        })
    }

    /// Promote comparison window to baseline (after confirmed drift or periodically).
    pub fn promote_comparison(&mut self) {
        self.baseline = self.comparison.clone();
    }

    /// Clear both windows and reset.
    pub fn reset(&mut self) {
        self.baseline.clear();
        self.comparison.clear();
        self.total_observations = 0;
    }

    /// Get current buffer sizes.
    pub fn buffer_sizes(&self) -> (usize, usize) {
        (self.baseline.len(), self.comparison.len())
    }

    /// Get total observations.
    pub fn total_observations(&self) -> usize {
        self.total_observations
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ks_same_distribution() {
        // Same distribution should not be significant
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let b = vec![1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1];
        let result = ks_two_sample(&a, &b);
        assert!(
            !result.significant,
            "Same distributions should not be significant"
        );
    }

    #[test]
    fn test_ks_different_distributions() {
        // Clearly different distributions
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = ks_two_sample(&a, &b);
        assert!(
            result.significant,
            "Different distributions should be significant"
        );
        assert!(result.statistic > 0.5);
    }

    #[test]
    fn test_ks_empty_samples() {
        let a = vec![];
        let b = vec![1.0, 2.0];
        let result = ks_two_sample(&a, &b);
        assert_eq!(result.statistic, 0.0);
        assert!(!result.significant);
    }

    #[test]
    fn test_drift_detector_no_drift() {
        let mut detector = DriftDetector::new(DriftConfig {
            min_samples: 10,
            baseline_window: 100,
            comparison_window: 100,
            alpha: 0.05,
            feature_index: 0,
        });

        // Add stable observations
        for _ in 0..100 {
            detector.add_observation(5.0 + rand_f64() * 0.1);
        }
        detector.promote_comparison();

        // Add more stable observations
        for _ in 0..50 {
            detector.add_observation(5.0 + rand_f64() * 0.1);
        }

        let alert = detector.check_drift();
        assert!(alert.is_none(), "Stable data should not trigger drift");
    }

    #[test]
    fn test_drift_detector_detects_drift() {
        let mut detector = DriftDetector::new(DriftConfig {
            min_samples: 10,
            baseline_window: 100,
            comparison_window: 100,
            alpha: 0.05,
            feature_index: 0,
        });

        // Baseline: centered around 5
        for _ in 0..100 {
            detector.add_observation(5.0 + rand_f64() * 0.5);
        }
        detector.promote_comparison();

        // Comparison: shifted to 15
        for _ in 0..100 {
            detector.add_observation(15.0 + rand_f64() * 0.5);
        }

        let alert = detector.check_drift();
        assert!(alert.is_some(), "Shifted data should trigger drift");
    }

    #[test]
    fn test_drift_detector_min_samples() {
        let mut detector = DriftDetector::new(DriftConfig {
            min_samples: 100,
            ..Default::default()
        });

        // Add fewer than min_samples
        for _ in 0..50 {
            detector.add_observation(1.0);
        }

        let alert = detector.check_drift();
        assert!(
            alert.is_none(),
            "Should not alert with insufficient samples"
        );
    }

    #[test]
    fn test_drift_alert_display() {
        let alert = DriftAlert {
            feature_name: "kex_preference".into(),
            ks_result: KsTestResult {
                statistic: 0.45,
                p_value: 0.001,
                significant: true,
            },
            baseline_mean: 5.0,
            baseline_std: 0.5,
            comparison_mean: 15.0,
            comparison_std: 0.5,
            timestamp: 1234567890,
        };

        let display = alert.to_string();
        assert!(display.contains("DRIFT ALERT"));
        assert!(display.contains("kex_preference"));
    }

    #[test]
    fn test_promote_comparison() {
        let mut detector = DriftDetector::new(DriftConfig::default());

        for _ in 0..10 {
            detector.add_observation(1.0);
        }

        let (baseline, comparison) = detector.buffer_sizes();
        assert_eq!(baseline, 0);
        assert_eq!(comparison, 10);

        detector.promote_comparison();
        let (baseline, comparison) = detector.buffer_sizes();
        assert_eq!(baseline, 10);
        assert_eq!(comparison, 10);
    }

    #[test]
    fn test_reset() {
        let mut detector = DriftDetector::new(DriftConfig::default());

        for _ in 0..10 {
            detector.add_observation(1.0);
        }

        detector.reset();
        let (baseline, comparison) = detector.buffer_sizes();
        assert_eq!(baseline, 0);
        assert_eq!(comparison, 0);
        assert_eq!(detector.total_observations(), 0);
    }

    #[test]
    fn test_for_kex_preference() {
        let detector = DriftDetector::for_kex_preference();
        assert_eq!(detector.config.feature_index, 0);
    }

    #[test]
    fn test_for_host_key_type() {
        let detector = DriftDetector::for_host_key_type();
        assert_eq!(detector.config.feature_index, 1);
    }

    #[test]
    fn test_mean() {
        assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-10);
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_std_dev() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let m = mean(&values);
        let sd = std_dev(&values, m);
        // Population std dev of this dataset is ~2.0
        assert!(sd > 1.5 && sd < 2.5, "std_dev should be ~2.0, got {}", sd);
    }

    fn rand_f64() -> f64 {
        // Simple deterministic pseudo-random for testing
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        (hash % 10000) as f64 / 10000.0
    }
}
