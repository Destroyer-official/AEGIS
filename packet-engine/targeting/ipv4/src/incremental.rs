use std::collections::HashMap;
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::{
    Feature, HostObservation, MatrixError, OptimizedMatrix, RawCounts, compute_raw_counts,
    scale::{PrefixEntry, ScaleConfig, ScaleMatrixBuilder},
};

#[derive(Debug, Error)]
pub enum IncrementalError {
    #[error("matrix error: {0}")]
    Matrix(#[from] MatrixError),
    #[error("scale error: {0}")]
    Scale(#[from] crate::scale::ScaleError),
    #[error("delta processing failed: {0}")]
    DeltaFailed(String),
}

/// Snapshot of a prefix's contribution to the matrix.
#[derive(Debug, Clone)]
pub struct PrefixSnapshot {
    pub prefix: String,
    pub asn: u32,
    pub features: Vec<Feature>,
    pub observation_count: u32,
}

/// A set of changes to apply to the prefix table.
#[derive(Debug, Clone)]
pub struct DeltaSet {
    pub added: Vec<PrefixEntry>,
    pub removed: Vec<String>,
    pub modified: Vec<PrefixEntry>,
}

/// Report from applying a delta.
#[derive(Debug, Clone)]
pub struct DeltaReport {
    pub prefixes_added: usize,
    pub prefixes_removed: usize,
    pub prefixes_modified: usize,
    pub observations_added: usize,
    pub observations_subtracted: usize,
    pub elapsed: Duration,
    pub matrix_total_hosts: u64,
}

/// Matrix wrapper that supports incremental updates from BGP/ROA deltas.
pub struct IncrementalMatrix {
    matrix: OptimizedMatrix,
    prefix_index: HashMap<String, PrefixSnapshot>,
    raw_cond: ndarray::Array2<f64>,
    raw_marginal: ndarray::Array1<u64>,
    raw_port_counts: HashMap<u16, u64>,
    raw_total: u64,
    config: ScaleConfig,
}

impl IncrementalMatrix {
    /// Build the initial matrix from prefixes using a full build.
    pub fn from_prefixes(
        prefixes: &[PrefixEntry],
        config: ScaleConfig,
    ) -> Result<Self, IncrementalError> {
        let mut builder = ScaleMatrixBuilder::new(config.clone());
        let matrix = builder.build_from_prefixes(prefixes)?;

        // Compute raw counts by re-processing observations.
        let mut prefix_index = HashMap::new();
        let mut all_raw: Option<RawCounts> = None;

        for prefix in prefixes {
            let features = builder.features_for_prefix(prefix);
            let observations = builder.generate_observations_for_prefix(prefix, &features);
            let obs_count = observations.len() as u32;

            if let Some(raw) = compute_raw_counts(&observations) {
                if let Some(ref mut existing) = all_raw {
                    existing.cooccurrence += &raw.cooccurrence;
                    existing.marginal += &raw.marginal;
                    for (port, count) in raw.port_counts {
                        *existing.port_counts.entry(port).or_insert(0) += count;
                    }
                    existing.total += raw.total;
                } else {
                    all_raw = Some(raw);
                }
            }

            prefix_index.insert(
                prefix.prefix.clone(),
                PrefixSnapshot {
                    prefix: prefix.prefix.clone(),
                    asn: prefix.asn,
                    features,
                    observation_count: obs_count,
                },
            );
        }

        let raw = all_raw.unwrap_or_else(|| RawCounts {
            cooccurrence: ndarray::Array2::zeros((Feature::COUNT, Feature::COUNT)),
            marginal: ndarray::Array1::zeros(Feature::COUNT),
            port_counts: HashMap::new(),
            total: 0,
        });

        Ok(Self {
            matrix,
            prefix_index,
            raw_cond: raw.cooccurrence,
            raw_marginal: raw.marginal,
            raw_port_counts: raw.port_counts,
            raw_total: raw.total,
            config,
        })
    }

    /// Apply a delta set (added/removed/modified prefixes) incrementally.
    pub fn apply_delta(&mut self, delta: &DeltaSet) -> Result<DeltaReport, IncrementalError> {
        let start = Instant::now();
        let mut obs_added = 0usize;
        let mut obs_subtracted = 0usize;

        let builder = ScaleMatrixBuilder::new(self.config.clone());

        // 1. Remove: subtract observations for removed prefixes.
        for prefix_str in &delta.removed {
            if let Some(snapshot) = self.prefix_index.remove(prefix_str) {
                let observations = self.generate_observation_list(&snapshot);
                obs_subtracted += observations.len();
                self.subtract_observations_from_raw(&observations);
            }
        }

        // 2. Modified: subtract old, add new.
        for prefix in &delta.modified {
            // Remove old snapshot.
            if let Some(old_snapshot) = self.prefix_index.remove(&prefix.prefix) {
                let old_obs = self.generate_observation_list(&old_snapshot);
                obs_subtracted += old_obs.len();
                self.subtract_observations_from_raw(&old_obs);
            }
            // Add new.
            let features = builder.features_for_prefix(prefix);
            let observations = builder.generate_observations_for_prefix(prefix, &features);
            obs_added += observations.len();
            self.add_observations_to_raw(&observations);
            self.prefix_index.insert(
                prefix.prefix.clone(),
                PrefixSnapshot {
                    prefix: prefix.prefix.clone(),
                    asn: prefix.asn,
                    features,
                    observation_count: observations.len() as u32,
                },
            );
        }

        // 3. Added: generate observations and add.
        for prefix in &delta.added {
            let features = builder.features_for_prefix(prefix);
            let observations = builder.generate_observations_for_prefix(prefix, &features);
            obs_added += observations.len();
            self.add_observations_to_raw(&observations);
            self.prefix_index.insert(
                prefix.prefix.clone(),
                PrefixSnapshot {
                    prefix: prefix.prefix.clone(),
                    asn: prefix.asn,
                    features,
                    observation_count: observations.len() as u32,
                },
            );
        }

        // 4. Rebuild matrix from updated raw counts.
        self.matrix.rebuild_from_raw(
            &self.raw_cond,
            &self.raw_marginal,
            &self.raw_port_counts,
            self.raw_total,
        );

        Ok(DeltaReport {
            prefixes_added: delta.added.len(),
            prefixes_removed: delta.removed.len(),
            prefixes_modified: delta.modified.len(),
            observations_added: obs_added,
            observations_subtracted: obs_subtracted,
            elapsed: start.elapsed(),
            matrix_total_hosts: self.raw_total,
        })
    }

    /// Reference to the underlying matrix (for predictions).
    pub fn matrix(&self) -> &OptimizedMatrix {
        &self.matrix
    }

    /// Number of tracked prefixes.
    pub fn prefix_count(&self) -> usize {
        self.prefix_index.len()
    }

    /// Get a snapshot by prefix string.
    pub fn get_snapshot(&self, prefix: &str) -> Option<&PrefixSnapshot> {
        self.prefix_index.get(prefix)
    }

    // ─── Private helpers ──────────────────────────────────────────────

    fn add_observations_to_raw(&mut self, observations: &[HostObservation]) {
        if observations.is_empty() {
            return;
        }
        if let Some(raw) = compute_raw_counts(observations) {
            self.raw_cond += &raw.cooccurrence;
            self.raw_marginal += &raw.marginal;
            for (port, count) in raw.port_counts {
                *self.raw_port_counts.entry(port).or_insert(0) += count;
            }
            self.raw_total += raw.total;
        }
    }

    fn subtract_observations_from_raw(&mut self, observations: &[HostObservation]) {
        if observations.is_empty() {
            return;
        }
        if let Some(raw) = compute_raw_counts(observations) {
            self.raw_cond.zip_mut_with(&raw.cooccurrence, |a, b| {
                *a = (*a - b).max(0.0);
            });
            self.raw_marginal.zip_mut_with(&raw.marginal, |a, b| {
                *a = a.saturating_sub(*b);
            });
            for (port, count) in raw.port_counts {
                if let Some(entry) = self.raw_port_counts.get_mut(&port) {
                    *entry = entry.saturating_sub(count);
                }
            }
            self.raw_total = self.raw_total.saturating_sub(raw.total);
        }
    }

    fn generate_observation_list(&self, snapshot: &PrefixSnapshot) -> Vec<HostObservation> {
        // Regenerate observations from the snapshot's prefix and features.
        let prefix = PrefixEntry {
            prefix: snapshot.prefix.clone(),
            asn: snapshot.asn,
            as_path: vec![snapshot.asn],
            origin_asn: snapshot.asn,
            roa_valid: None,
        };
        let builder = ScaleMatrixBuilder::new(self.config.clone());
        builder.generate_observations_for_prefix(&prefix, &snapshot.features)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prefix(i: usize) -> PrefixEntry {
        PrefixEntry {
            prefix: format!("10.{}.0.0/16", i % 256),
            asn: 16509 + (i % 100) as u32,
            as_path: vec![16509 + (i % 100) as u32, 1299],
            origin_asn: 16509 + (i % 100) as u32,
            roa_valid: Some(true),
        }
    }

    #[test]
    fn test_initial_build() {
        let prefixes: Vec<PrefixEntry> = (0..100).map(make_prefix).collect();
        let matrix = IncrementalMatrix::from_prefixes(&prefixes, ScaleConfig::default()).unwrap();
        assert!(matrix.matrix().is_built());
        assert!(matrix.matrix().total_hosts() > 0);
        assert_eq!(matrix.prefix_count(), 100);
    }

    #[test]
    fn test_apply_delta_add() {
        let prefixes: Vec<PrefixEntry> = (0..50).map(make_prefix).collect();
        let mut matrix =
            IncrementalMatrix::from_prefixes(&prefixes, ScaleConfig::default()).unwrap();
        let hosts_before = matrix.matrix().total_hosts();

        let new_prefixes: Vec<PrefixEntry> = (50..60).map(make_prefix).collect();
        let delta = DeltaSet {
            added: new_prefixes,
            removed: vec![],
            modified: vec![],
        };

        let report = matrix.apply_delta(&delta).unwrap();
        assert_eq!(report.prefixes_added, 10);
        assert!(report.observations_added > 0);
        assert!(matrix.matrix().total_hosts() > hosts_before);
        assert_eq!(matrix.prefix_count(), 60);
    }

    #[test]
    fn test_apply_delta_remove() {
        let prefixes: Vec<PrefixEntry> = (0..50).map(make_prefix).collect();
        let mut matrix =
            IncrementalMatrix::from_prefixes(&prefixes, ScaleConfig::default()).unwrap();
        let hosts_before = matrix.matrix().total_hosts();

        let delta = DeltaSet {
            added: vec![],
            removed: vec!["10.0.0.0/16".to_string()],
            modified: vec![],
        };

        let report = matrix.apply_delta(&delta).unwrap();
        assert_eq!(report.prefixes_removed, 1);
        assert!(report.observations_subtracted > 0);
        assert!(matrix.matrix().total_hosts() < hosts_before);
        assert_eq!(matrix.prefix_count(), 49);
    }

    #[test]
    fn test_apply_delta_modify() {
        let prefixes: Vec<PrefixEntry> = (0..50).map(make_prefix).collect();
        let mut matrix =
            IncrementalMatrix::from_prefixes(&prefixes, ScaleConfig::default()).unwrap();

        let modified = PrefixEntry {
            prefix: "10.0.0.0/16".to_string(),
            asn: 99999,
            as_path: vec![99999],
            origin_asn: 99999,
            roa_valid: Some(false),
        };

        let delta = DeltaSet {
            added: vec![],
            removed: vec![],
            modified: vec![modified],
        };

        let report = matrix.apply_delta(&delta).unwrap();
        assert_eq!(report.prefixes_modified, 1);
        let snapshot = matrix.get_snapshot("10.0.0.0/16").unwrap();
        assert_eq!(snapshot.asn, 99999);
    }

    #[test]
    fn test_incremental_matches_full() {
        // Build full matrix with 100 prefixes.
        let all_prefixes: Vec<PrefixEntry> = (0..100).map(make_prefix).collect();
        let mut full_builder = ScaleMatrixBuilder::new(ScaleConfig::default());
        let full_matrix = full_builder.build_from_prefixes(&all_prefixes).unwrap();

        // Build incremental with 80, then add 20.
        let initial: Vec<PrefixEntry> = (0..80).map(make_prefix).collect();
        let mut incr = IncrementalMatrix::from_prefixes(&initial, ScaleConfig::default()).unwrap();

        let added: Vec<PrefixEntry> = (80..100).map(make_prefix).collect();
        let delta = DeltaSet {
            added,
            removed: vec![],
            modified: vec![],
        };
        incr.apply_delta(&delta).unwrap();

        // Both should have the same total host count.
        assert_eq!(full_matrix.total_hosts(), incr.matrix().total_hosts());
    }

    #[test]
    fn test_delta_report_timing() {
        let prefixes: Vec<PrefixEntry> = (0..1000).map(make_prefix).collect();
        let mut matrix =
            IncrementalMatrix::from_prefixes(&prefixes, ScaleConfig::default()).unwrap();

        let delta = DeltaSet {
            added: (1000..1100).map(make_prefix).collect(),
            removed: (0..50).map(|i| format!("10.{}.0.0/16", i)).collect(),
            modified: vec![],
        };

        let report = matrix.apply_delta(&delta).unwrap();
        assert!(
            report.elapsed.as_secs() < 5,
            "Delta took {:?}",
            report.elapsed
        );
    }
}
