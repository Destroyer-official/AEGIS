use crate::{BgpPrefix, CtCertificate, PassiveDnsRecord};
use serde::{Deserialize, Serialize};

/// Quality score components for a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub freshness: f64,
    pub completeness: f64,
    pub consistency: f64,
    pub volume: f64,
    pub composite: f64,
}

impl QualityScore {
    pub fn new(freshness: f64, completeness: f64, consistency: f64, volume: f64) -> Self {
        Self::with_weights(
            freshness,
            completeness,
            consistency,
            volume,
            &QualityWeights::default(),
        )
    }

    pub fn with_weights(
        freshness: f64,
        completeness: f64,
        consistency: f64,
        volume: f64,
        weights: &QualityWeights,
    ) -> Self {
        let composite = weights.freshness * freshness
            + weights.completeness * completeness
            + weights.consistency * consistency
            + weights.volume * volume;
        Self {
            freshness,
            completeness,
            consistency,
            volume,
            composite,
        }
    }

    pub fn passes_threshold(&self, threshold: f64) -> bool {
        self.composite >= threshold
    }
}

/// Configuration for quality scoring thresholds and weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    pub freshness_window_days: u32,
    pub min_records_for_volume: usize,
    pub default_threshold: f64,
    pub weights: QualityWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    pub freshness: f64,
    pub completeness: f64,
    pub consistency: f64,
    pub volume: f64,
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            freshness: 0.3,
            completeness: 0.25,
            consistency: 0.25,
            volume: 0.2,
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            freshness_window_days: 30,
            min_records_for_volume: 100,
            default_threshold: 0.5,
            weights: QualityWeights {
                freshness: 0.3,
                completeness: 0.25,
                consistency: 0.25,
                volume: 0.2,
            },
        }
    }
}

/// Filter result after quality scoring.
#[derive(Debug, Clone)]
pub struct QualityFilterResult<T> {
    pub passed: Vec<T>,
    pub rejected: Vec<T>,
    pub scores: Vec<QualityScore>,
}

// ─── BGP Quality ──────────────────────────────────────────────────

pub struct BgpQualityScorer {
    config: QualityConfig,
}

impl BgpQualityScorer {
    pub fn new(config: QualityConfig) -> Self {
        Self { config }
    }

    pub fn score_prefixes(&self, prefixes: &[BgpPrefix]) -> QualityFilterResult<BgpPrefix> {
        let mut passed = Vec::new();
        let mut rejected = Vec::new();
        let mut scores = Vec::new();

        let total = prefixes.len() as f64;

        for prefix in prefixes {
            let completeness = if prefix.as_path.is_empty() { 0.0 } else { 1.0 };
            let consistency = 1.0; // BGP data is inherently consistent
            let volume = (total / self.config.min_records_for_volume as f64).min(1.0);
            let freshness = 1.0; // BGP dumps are always "fresh" within the window

            let score = QualityScore::with_weights(
                freshness,
                completeness,
                consistency,
                volume,
                &self.config.weights,
            );
            scores.push(score.clone());

            if score.passes_threshold(self.config.default_threshold) {
                passed.push(prefix.clone());
            } else {
                rejected.push(prefix.clone());
            }
        }

        QualityFilterResult {
            passed,
            rejected,
            scores,
        }
    }
}

// ─── Passive DNS Quality ──────────────────────────────────────────

pub struct PassiveDnsQualityScorer {
    config: QualityConfig,
}

impl PassiveDnsQualityScorer {
    pub fn new(config: QualityConfig) -> Self {
        Self { config }
    }

    pub fn score_records(
        &self,
        records: &[PassiveDnsRecord],
    ) -> QualityFilterResult<PassiveDnsRecord> {
        let mut passed = Vec::new();
        let mut rejected = Vec::new();
        let mut scores = Vec::new();

        for record in records {
            let has_ip = !record.ip_address.is_empty();
            let has_domain = !record.domain.is_empty();
            let has_ttl = record.ttl.is_some();
            let has_times = !record.first_seen.is_empty() && !record.last_seen.is_empty();

            let completeness = [has_ip, has_domain, has_ttl, has_times]
                .iter()
                .filter(|&&x| x)
                .count() as f64
                / 4.0;

            let freshness = if has_times {
                // Rough heuristic: score higher if last_seen is recent.
                // Since we don't have real-time access, just check non-empty.
                0.8
            } else {
                0.0
            };

            let consistency = 1.0; // Passive DNS is inherently consistent
            let volume = 0.5; // Default volume score

            let score = QualityScore::with_weights(
                freshness,
                completeness,
                consistency,
                volume,
                &self.config.weights,
            );
            scores.push(score.clone());

            if score.passes_threshold(self.config.default_threshold) {
                passed.push(record.clone());
            } else {
                rejected.push(record.clone());
            }
        }

        QualityFilterResult {
            passed,
            rejected,
            scores,
        }
    }
}

// ─── CT Log Quality ───────────────────────────────────────────────

pub struct CtQualityScorer {
    config: QualityConfig,
}

impl CtQualityScorer {
    pub fn new(config: QualityConfig) -> Self {
        Self { config }
    }

    pub fn score_certificates(
        &self,
        certs: &[CtCertificate],
    ) -> QualityFilterResult<CtCertificate> {
        let mut passed = Vec::new();
        let mut rejected = Vec::new();
        let mut scores = Vec::new();

        for cert in certs {
            let has_issuer = !cert.issuer_ca.is_empty();
            let has_cn = !cert.common_name.is_empty();
            let has_sans = !cert.name_value.is_empty();

            let completeness = [has_issuer, has_cn, has_sans]
                .iter()
                .filter(|&&x| x)
                .count() as f64
                / 3.0;

            let freshness = 0.7; // CT logs are generally fresh
            let consistency = 1.0;
            let volume = 0.5;

            let score = QualityScore::with_weights(
                freshness,
                completeness,
                consistency,
                volume,
                &self.config.weights,
            );
            scores.push(score.clone());

            if score.passes_threshold(self.config.default_threshold) {
                passed.push(cert.clone());
            } else {
                rejected.push(cert.clone());
            }
        }

        QualityFilterResult {
            passed,
            rejected,
            scores,
        }
    }
}

// ─── Unified Quality Pipeline ─────────────────────────────────────

pub struct QualityPipeline {
    bgp_scorer: BgpQualityScorer,
    dns_scorer: PassiveDnsQualityScorer,
    ct_scorer: CtQualityScorer,
}

impl QualityPipeline {
    pub fn new(config: QualityConfig) -> Self {
        Self {
            bgp_scorer: BgpQualityScorer::new(config.clone()),
            dns_scorer: PassiveDnsQualityScorer::new(config.clone()),
            ct_scorer: CtQualityScorer::new(config.clone()),
        }
    }

    pub fn filter_bgp(&self, prefixes: &[BgpPrefix]) -> QualityFilterResult<BgpPrefix> {
        self.bgp_scorer.score_prefixes(prefixes)
    }

    pub fn filter_dns(
        &self,
        records: &[PassiveDnsRecord],
    ) -> QualityFilterResult<PassiveDnsRecord> {
        self.dns_scorer.score_records(records)
    }

    pub fn filter_ct(&self, certs: &[CtCertificate]) -> QualityFilterResult<CtCertificate> {
        self.ct_scorer.score_certificates(certs)
    }

    /// Compute aggregate quality stats across all sources.
    pub fn aggregate_stats(
        &self,
        bgp_result: &QualityFilterResult<BgpPrefix>,
        dns_result: &QualityFilterResult<PassiveDnsRecord>,
        ct_result: &QualityFilterResult<CtCertificate>,
    ) -> AggregateQualityStats {
        let total_scores =
            bgp_result.scores.len() + dns_result.scores.len() + ct_result.scores.len();

        let avg_composite = if total_scores > 0 {
            let sum: f64 = bgp_result.scores.iter().map(|s| s.composite).sum::<f64>()
                + dns_result.scores.iter().map(|s| s.composite).sum::<f64>()
                + ct_result.scores.iter().map(|s| s.composite).sum::<f64>();
            sum / total_scores as f64
        } else {
            0.0
        };

        AggregateQualityStats {
            bgp_passed: bgp_result.passed.len(),
            bgp_rejected: bgp_result.rejected.len(),
            dns_passed: dns_result.passed.len(),
            dns_rejected: dns_result.rejected.len(),
            ct_passed: ct_result.passed.len(),
            ct_rejected: ct_result.rejected.len(),
            avg_composite_score: avg_composite,
        }
    }
}

impl Default for QualityPipeline {
    fn default() -> Self {
        Self::new(QualityConfig::default())
    }
}

/// Aggregate quality statistics across all data sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateQualityStats {
    pub bgp_passed: usize,
    pub bgp_rejected: usize,
    pub dns_passed: usize,
    pub dns_rejected: usize,
    pub ct_passed: usize,
    pub ct_rejected: usize,
    pub avg_composite_score: f64,
}

// ─── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_composite() {
        let score = QualityScore::new(1.0, 0.8, 0.9, 0.7);
        let expected = 0.3 * 1.0 + 0.25 * 0.8 + 0.25 * 0.9 + 0.2 * 0.7;
        assert!((score.composite - expected).abs() < 1e-10);
    }

    #[test]
    fn test_quality_score_threshold() {
        let high = QualityScore::new(1.0, 1.0, 1.0, 1.0);
        assert!(high.passes_threshold(0.5));

        let low = QualityScore::new(0.0, 0.0, 0.0, 0.0);
        assert!(!low.passes_threshold(0.5));
    }

    #[test]
    fn test_bgp_quality_all_pass() {
        let scorer = BgpQualityScorer::new(QualityConfig::default());
        let prefixes = vec![
            BgpPrefix {
                prefix: "10.0.0.0/8".to_string(),
                asn: 16509,
                as_path: vec![16509, 1299],
                origin_asn: 16509,
                roa_valid: Some(true),
            },
            BgpPrefix {
                prefix: "192.168.0.0/16".to_string(),
                asn: 15169,
                as_path: vec![15169],
                origin_asn: 15169,
                roa_valid: None,
            },
        ];

        let result = scorer.score_prefixes(&prefixes);
        assert_eq!(result.passed.len(), 2);
        assert_eq!(result.rejected.len(), 0);
    }

    #[test]
    fn test_bgp_quality_empty_path_rejected() {
        let config = QualityConfig {
            default_threshold: 0.8,
            ..Default::default()
        };
        let scorer = BgpQualityScorer::new(config);
        let prefixes = vec![BgpPrefix {
            prefix: "10.0.0.0/8".to_string(),
            asn: 16509,
            as_path: vec![],
            origin_asn: 16509,
            roa_valid: None,
        }];

        let result = scorer.score_prefixes(&prefixes);
        assert_eq!(result.passed.len(), 0);
        assert_eq!(result.rejected.len(), 1);
    }

    #[test]
    fn test_dns_quality() {
        let scorer = PassiveDnsQualityScorer::new(QualityConfig::default());
        let records = vec![
            PassiveDnsRecord {
                domain: "example.com".to_string(),
                ip_address: "93.184.216.34".to_string(),
                record_type: "A".to_string(),
                first_seen: "2024-01-01T00:00:00Z".to_string(),
                last_seen: "2026-08-29T00:00:00Z".to_string(),
                ttl: Some(3600),
            },
            PassiveDnsRecord {
                domain: "".to_string(),
                ip_address: "1.2.3.4".to_string(),
                record_type: "A".to_string(),
                first_seen: "".to_string(),
                last_seen: "".to_string(),
                ttl: None,
            },
        ];

        let result = scorer.score_records(&records);
        assert_eq!(result.passed.len(), 1);
        assert_eq!(result.rejected.len(), 1);
    }

    #[test]
    fn test_ct_quality() {
        let config = QualityConfig {
            default_threshold: 0.8,
            ..Default::default()
        };
        let scorer = CtQualityScorer::new(config);
        let certs = vec![
            CtCertificate {
                issuer_ca: "Let's Encrypt".to_string(),
                common_name: "example.com".to_string(),
                name_value: "example.com\nwww.example.com".to_string(),
            },
            CtCertificate {
                issuer_ca: "".to_string(),
                common_name: "".to_string(),
                name_value: "".to_string(),
            },
        ];

        let result = scorer.score_certificates(&certs);
        assert_eq!(result.passed.len(), 1);
        assert_eq!(result.rejected.len(), 1);
    }

    #[test]
    fn test_quality_pipeline() {
        let pipeline = QualityPipeline::new(QualityConfig::default());

        let bgp = vec![BgpPrefix {
            prefix: "10.0.0.0/8".to_string(),
            asn: 16509,
            as_path: vec![16509],
            origin_asn: 16509,
            roa_valid: Some(true),
        }];

        let dns = vec![PassiveDnsRecord {
            domain: "example.com".to_string(),
            ip_address: "93.184.216.34".to_string(),
            record_type: "A".to_string(),
            first_seen: "2024-01-01T00:00:00Z".to_string(),
            last_seen: "2026-08-29T00:00:00Z".to_string(),
            ttl: Some(3600),
        }];

        let ct = vec![CtCertificate {
            issuer_ca: "Let's Encrypt".to_string(),
            common_name: "example.com".to_string(),
            name_value: "example.com".to_string(),
        }];

        let bgp_result = pipeline.filter_bgp(&bgp);
        let dns_result = pipeline.filter_dns(&dns);
        let ct_result = pipeline.filter_ct(&ct);

        let stats = pipeline.aggregate_stats(&bgp_result, &dns_result, &ct_result);
        assert_eq!(stats.bgp_passed, 1);
        assert_eq!(stats.dns_passed, 1);
        assert_eq!(stats.ct_passed, 1);
        assert!(stats.avg_composite_score > 0.0);
    }

    #[test]
    fn test_default_config() {
        let config = QualityConfig::default();
        assert_eq!(config.freshness_window_days, 30);
        assert_eq!(config.min_records_for_volume, 100);
        assert!((config.default_threshold - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_aggregate_stats_empty() {
        let pipeline = QualityPipeline::new(QualityConfig::default());
        let bgp_result = pipeline.filter_bgp(&[]);
        let dns_result = pipeline.filter_dns(&[]);
        let ct_result = pipeline.filter_ct(&[]);

        let stats = pipeline.aggregate_stats(&bgp_result, &dns_result, &ct_result);
        assert_eq!(stats.bgp_passed, 0);
        assert_eq!(stats.avg_composite_score, 0.0);
    }
}
