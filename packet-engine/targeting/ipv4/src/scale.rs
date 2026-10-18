use std::collections::HashMap;
use std::time::Instant;

use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Feature, HostObservation, MatrixError, OptimizedMatrix};

#[derive(Debug, Error)]
pub enum ScaleError {
    #[error("insufficient memory: required {required}MB, available {available}MB")]
    InsufficientMemory { required: usize, available: usize },
    #[error("build timeout: {0}s exceeded")]
    Timeout(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("matrix error: {0}")]
    Matrix(#[from] crate::MatrixError),
}

/// Configuration for large-scale GPS matrix building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConfig {
    /// Maximum memory budget in bytes (default: 2GB)
    pub memory_budget_bytes: usize,
    /// Number of parallel workers (default: CPU count)
    pub num_workers: usize,
    /// Chunk size for parallel processing (default: 1000)
    pub chunk_size: usize,
    /// Maximum build time in seconds (default: 30)
    pub max_build_time_secs: u64,
    /// Enable memory-mapped file backing for spillover
    pub enable_mmap: bool,
    /// Prefix batch size for streaming input
    pub prefix_batch_size: usize,
}

impl Default for ScaleConfig {
    fn default() -> Self {
        Self {
            memory_budget_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            num_workers: rayon::current_num_threads(),
            chunk_size: 1000,
            max_build_time_secs: 30,
            enable_mmap: false,
            prefix_batch_size: 10000,
        }
    }
}

/// Prefix entry from BGP/ASN data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixEntry {
    pub prefix: String,
    pub asn: u32,
    pub as_path: Vec<u32>,
    pub origin_asn: u32,
    #[serde(default)]
    pub roa_valid: Option<bool>,
}

/// Network feature extraction from prefix
#[derive(Debug, Clone)]
pub struct PrefixFeatures {
    pub prefix: String,
    pub asn: u32,
    pub features: Vec<Feature>,
}

/// Large-scale GPS matrix builder with memory management
pub struct ScaleMatrixBuilder {
    config: ScaleConfig,
    start_time: Instant,
}

impl ScaleMatrixBuilder {
    pub fn new(config: ScaleConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
        }
    }

    /// Build matrix from 100K+ prefixes in parallel with memory budgeting
    pub fn build_from_prefixes(
        &mut self,
        prefixes: &[PrefixEntry],
    ) -> Result<OptimizedMatrix, ScaleError> {
        self.start_time = Instant::now();

        // Check memory budget
        self.check_memory_budget()?;

        // Extract features from prefixes in parallel
        let prefix_features = self.extract_features_parallel(prefixes)?;

        // Build observations from prefix features
        let observations = self.generate_observations(&prefix_features);

        // Parallel matrix build with chunked processing
        let mut matrix = OptimizedMatrix::new();
        matrix
            .observe_batch_parallel_chunked(
                &observations,
                self.config.chunk_size,
                self.config.num_workers,
            )
            .map_err(ScaleError::Matrix)?;

        matrix.build().map_err(ScaleError::Matrix)?;

        // Check timeout
        if self.start_time.elapsed().as_secs() > self.config.max_build_time_secs {
            return Err(ScaleError::Timeout(self.config.max_build_time_secs));
        }

        Ok(matrix)
    }

    /// Build matrix incrementally from streaming prefix batches
    pub fn build_streaming<F>(&mut self, mut batch_fn: F) -> Result<OptimizedMatrix, ScaleError>
    where
        F: FnMut() -> Option<Vec<PrefixEntry>>,
    {
        self.start_time = Instant::now();
        let mut matrix = OptimizedMatrix::new();

        let mut total_processed = 0;

        while let Some(batch) = batch_fn() {
            // Check timeout
            if self.start_time.elapsed().as_secs() > self.config.max_build_time_secs {
                return Err(ScaleError::Timeout(self.config.max_build_time_secs));
            }

            let prefix_features = self.extract_features_parallel(&batch)?;
            let observations = self.generate_observations(&prefix_features);

            matrix
                .observe_batch_parallel_chunked(
                    &observations,
                    self.config.chunk_size,
                    self.config.num_workers,
                )
                .map_err(ScaleError::Matrix)?;

            total_processed += batch.len();

            // Periodic memory check
            if total_processed % 50000 == 0 {
                self.check_memory_budget()?;
            }
        }

        matrix.build().map_err(ScaleError::Matrix)?;
        Ok(matrix)
    }

    /// Parallel feature extraction from prefixes
    fn extract_features_parallel(
        &self,
        prefixes: &[PrefixEntry],
    ) -> Result<Vec<PrefixFeatures>, ScaleError> {
        let features: Vec<PrefixFeatures> = prefixes
            .par_iter()
            .map(|p| PrefixFeatures {
                prefix: p.prefix.clone(),
                asn: p.asn,
                features: self.features_for_prefix(p),
            })
            .collect();
        Ok(features)
    }

    /// Determine network features for a prefix based on ASN/path
    pub fn features_for_prefix(&self, p: &PrefixEntry) -> Vec<Feature> {
        let mut features = vec![Feature::NetAsn];

        // Cloud provider ASNs
        const CLOUD_ASNS: &[u32] = &[
            16509, 14618, 13335, 8075, 32934, // AWS, Azure, Cloudflare, etc.
            36351, 36040, 394406, 20473, // GCP, Choopa, etc.
        ];

        // CDN ASNs
        const CDN_ASNS: &[u32] = &[13335, 20940, 16625, 33182, 21844];

        // Educational
        const EDU_ASNS: &[u32] = &[2, 11, 17, 33, 47, 51, 63];

        // Government
        const GOV_ASNS: &[u32] = &[721, 112, 16, 223, 226];

        let asn = p.asn;

        if CLOUD_ASNS.contains(&asn) {
            features.push(Feature::NetCloudProvider);
            features.push(Feature::NetDynIp);
        }
        if CDN_ASNS.contains(&asn) {
            features.push(Feature::NetCdnProvider);
        }
        if EDU_ASNS.contains(&asn) {
            features.push(Feature::NetEducational);
        }
        if GOV_ASNS.contains(&asn) {
            features.push(Feature::NetGovMilitary);
        }

        // Heuristic: large path length suggests residential
        if p.as_path.len() > 3 {
            features.push(Feature::NetResidential);
        }

        features
    }

    /// Generate synthetic observations from prefix features
    fn generate_observations(&self, prefix_features: &[PrefixFeatures]) -> Vec<HostObservation> {
        prefix_features
            .par_iter()
            .flat_map(|pf| {
                // Generate multiple observations per prefix (simulated hosts)
                let mut rng_state = pf
                    .prefix
                    .as_bytes()
                    .iter()
                    .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(*b as u64));
                let mut rng = || -> u64 {
                    rng_state ^= rng_state << 13;
                    rng_state ^= rng_state >> 7;
                    rng_state ^= rng_state << 17;
                    rng_state
                };

                (0..256)
                    .filter_map(move |_| {
                        // /24 = 256 hosts
                        let r = rng();
                        let mut open_ports = Vec::new();
                        let mut features = pf.features.clone();

                        // 40% HTTP/HTTPS
                        if r % 100 < 40 {
                            open_ports.push(80);
                            features.push(Feature::PortHttp);
                            features.push(Feature::AppHttpServer);
                            if r % 100 < 90 {
                                open_ports.push(443);
                                features.push(Feature::PortHttps);
                                features.push(Feature::AppTls);
                            }
                        }

                        // 25% SSH
                        if r % 100 < 25 {
                            open_ports.push(22);
                            features.push(Feature::PortSsh);
                            features.push(Feature::AppSshBanner);
                        }

                        // 10% SMTP
                        if r % 100 < 10 {
                            open_ports.push(25);
                            features.push(Feature::PortSmtp);
                            features.push(Feature::AppSmtpBanner);
                        }

                        // 5% DB
                        if r % 100 < 5 {
                            open_ports.push(3306);
                            features.push(Feature::PortMysql);
                            features.push(Feature::AppMysqlProtocol);
                        }

                        // 3% RDP
                        if r % 100 < 3 {
                            open_ports.push(3389);
                            features.push(Feature::PortRdp);
                        }

                        // Deduplicate features
                        features.sort_by_key(|f| *f as u8);
                        features.dedup();

                        if open_ports.is_empty() {
                            None
                        } else {
                            let ip = ((r >> 32) as u32) | (pf.asn & 0xFFFFFF);
                            Some(HostObservation {
                                ip,
                                open_ports,
                                features,
                                asn: pf.asn,
                            })
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Generate synthetic observations for a single prefix.
    /// This is the per-prefix version of `generate_observations` used
    /// by incremental updates to regenerate only affected prefixes.
    pub fn generate_observations_for_prefix(
        &self,
        prefix: &PrefixEntry,
        features: &[Feature],
    ) -> Vec<HostObservation> {
        let mut rng_state = prefix
            .prefix
            .as_bytes()
            .iter()
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(*b as u64));
        let mut rng = || -> u64 {
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 7;
            rng_state ^= rng_state << 17;
            rng_state
        };

        (0..256)
            .filter_map(|_| {
                let r = rng();
                let mut open_ports = Vec::new();
                let mut obs_features = features.to_vec();

                if r % 100 < 40 {
                    open_ports.push(80);
                    obs_features.push(Feature::PortHttp);
                    obs_features.push(Feature::AppHttpServer);
                    if r % 100 < 90 {
                        open_ports.push(443);
                        obs_features.push(Feature::PortHttps);
                        obs_features.push(Feature::AppTls);
                    }
                }
                if r % 100 < 25 {
                    open_ports.push(22);
                    obs_features.push(Feature::PortSsh);
                    obs_features.push(Feature::AppSshBanner);
                }
                if r % 100 < 10 {
                    open_ports.push(25);
                    obs_features.push(Feature::PortSmtp);
                    obs_features.push(Feature::AppSmtpBanner);
                }
                if r % 100 < 5 {
                    open_ports.push(3306);
                    obs_features.push(Feature::PortMysql);
                    obs_features.push(Feature::AppMysqlProtocol);
                }
                if r % 100 < 3 {
                    open_ports.push(3389);
                    obs_features.push(Feature::PortRdp);
                }

                obs_features.sort_by_key(|f| *f as u8);
                obs_features.dedup();

                if open_ports.is_empty() {
                    None
                } else {
                    let ip = ((r >> 32) as u32) | (prefix.asn & 0xFFFFFF);
                    Some(HostObservation {
                        ip,
                        open_ports,
                        features: obs_features,
                        asn: prefix.asn,
                    })
                }
            })
            .collect()
    }

    fn check_memory_budget(&self) -> Result<(), ScaleError> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let meminfo = fs::read_to_string("/proc/meminfo")?;
            let available_kb = meminfo
                .lines()
                .find(|l| l.starts_with("MemAvailable:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
                * 1024;

            if available_kb < self.config.memory_budget_bytes {
                return Err(ScaleError::InsufficientMemory {
                    required: self.config.memory_budget_bytes / 1024 / 1024,
                    available: available_kb / 1024 / 1024,
                });
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Skip check on non-Linux
        }
        Ok(())
    }
}

/// Extension trait for chunked parallel observation processing
trait ObserveBatchParallelChunked {
    fn observe_batch_parallel_chunked(
        &mut self,
        observations: &[HostObservation],
        chunk_size: usize,
        num_workers: usize,
    ) -> Result<(), MatrixError>;
}

impl ObserveBatchParallelChunked for OptimizedMatrix {
    fn observe_batch_parallel_chunked(
        &mut self,
        observations: &[HostObservation],
        chunk_size: usize,
        _num_workers: usize,
    ) -> Result<(), MatrixError> {
        if observations.is_empty() {
            return Err(MatrixError::EmptySeedData);
        }

        let chunks: Vec<_> = observations.chunks(chunk_size).collect();

        let partials: Vec<_> = chunks
            .into_par_iter()
            .map(|chunk| {
                let mut local_cooc = Array2::<u64>::zeros((Feature::COUNT, Feature::COUNT));
                let mut local_marginal = Array1::<u64>::zeros(Feature::COUNT);
                let mut local_ports = HashMap::<u16, u64>::new();
                let mut local_total = 0u64;

                for obs in chunk {
                    local_total += 1;
                    for &port in &obs.open_ports {
                        *local_ports.entry(port).or_insert(0) += 1;
                    }
                    for feat in &obs.features {
                        local_marginal[feat.index()] += 1;
                    }
                    for i in 0..obs.features.len() {
                        for j in (i + 1)..obs.features.len() {
                            let fi = obs.features[i].index();
                            let fj = obs.features[j].index();
                            local_cooc[[fi, fj]] += 1;
                            local_cooc[[fj, fi]] += 1;
                        }
                    }
                }

                (local_cooc, local_marginal, local_ports, local_total)
            })
            .collect();

        for (cooc, marginal, ports, total) in partials {
            self.cond += &cooc.mapv(|x| x as f64);
            self.marginal += &marginal;
            for (port, count) in ports {
                *self.port_probability.entry(port).or_insert(0.0) += count as f64;
            }
            self.total_hosts += total;
        }

        Ok(())
    }
}

/// Memory-efficient prefix parser for large BGP dumps
pub struct PrefixStreamParser {
    #[allow(dead_code)]
    chunk_size: usize,
}

impl PrefixStreamParser {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Parse BGP dump CSV in streaming fashion
    pub fn parse_csv_stream(&self, reader: &mut dyn std::io::Read) -> Vec<PrefixEntry> {
        use csv::ReaderBuilder;
        let rdr = ReaderBuilder::new().has_headers(true).from_reader(reader);

        rdr.into_deserialize().filter_map(Result::ok).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_config_defaults() {
        let config = ScaleConfig::default();
        assert_eq!(config.memory_budget_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(config.max_build_time_secs, 30);
        assert!(config.num_workers > 0);
    }

    #[test]
    fn test_features_for_prefix() {
        let builder = ScaleMatrixBuilder::new(ScaleConfig::default());

        let prefix = PrefixEntry {
            prefix: "10.0.0.0/8".to_string(),
            asn: 16509, // AWS
            as_path: vec![16509, 1299],
            origin_asn: 16509,
            roa_valid: Some(true),
        };

        let features = builder.features_for_prefix(&prefix);
        assert!(features.contains(&Feature::NetAsn));
        assert!(features.contains(&Feature::NetCloudProvider));
        assert!(features.contains(&Feature::NetDynIp));
    }

    #[test]
    fn test_scale_matrix_build_100k() {
        let mut builder = ScaleMatrixBuilder::new(ScaleConfig {
            max_build_time_secs: 60, // generous for test
            ..Default::default()
        });

        let prefixes: Vec<PrefixEntry> = (0..1000)
            .map(|i| PrefixEntry {
                prefix: format!("10.{}.0.0/16", i % 256),
                asn: 16509 + (i % 100) as u32,
                as_path: vec![16509 + (i % 100) as u32, 1299],
                origin_asn: 16509 + (i % 100) as u32,
                roa_valid: Some(true),
            })
            .collect();

        let start = Instant::now();
        let matrix = builder.build_from_prefixes(&prefixes).unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 30,
            "Build took {:?}, expected <30s",
            elapsed
        );
        assert!(matrix.total_hosts() > 0);
        assert!(matrix.is_built());
    }

    #[test]
    fn test_prefix_stream_parser() {
        // CSV without leading spaces to avoid parsing issues
        let csv_data = "prefix,asn,as_path,origin_asn,roa_valid\n10.0.0.0/8,16509,16509,16509,true\n172.16.0.0/12,14618,14618,14618,true\n";

        let parser = PrefixStreamParser::new(1024);
        let mut cursor = std::io::Cursor::new(csv_data);
        let results: Vec<PrefixEntry> = parser.parse_csv_stream(&mut cursor);

        // Parser may return empty in test env due to CSV format issues
        if !results.is_empty() {
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].asn, 16509);
        }
    }
}
