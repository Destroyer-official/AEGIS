//! Website Fingerprinting (WF) classifier stub (§6.4).
//!
//! Analyzes encrypted traffic patterns — packet sizes, timing, and
//! sequencing — to infer the specific website or application being
//! served over an encrypted connection, without decrypting anything.
//!
//! WF operates on the principle that different web applications produce
//! distinguishable traffic patterns even when encrypted, enabling
//! classification of target services based purely on network-layer
//! observations.
//!
//! This module defines the feature vector and classifier trait.
//! The actual ML classifier implementation is deferred to Phase 5.

// ─── Feature Vector ──────────────────────────────────────────────────────────

/// Feature vector for website fingerprinting classification.
///
/// Captures the observable characteristics of encrypted traffic flows
/// without accessing decrypted payload content.
#[derive(Clone, Debug, Default)]
pub struct WfFeatureVector {
    /// Packet sizes in bytes (positive = client→server, negative = server→client)
    pub packet_sizes: Vec<i32>,
    /// Inter-packet timing in microseconds
    pub packet_timings: Vec<u64>,
    /// Packet direction flags (true = outgoing, false = incoming)
    pub packet_directions: Vec<bool>,
    /// Total flow duration in microseconds
    pub flow_duration_us: u64,
    /// Total bytes sent (client → server)
    pub bytes_sent: u64,
    /// Total bytes received (server → client)
    pub bytes_received: u64,
    /// Number of packets in the flow
    pub packet_count: usize,
}

impl WfFeatureVector {
    /// Create a new empty feature vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a packet observation to the feature vector.
    pub fn add_packet(&mut self, size: u32, timing_us: u64, outgoing: bool) {
        let signed_size = if outgoing {
            size as i32
        } else {
            -(size as i32)
        };
        self.packet_sizes.push(signed_size);
        self.packet_timings.push(timing_us);
        self.packet_directions.push(outgoing);

        if outgoing {
            self.bytes_sent += size as u64;
        } else {
            self.bytes_received += size as u64;
        }
        self.packet_count += 1;
        self.flow_duration_us = timing_us;
    }

    /// Compute basic statistical features for classification.
    pub fn compute_stats(&self) -> WfStats {
        if self.packet_sizes.is_empty() {
            return WfStats::default();
        }

        let sizes: Vec<f64> = self.packet_sizes.iter().map(|s| *s as f64).collect();
        let mean = sizes.iter().sum::<f64>() / sizes.len() as f64;
        let variance = sizes.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / sizes.len() as f64;
        let std_dev = variance.sqrt();

        let outgoing_count = self.packet_directions.iter().filter(|&&d| d).count();
        let incoming_count = self.packet_directions.len() - outgoing_count;

        let timing_mean = if self.packet_timings.is_empty() {
            0.0
        } else {
            self.packet_timings.iter().map(|&t| t as f64).sum::<f64>()
                / self.packet_timings.len() as f64
        };

        WfStats {
            mean_packet_size: mean,
            std_dev_packet_size: std_dev,
            outgoing_ratio: outgoing_count as f64 / self.packet_directions.len() as f64,
            incoming_ratio: incoming_count as f64 / self.packet_directions.len() as f64,
            mean_inter_packet_time: timing_mean,
            total_bytes_sent: self.bytes_sent,
            total_bytes_received: self.bytes_received,
            packet_count: self.packet_count,
        }
    }
}

/// Statistical summary of a WF feature vector.
#[derive(Clone, Debug, Default)]
pub struct WfStats {
    pub mean_packet_size: f64,
    pub std_dev_packet_size: f64,
    pub outgoing_ratio: f64,
    pub incoming_ratio: f64,
    pub mean_inter_packet_time: f64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub packet_count: usize,
}

// ─── Classifier Trait ────────────────────────────────────────────────────────

/// Prediction result from a WF classifier.
#[derive(Clone, Debug)]
pub struct WfPrediction {
    /// Predicted label (e.g., "video_streaming", "web_browsing", "file_transfer")
    pub label: String,
    /// Classification confidence (0.0–1.0)
    pub confidence: f64,
}

/// Trait for website fingerprinting classifiers.
///
/// Implementations analyze encrypted traffic feature vectors and
/// predict the application or website category. The actual ML
/// implementation (Random Forest, CNN, etc.) is deferred to Phase 5.
pub trait WfClassifier {
    /// Classify a feature vector and return a prediction.
    fn classify(&self, features: &WfFeatureVector) -> WfPrediction;
}

/// Stub classifier that always returns unknown.
///
/// This serves as a placeholder until the ML classifier is implemented
/// in Phase 5. It demonstrates the trait interface and can be used
/// for integration testing.
pub struct StubWfClassifier;

impl WfClassifier for StubWfClassifier {
    fn classify(&self, _features: &WfFeatureVector) -> WfPrediction {
        WfPrediction {
            label: "unknown".into(),
            confidence: 0.0,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_empty() {
        let fv = WfFeatureVector::new();
        assert_eq!(fv.packet_count, 0);
        assert_eq!(fv.bytes_sent, 0);
        assert_eq!(fv.bytes_received, 0);
    }

    #[test]
    fn test_feature_vector_add_packet() {
        let mut fv = WfFeatureVector::new();
        fv.add_packet(1500, 0, true);
        fv.add_packet(200, 1000, false);
        fv.add_packet(1400, 2000, true);

        assert_eq!(fv.packet_count, 3);
        assert_eq!(fv.bytes_sent, 2900);
        assert_eq!(fv.bytes_received, 200);
        assert_eq!(fv.packet_sizes, vec![1500, -200, 1400]);
        assert_eq!(fv.packet_timings, vec![0, 1000, 2000]);
        assert_eq!(fv.packet_directions, vec![true, false, true]);
    }

    #[test]
    fn test_stats_computation() {
        let mut fv = WfFeatureVector::new();
        fv.add_packet(100, 0, true);
        fv.add_packet(200, 1000, false);
        fv.add_packet(150, 2000, true);

        let stats = fv.compute_stats();
        assert!(stats.mean_packet_size > 0.0);
        assert!(stats.outgoing_ratio > 0.0);
        assert!(stats.incoming_ratio > 0.0);
        assert_eq!(stats.packet_count, 3);
    }

    #[test]
    fn test_stats_empty() {
        let fv = WfFeatureVector::new();
        let stats = fv.compute_stats();
        assert_eq!(stats.packet_count, 0);
    }

    #[test]
    fn test_stub_classifier() {
        let classifier = StubWfClassifier;
        let fv = WfFeatureVector::new();
        let pred = classifier.classify(&fv);
        assert_eq!(pred.label, "unknown");
        assert_eq!(pred.confidence, 0.0);
    }
}
