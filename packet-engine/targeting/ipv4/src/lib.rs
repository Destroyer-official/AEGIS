use std::collections::HashMap;
use std::fmt;

use ndarray::{Array1, Array2, ArrayView2};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "kafka")]
pub mod kafka_integration;

#[cfg(all(test, feature = "kafka"))]
mod integration_test;

pub mod seed_collector;

pub mod scale;

pub mod incremental;

pub mod backfill;

#[derive(Debug, Error)]
pub enum MatrixError {
    #[error("seed data is empty")]
    EmptySeedData,
    #[error("feature index out of bounds: {0}")]
    FeatureIndexOutOfBounds(usize),
    #[error("port out of valid range: {0}")]
    InvalidPort(u16),
    #[error("ndarray error: {0}")]
    NdArray(String),
    #[error("linear algebra error: {0}")]
    Linalg(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    PortSsh,
    PortHttp,
    PortHttps,
    PortSmtp,
    PortDns,
    PortFtp,
    PortMysql,
    PortRdp,
    AppHttpServer,
    AppTls,
    AppSshBanner,
    AppSmtpBanner,
    AppMysqlProtocol,
    AppRedisProtocol,
    AppPostgresProtocol,
    AppMongoProtocol,
    AppGrpcAlpn,
    AppQuic,
    NetAsn,
    NetCloudProvider,
    NetCdnProvider,
    NetDynIp,
    NetResidential,
    NetEducational,
    NetGovMilitary,
}

impl Feature {
    pub const COUNT: usize = 25;

    pub fn index(self) -> usize {
        match self {
            Feature::PortSsh => 0,
            Feature::PortHttp => 1,
            Feature::PortHttps => 2,
            Feature::PortSmtp => 3,
            Feature::PortDns => 4,
            Feature::PortFtp => 5,
            Feature::PortMysql => 6,
            Feature::PortRdp => 7,
            Feature::AppHttpServer => 8,
            Feature::AppTls => 9,
            Feature::AppSshBanner => 10,
            Feature::AppSmtpBanner => 11,
            Feature::AppMysqlProtocol => 12,
            Feature::AppRedisProtocol => 13,
            Feature::AppPostgresProtocol => 14,
            Feature::AppMongoProtocol => 15,
            Feature::AppGrpcAlpn => 16,
            Feature::AppQuic => 17,
            Feature::NetAsn => 18,
            Feature::NetCloudProvider => 19,
            Feature::NetCdnProvider => 20,
            Feature::NetDynIp => 21,
            Feature::NetResidential => 22,
            Feature::NetEducational => 23,
            Feature::NetGovMilitary => 24,
        }
    }

    pub fn all() -> impl Iterator<Item = Feature> {
        (0..Feature::COUNT).map(|i| Feature::from_index(i).unwrap())
    }

    fn from_index(i: usize) -> Option<Feature> {
        match i {
            0 => Some(Feature::PortSsh),
            1 => Some(Feature::PortHttp),
            2 => Some(Feature::PortHttps),
            3 => Some(Feature::PortSmtp),
            4 => Some(Feature::PortDns),
            5 => Some(Feature::PortFtp),
            6 => Some(Feature::PortMysql),
            7 => Some(Feature::PortRdp),
            8 => Some(Feature::AppHttpServer),
            9 => Some(Feature::AppTls),
            10 => Some(Feature::AppSshBanner),
            11 => Some(Feature::AppSmtpBanner),
            12 => Some(Feature::AppMysqlProtocol),
            13 => Some(Feature::AppRedisProtocol),
            14 => Some(Feature::AppPostgresProtocol),
            15 => Some(Feature::AppMongoProtocol),
            16 => Some(Feature::AppGrpcAlpn),
            17 => Some(Feature::AppQuic),
            18 => Some(Feature::NetAsn),
            19 => Some(Feature::NetCloudProvider),
            20 => Some(Feature::NetCdnProvider),
            21 => Some(Feature::NetDynIp),
            22 => Some(Feature::NetResidential),
            23 => Some(Feature::NetEducational),
            24 => Some(Feature::NetGovMilitary),
            _ => None,
        }
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Feature::PortSsh => write!(f, "port_ssh"),
            Feature::PortHttp => write!(f, "port_http"),
            Feature::PortHttps => write!(f, "port_https"),
            Feature::PortSmtp => write!(f, "port_smtp"),
            Feature::PortDns => write!(f, "port_dns"),
            Feature::PortFtp => write!(f, "port_ftp"),
            Feature::PortMysql => write!(f, "port_mysql"),
            Feature::PortRdp => write!(f, "port_rdp"),
            Feature::AppHttpServer => write!(f, "app_http_server"),
            Feature::AppTls => write!(f, "app_tls"),
            Feature::AppSshBanner => write!(f, "app_ssh_banner"),
            Feature::AppSmtpBanner => write!(f, "app_smtp_banner"),
            Feature::AppMysqlProtocol => write!(f, "app_mysql_proto"),
            Feature::AppRedisProtocol => write!(f, "app_redis_proto"),
            Feature::AppPostgresProtocol => write!(f, "app_postgres_proto"),
            Feature::AppMongoProtocol => write!(f, "app_mongo_proto"),
            Feature::AppGrpcAlpn => write!(f, "app_grpc_alpn"),
            Feature::AppQuic => write!(f, "app_quic"),
            Feature::NetAsn => write!(f, "net_asn"),
            Feature::NetCloudProvider => write!(f, "net_cloud"),
            Feature::NetCdnProvider => write!(f, "net_cdn"),
            Feature::NetDynIp => write!(f, "net_dyn_ip"),
            Feature::NetResidential => write!(f, "net_residential"),
            Feature::NetEducational => write!(f, "net_educational"),
            Feature::NetGovMilitary => write!(f, "net_gov_military"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostObservation {
    pub ip: u32,
    pub open_ports: Vec<u16>,
    pub features: Vec<Feature>,
    pub asn: u32,
}

#[derive(Debug, Clone)]
pub struct OptimizedMatrix {
    cond: Array2<f64>,
    marginal: Array1<u64>,
    port_probability: HashMap<u16, f64>,
    total_hosts: u64,
    built: bool,
}

impl OptimizedMatrix {
    pub fn new() -> Self {
        Self {
            cond: Array2::zeros((Feature::COUNT, Feature::COUNT)),
            marginal: Array1::zeros(Feature::COUNT),
            port_probability: HashMap::new(),
            total_hosts: 0,
            built: false,
        }
    }

    pub fn observe_batch_parallel(
        &mut self,
        observations: &[HostObservation],
    ) -> Result<(), MatrixError> {
        if observations.is_empty() {
            return Err(MatrixError::EmptySeedData);
        }

        let chunk_size = (observations.len() / rayon::current_num_threads()).max(1000);

        let partials: Vec<_> = observations
            .par_chunks(chunk_size)
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

    pub fn build(&mut self) -> Result<(), MatrixError> {
        if self.total_hosts == 0 {
            return Err(MatrixError::EmptySeedData);
        }

        for i in 0..Feature::COUNT {
            if self.marginal[i] == 0 {
                continue;
            }
            for j in 0..Feature::COUNT {
                if i == j {
                    self.cond[[i, j]] = 1.0;
                } else {
                    self.cond[[i, j]] /= self.marginal[i] as f64;
                }
            }
        }

        let total = self.total_hosts as f64;
        for prob in self.port_probability.values_mut() {
            *prob /= total;
        }

        self.built = true;
        Ok(())
    }

    pub fn predict_port(
        &self,
        target_port: u16,
        observed_features: &[Feature],
    ) -> Result<f64, MatrixError> {
        if !self.built {
            return Err(MatrixError::EmptySeedData);
        }

        let port_feat = port_to_feature(target_port)?;
        let base = self
            .port_probability
            .get(&target_port)
            .copied()
            .unwrap_or(0.0);

        if observed_features.is_empty() {
            return Ok(base);
        }

        let mut complement_product = 1.0 - base;
        for feat in observed_features {
            let fi = feat.index();
            let p_feat = self.cond[[fi, port_feat.index()]];
            complement_product *= 1.0 - p_feat;
        }

        Ok(1.0 - complement_product)
    }

    pub fn conditional_probability(
        &self,
        source: Feature,
        target: Feature,
    ) -> Result<f64, MatrixError> {
        if !self.built {
            return Err(MatrixError::EmptySeedData);
        }
        Ok(self.cond[[source.index(), target.index()]])
    }

    pub fn predict_first_service(
        &self,
        network_features: &[Feature],
    ) -> Result<Option<(u16, f64)>, MatrixError> {
        if !self.built {
            return Err(MatrixError::EmptySeedData);
        }

        if network_features.is_empty() {
            return Ok(self
                .port_probability
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&port, &prob)| (port, prob)));
        }

        let mut best_port = None;
        let mut best_prob = 0.0f64;

        for &port in self.port_probability.keys() {
            let prob = self.predict_port(port, network_features)?;
            if prob > best_prob {
                best_prob = prob;
                best_port = Some(port);
            }
        }

        Ok(best_port.map(|p| (p, best_prob)))
    }

    pub fn predict_remaining_services(
        &self,
        observed_features: &[Feature],
        threshold: f64,
    ) -> Result<Vec<(u16, f64)>, MatrixError> {
        if !self.built {
            return Err(MatrixError::EmptySeedData);
        }

        let mut predictions: Vec<(u16, f64)> = self
            .port_probability
            .keys()
            .filter_map(|&port| {
                self.predict_port(port, observed_features)
                    .ok()
                    .map(|prob| (port, prob))
            })
            .filter(|(_, prob)| *prob >= threshold)
            .collect();

        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(predictions)
    }

    pub fn total_hosts(&self) -> u64 {
        self.total_hosts
    }
    pub fn is_built(&self) -> bool {
        self.built
    }
    pub fn cond_view(&self) -> ArrayView2<'_, f64> {
        self.cond.view()
    }

    // ─── Incremental update support (pub(crate)) ────────────────────────

    /// Rebuild probability tables from raw co-occurrence / marginal counts.
    /// `raw_cond` must be the unnormalized co-occurrence matrix (Feature×Feature).
    /// `raw_marginal` must be the unnormalized per-feature counts.
    /// `raw_port_counts` must be unnormalized per-port counts.
    /// This overwrites `self.cond`, `self.marginal`, `self.port_probability`
    /// and marks the matrix as built.
    pub fn rebuild_from_raw(
        &mut self,
        raw_cond: &Array2<f64>,
        raw_marginal: &Array1<u64>,
        raw_port_counts: &HashMap<u16, u64>,
        total_hosts: u64,
    ) {
        // Copy raw counts into the matrix fields.
        self.cond.assign(raw_cond);
        self.marginal.assign(raw_marginal);
        self.port_probability.clear();
        for (&port, &count) in raw_port_counts {
            self.port_probability.insert(port, count as f64);
        }
        self.total_hosts = total_hosts;

        // Normalize.
        if self.total_hosts > 0 {
            for i in 0..Feature::COUNT {
                if self.marginal[i] == 0 {
                    continue;
                }
                for j in 0..Feature::COUNT {
                    if i == j {
                        self.cond[[i, j]] = 1.0;
                    } else {
                        self.cond[[i, j]] /= self.marginal[i] as f64;
                    }
                }
            }
            let total = self.total_hosts as f64;
            for prob in self.port_probability.values_mut() {
                *prob /= total;
            }
        }
        self.built = true;
    }
}

impl Default for OptimizedMatrix {
    fn default() -> Self {
        Self::new()
    }
}

fn port_to_feature(port: u16) -> Result<Feature, MatrixError> {
    match port {
        22 => Ok(Feature::PortSsh),
        80 => Ok(Feature::PortHttp),
        443 => Ok(Feature::PortHttps),
        25 => Ok(Feature::PortSmtp),
        53 => Ok(Feature::PortDns),
        21 => Ok(Feature::PortFtp),
        3306 => Ok(Feature::PortMysql),
        3389 => Ok(Feature::PortRdp),
        _ => Err(MatrixError::InvalidPort(port)),
    }
}

/// Raw co-occurrence counts computed from a batch of observations.
/// Used by both the full build and incremental updates.
#[derive(Debug, Clone)]
pub struct RawCounts {
    pub cooccurrence: Array2<f64>,
    pub marginal: Array1<u64>,
    pub port_counts: HashMap<u16, u64>,
    pub total: u64,
}

/// Compute raw co-occurrence counts from observations in parallel.
pub fn compute_raw_counts(observations: &[HostObservation]) -> Option<RawCounts> {
    if observations.is_empty() {
        return None;
    }

    let chunk_size = (observations.len() / rayon::current_num_threads()).max(1000);

    let partials: Vec<_> = observations
        .par_chunks(chunk_size)
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

    let mut cooccurrence = Array2::<f64>::zeros((Feature::COUNT, Feature::COUNT));
    let mut marginal = Array1::<u64>::zeros(Feature::COUNT);
    let mut port_counts = HashMap::<u16, u64>::new();
    let mut total = 0u64;

    for (cooc, marg, ports, t) in partials {
        cooccurrence += &cooc.mapv(|x| x as f64);
        marginal += &marg;
        for (port, count) in ports {
            *port_counts.entry(port).or_insert(0) += count;
        }
        total += t;
    }

    Some(RawCounts {
        cooccurrence,
        marginal,
        port_counts,
        total,
    })
}

#[derive(Debug, Clone)]
pub struct TargetPrediction {
    pub ip: u32,
    pub asn: u32,
    pub predicted_ports: Vec<(u16, f64)>,
    pub source_features: Vec<Feature>,
    pub prediction_method: PredictionMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionMethod {
    FirstService,
    RemainingService,
}

impl fmt::Display for PredictionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictionMethod::FirstService => write!(f, "first_service"),
            PredictionMethod::RemainingService => write!(f, "remaining_service"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_observation(
        ip: u32,
        ports: Vec<u16>,
        features: Vec<Feature>,
        asn: u32,
    ) -> HostObservation {
        HostObservation {
            ip,
            open_ports: ports,
            features,
            asn,
        }
    }

    #[test]
    fn test_optimized_matrix() {
        let mut matrix = OptimizedMatrix::new();

        for i in 0..3 {
            matrix
                .observe_batch_parallel(&[make_observation(
                    i,
                    vec![22, 80, 443],
                    vec![
                        Feature::PortSsh,
                        Feature::PortHttp,
                        Feature::PortHttps,
                        Feature::AppTls,
                    ],
                    15169,
                )])
                .unwrap();
        }

        matrix.build().unwrap();
        assert_eq!(matrix.total_hosts(), 3);

        let p = matrix
            .conditional_probability(Feature::PortHttp, Feature::PortHttps)
            .unwrap();
        assert!((p - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parallel_performance() {
        let mut matrix = OptimizedMatrix::new();
        let obs: Vec<_> = (0..10000)
            .map(|i| {
                make_observation(
                    i,
                    vec![80, 443],
                    vec![Feature::PortHttp, Feature::PortHttps, Feature::AppTls],
                    16509,
                )
            })
            .collect();

        let start = std::time::Instant::now();
        matrix.observe_batch_parallel(&obs).unwrap();
        matrix.build().unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs_f64() < 2.0,
            "10K hosts should build in <2s, took {:.3}s",
            elapsed.as_secs_f64()
        );
        assert_eq!(matrix.total_hosts(), 10000);
    }
}
