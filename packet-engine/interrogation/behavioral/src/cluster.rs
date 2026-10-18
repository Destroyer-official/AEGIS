//! SSH behavioral clustering — K-means on KEX preferences → threat actor clusters.
//!
//! Clusters SSH behavioral fingerprints by their KEX algorithm preferences
//! and other behavioral features to identify threat actor groups with
//! similar SSH server configurations.

use crate::ssh::{SshBehavioralFingerprint, SshHostKeyType, SshKexAlgo};

/// Feature vector extracted from an SSH behavioral fingerprint.
#[derive(Clone, Debug)]
pub struct SshFeatureVector {
    /// KEX algorithm preference as a numeric index
    pub kex_pref: u8,
    /// Host key type as a numeric index
    pub host_key_type: u8,
    /// Number of KEX algorithms offered
    pub kex_algo_count: f64,
    /// Number of host key algorithms offered
    pub host_key_algo_count: f64,
    /// Number of cipher suites offered
    pub cipher_count: f64,
    /// Supports Ed25519 (0.0 or 1.0)
    pub supports_ed25519: f64,
    /// Offers DH group14 SHA1 (0.0 or 1.0)
    pub offers_dh_group14_sha1: f64,
    /// UNIMPLEMENTED frame total length (0.0 if not available)
    pub unimpl_total_length: f64,
}

impl SshFeatureVector {
    /// Extract feature vector from a behavioral fingerprint.
    pub fn from_fingerprint(fp: &SshBehavioralFingerprint) -> Self {
        let kex_pref = match fp.kex_preference {
            SshKexAlgo::Curve25519Sha256 => 1,
            SshKexAlgo::Curve25519Sha256AtLibsshOrg => 1,
            SshKexAlgo::EcdhSha2Nistp256 => 2,
            SshKexAlgo::EcdhSha2Nistp384 => 3,
            SshKexAlgo::EcdhSha2Nistp521 => 4,
            SshKexAlgo::DiffieHellmanGroup14Sha256 => 5,
            SshKexAlgo::DiffieHellmanGroup16Sha512 => 6,
            SshKexAlgo::DiffieHellmanGroup18Sha512 => 6,
            SshKexAlgo::DiffieHellmanGroup14Sha1 => 7,
            SshKexAlgo::Unknown(_) => 0,
        };

        let hk = match fp.host_key_type {
            SshHostKeyType::Ed25519 => 1,
            SshHostKeyType::RsaSha2_256 => 2,
            SshHostKeyType::RsaSha2_512 => 2,
            SshHostKeyType::RsaSha1 => 2,
            SshHostKeyType::EcdsaSha2Nistp256 => 3,
            SshHostKeyType::EcdsaSha2Nistp384 => 4,
            SshHostKeyType::EcdsaSha2Nistp521 => 5,
            SshHostKeyType::Dss => 6,
            SshHostKeyType::Unknown(_) => 0,
        };

        let unimpl_len = fp
            .unimplemented_metrics
            .as_ref()
            .map(|m| m.total_length as f64)
            .unwrap_or(0.0);

        SshFeatureVector {
            kex_pref,
            host_key_type: hk,
            kex_algo_count: fp.kex_algo_count as f64,
            host_key_algo_count: fp.host_key_algo_count as f64,
            cipher_count: fp.cipher_count as f64,
            supports_ed25519: if fp.supports_ed25519 { 1.0 } else { 0.0 },
            offers_dh_group14_sha1: if fp.offers_dh_group14_sha1 { 1.0 } else { 0.0 },
            unimpl_total_length: unimpl_len,
        }
    }

    /// Euclidean distance to another feature vector.
    pub fn distance(&self, other: &SshFeatureVector) -> f64 {
        let sum = (self.kex_pref as f64 - other.kex_pref as f64).powi(2)
            + (self.host_key_type as f64 - other.host_key_type as f64).powi(2)
            + (self.kex_algo_count - other.kex_algo_count).powi(2)
            + (self.host_key_algo_count - other.host_key_algo_count).powi(2)
            + (self.cipher_count - other.cipher_count).powi(2)
            + (self.supports_ed25519 - other.supports_ed25519).powi(2)
            + (self.offers_dh_group14_sha1 - other.offers_dh_group14_sha1).powi(2)
            + (self.unimpl_total_length - other.unimpl_total_length).powi(2);
        sum.sqrt()
    }
}

/// A cluster of SSH behavioral fingerprints.
#[derive(Clone, Debug)]
pub struct Cluster {
    /// Cluster ID
    pub id: usize,
    /// Centroid of the cluster
    pub centroid: SshFeatureVector,
    /// Indices of fingerprints in this cluster
    pub members: Vec<usize>,
}

/// K-means clustering for SSH behavioral fingerprints.
#[derive(Clone, Debug)]
pub struct SshClusterer {
    /// Number of clusters
    k: usize,
    /// Maximum iterations
    max_iter: usize,
}

impl SshClusterer {
    /// Create a new clusterer with k clusters.
    pub fn new(k: usize) -> Self {
        Self { k, max_iter: 100 }
    }

    /// Create a new clusterer with k clusters and max iterations.
    pub fn with_max_iter(k: usize, max_iter: usize) -> Self {
        Self { k, max_iter }
    }

    /// Run K-means clustering on a set of feature vectors.
    pub fn cluster(&self, vectors: &[SshFeatureVector]) -> Vec<Cluster> {
        if vectors.is_empty() {
            return Vec::new();
        }

        let n = vectors.len();
        let k = self.k.min(n);

        // Initialize centroids by selecting k evenly-spaced vectors
        let mut centroids: Vec<SshFeatureVector> = (0..k)
            .map(|i| {
                let idx = i * (n / k);
                vectors[idx].clone()
            })
            .collect();

        let mut assignments = vec![0usize; n];

        for _ in 0..self.max_iter {
            let mut changed = false;

            // Assign each vector to nearest centroid
            for (i, v) in vectors.iter().enumerate() {
                let best = centroids
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.distance(v)
                            .partial_cmp(&b.distance(v))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);

                if assignments[i] != best {
                    assignments[i] = best;
                    changed = true;
                }
            }

            if !changed {
                break;
            }

            // Recompute centroids
            for (c, centroid) in centroids.iter_mut().enumerate() {
                let members: Vec<&SshFeatureVector> = vectors
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == c)
                    .map(|(_, v)| v)
                    .collect();

                if members.is_empty() {
                    continue;
                }

                let _dim = 8;
                let avg = |idx: usize| -> f64 {
                    members
                        .iter()
                        .map(|v| match idx {
                            0 => v.kex_pref as f64,
                            1 => v.host_key_type as f64,
                            2 => v.kex_algo_count,
                            3 => v.host_key_algo_count,
                            4 => v.cipher_count,
                            5 => v.supports_ed25519,
                            6 => v.offers_dh_group14_sha1,
                            7 => v.unimpl_total_length,
                            _ => 0.0,
                        })
                        .sum::<f64>()
                        / members.len() as f64
                };

                *centroid = SshFeatureVector {
                    kex_pref: avg(0).round() as u8,
                    host_key_type: avg(1).round() as u8,
                    kex_algo_count: avg(2),
                    host_key_algo_count: avg(3),
                    cipher_count: avg(4),
                    supports_ed25519: avg(5),
                    offers_dh_group14_sha1: avg(6),
                    unimpl_total_length: avg(7),
                };
            }
        }

        // Build clusters
        (0..k)
            .map(|c| {
                let members: Vec<usize> = assignments
                    .iter()
                    .enumerate()
                    .filter(|(_, &a)| a == c)
                    .map(|(i, _)| i)
                    .collect();
                Cluster {
                    id: c,
                    centroid: centroids[c].clone(),
                    members,
                }
            })
            .collect()
    }
}

/// Threat actor cluster label derived from clustering results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreatActorCluster {
    /// Modern config (Curve25519, Ed25519, no legacy KEX)
    Modern,
    /// Legacy config (DH group14-sha1, RSA, no Ed25519)
    Legacy,
    /// Mixed config (has both modern and legacy features)
    Mixed,
    /// Unknown or unclassifiable
    Unknown,
}

impl ThreatActorCluster {
    /// Classify a cluster based on its centroid features.
    pub fn from_centroid(c: &SshFeatureVector) -> Self {
        let is_modern = c.supports_ed25519 > 0.5 && c.kex_pref >= 1 && c.kex_pref <= 4;
        let is_legacy = c.offers_dh_group14_sha1 > 0.5 || c.kex_pref == 7 || c.kex_pref == 8;

        match (is_modern, is_legacy) {
            (true, false) => ThreatActorCluster::Modern,
            (false, true) => ThreatActorCluster::Legacy,
            (true, true) => ThreatActorCluster::Mixed,
            _ => ThreatActorCluster::Unknown,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn modern_fp() -> SshFeatureVector {
        SshFeatureVector {
            kex_pref: 1,      // Curve25519
            host_key_type: 1, // Ed25519
            kex_algo_count: 12.0,
            host_key_algo_count: 6.0,
            cipher_count: 8.0,
            supports_ed25519: 1.0,
            offers_dh_group14_sha1: 0.0,
            unimpl_total_length: 0.0,
        }
    }

    fn legacy_fp() -> SshFeatureVector {
        SshFeatureVector {
            kex_pref: 7,      // DH group14-sha1
            host_key_type: 2, // RSA
            kex_algo_count: 8.0,
            host_key_algo_count: 3.0,
            cipher_count: 4.0,
            supports_ed25519: 0.0,
            offers_dh_group14_sha1: 1.0,
            unimpl_total_length: 0.0,
        }
    }

    fn mixed_fp() -> SshFeatureVector {
        SshFeatureVector {
            kex_pref: 1,      // Curve25519
            host_key_type: 2, // RSA
            kex_algo_count: 15.0,
            host_key_algo_count: 8.0,
            cipher_count: 10.0,
            supports_ed25519: 1.0,
            offers_dh_group14_sha1: 1.0,
            unimpl_total_length: 0.0,
        }
    }

    #[test]
    fn test_feature_vector_extraction() {
        let fp = SshBehavioralFingerprint {
            software_version: "OpenSSH_9.6".into(),
            kex_preference: SshKexAlgo::Curve25519Sha256,
            host_key_type: SshHostKeyType::Ed25519,
            kex_algo_count: 12,
            host_key_algo_count: 6,
            cipher_count: 8,
            supports_ed25519: true,
            offers_dh_group14_sha1: false,
            unimplemented_metrics: None,
            signature: "test".into(),
        };

        let fv = SshFeatureVector::from_fingerprint(&fp);
        assert_eq!(fv.kex_pref, 1);
        assert_eq!(fv.host_key_type, 1);
        assert_eq!(fv.kex_algo_count, 12.0);
    }

    #[test]
    fn test_distance_same() {
        let a = modern_fp();
        let b = modern_fp();
        assert!((a.distance(&b) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_distance_different() {
        let a = modern_fp();
        let b = legacy_fp();
        assert!(a.distance(&b) > 0.0);
    }

    #[test]
    fn test_cluster_basic() {
        let vectors = vec![modern_fp(), legacy_fp(), mixed_fp()];
        let clusterer = SshClusterer::new(2);
        let clusters = clusterer.cluster(&vectors);
        assert_eq!(clusters.len(), 2);
        // Each cluster should have at least one member
        assert!(clusters.iter().all(|c| !c.members.is_empty()));
    }

    #[test]
    fn test_cluster_single_group() {
        let vectors = vec![modern_fp(), modern_fp(), modern_fp()];
        let clusterer = SshClusterer::new(2);
        let clusters = clusterer.cluster(&vectors);
        assert_eq!(clusters.len(), 2);
        // One cluster should have all members, other should be empty or have 1
        let total_members: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total_members, 3);
    }

    #[test]
    fn test_empty_vectors() {
        let vectors = vec![];
        let clusterer = SshClusterer::new(3);
        let clusters = clusterer.cluster(&vectors);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_threat_actor_classify_modern() {
        let label = ThreatActorCluster::from_centroid(&modern_fp());
        assert_eq!(label, ThreatActorCluster::Modern);
    }

    #[test]
    fn test_threat_actor_classify_legacy() {
        let label = ThreatActorCluster::from_centroid(&legacy_fp());
        assert_eq!(label, ThreatActorCluster::Legacy);
    }

    #[test]
    fn test_threat_actor_classify_mixed() {
        let label = ThreatActorCluster::from_centroid(&mixed_fp());
        assert_eq!(label, ThreatActorCluster::Mixed);
    }

    #[test]
    fn test_kmeans_convergence() {
        // 4 clearly separable clusters
        let mut vectors = Vec::new();
        for _ in 0..5 {
            vectors.push(SshFeatureVector {
                kex_pref: 1,
                host_key_type: 1,
                kex_algo_count: 12.0,
                host_key_algo_count: 6.0,
                cipher_count: 8.0,
                supports_ed25519: 1.0,
                offers_dh_group14_sha1: 0.0,
                unimpl_total_length: 0.0,
            });
            vectors.push(SshFeatureVector {
                kex_pref: 7,
                host_key_type: 2,
                kex_algo_count: 8.0,
                host_key_algo_count: 3.0,
                cipher_count: 4.0,
                supports_ed25519: 0.0,
                offers_dh_group14_sha1: 1.0,
                unimpl_total_length: 0.0,
            });
            vectors.push(SshFeatureVector {
                kex_pref: 2,
                host_key_type: 3,
                kex_algo_count: 10.0,
                host_key_algo_count: 4.0,
                cipher_count: 6.0,
                supports_ed25519: 0.0,
                offers_dh_group14_sha1: 0.0,
                unimpl_total_length: 0.0,
            });
            vectors.push(SshFeatureVector {
                kex_pref: 5,
                host_key_type: 2,
                kex_algo_count: 15.0,
                host_key_algo_count: 8.0,
                cipher_count: 12.0,
                supports_ed25519: 0.0,
                offers_dh_group14_sha1: 1.0,
                unimpl_total_length: 0.0,
            });
        }

        let clusterer = SshClusterer::with_max_iter(4, 200);
        let clusters = clusterer.cluster(&vectors);

        // Should form 4 distinct clusters
        let total_members: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total_members, 20);
        assert_eq!(clusters.len(), 4);
    }
}
