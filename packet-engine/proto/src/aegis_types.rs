//! Pre-generated prost message types for AEGIS protobuf schema.
//! Generated from aegis_types.proto — DO NOT EDIT MANUALLY.
//! Re-generate with: protoc --prost_out=. --proto_path=. aegis_types.proto

use prost::Message;

// ─── Enums ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum Transport {
    Unspecified = 0,
    Tcp = 1,
    Udp = 2,
}

impl Transport {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Transport::Unspecified => "TRANSPORT_UNSPECIFIED",
            Transport::Tcp => "TRANSPORT_TCP",
            Transport::Udp => "TRANSPORT_UDP",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum TargetSource {
    Unspecified = 0,
    GpsCoOccurrence = 1,
    GpsFirstService = 2,
    SixHitRl = 3,
    PassiveCt = 4,
    PassiveDns = 5,
    BgpSeed = 6,
    Random = 7,
}

impl TargetSource {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TargetSource::Unspecified => "TARGET_SOURCE_UNSPECIFIED",
            TargetSource::GpsCoOccurrence => "TARGET_SOURCE_GPS_CO_OCCURRENCE",
            TargetSource::GpsFirstService => "TARGET_SOURCE_GPS_FIRST_SERVICE",
            TargetSource::SixHitRl => "TARGET_SOURCE_6HIT_RL",
            TargetSource::PassiveCt => "TARGET_SOURCE_PASSIVE_CT",
            TargetSource::PassiveDns => "TARGET_SOURCE_PASSIVE_DNS",
            TargetSource::BgpSeed => "TARGET_SOURCE_BGP_SEED",
            TargetSource::Random => "TARGET_SOURCE_RANDOM",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum DetectionMethod {
    Unspecified = 0,
    PortNumber = 1,
    LzrHeuristic = 2,
    ActiveProbe = 3,
}

impl DetectionMethod {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DetectionMethod::Unspecified => "DETECTION_METHOD_UNSPECIFIED",
            DetectionMethod::PortNumber => "DETECTION_METHOD_PORT_NUMBER",
            DetectionMethod::LzrHeuristic => "DETECTION_METHOD_LZR_HEURISTIC",
            DetectionMethod::ActiveProbe => "DETECTION_METHOD_ACTIVE_PROBE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum HostStatus {
    Unspecified = 0,
    Active = 1,
    PendingEviction = 2,
    Evicted = 3,
}

impl HostStatus {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            HostStatus::Unspecified => "HOST_STATUS_UNSPECIFIED",
            HostStatus::Active => "HOST_STATUS_ACTIVE",
            HostStatus::PendingEviction => "HOST_STATUS_PENDING_EVICTION",
            HostStatus::Evicted => "HOST_STATUS_EVICTED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum GeoTier {
    Unspecified = 0,
    Cryptographic = 1,
    Rtt = 2,
    Passive = 3,
}

impl GeoTier {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GeoTier::Unspecified => "GEO_TIER_UNSPECIFIED",
            GeoTier::Cryptographic => "GEO_TIER_CRYPTOGRAPHIC",
            GeoTier::Rtt => "GEO_TIER_RTT",
            GeoTier::Passive => "GEO_TIER_PASSIVE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum L4Result {
    Unspecified = 0,
    Open = 1,
    Closed = 2,
    Filtered = 3,
    HostAlive = 4,
}

impl L4Result {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            L4Result::Unspecified => "L4_RESULT_UNSPECIFIED",
            L4Result::Open => "L4_RESULT_OPEN",
            L4Result::Closed => "L4_RESULT_CLOSED",
            L4Result::Filtered => "L4_RESULT_FILTERED",
            L4Result::HostAlive => "L4_RESULT_HOST_ALIVE",
        }
    }
}

// ─── Messages ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct ScanCommand {
    #[prost(uint64, tag = "1")]
    pub command_id: u64,
    #[prost(bytes, tag = "2")]
    pub target_ip: Vec<u8>,
    #[prost(uint32, tag = "3")]
    pub port: u32,
    #[prost(enumeration = "Transport", tag = "4")]
    pub transport: i32,
    #[prost(double, tag = "5")]
    pub priority: f64,
    #[prost(enumeration = "TargetSource", tag = "6")]
    pub source: i32,
    #[prost(double, tag = "7")]
    pub conditional_probability: f64,
    #[prost(uint32, tag = "8")]
    pub asn: u32,
    #[prost(uint32, repeated, tag = "9")]
    pub feature_tuple: Vec<u32>,
    #[prost(string, tag = "10")]
    pub generated_at: String,
    #[prost(string, tag = "11")]
    pub cidr_block: String,
    #[prost(string, tag = "12")]
    pub asn_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct L4Response {
    #[prost(string, tag = "1")]
    pub event_id: String,
    #[prost(bytes, tag = "2")]
    pub target_ip: Vec<u8>,
    #[prost(uint32, tag = "3")]
    pub target_port: u32,
    #[prost(enumeration = "Transport", tag = "4")]
    pub transport: i32,
    #[prost(enumeration = "L4Result", tag = "5")]
    pub result: i32,
    #[prost(uint32, tag = "6")]
    pub rtt_usec: u32,
    #[prost(uint32, tag = "7")]
    pub ttl: u32,
    #[prost(uint32, tag = "8")]
    pub tcp_window: u32,
    #[prost(string, tag = "9")]
    pub source_pop: String,
    #[prost(string, tag = "10")]
    pub sent_at: String,
    #[prost(string, tag = "11")]
    pub received_at: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct TLSFingerprint {
    #[prost(string, tag = "1")]
    pub event_id: String,
    #[prost(bytes, tag = "2")]
    pub target_ip: Vec<u8>,
    #[prost(uint32, tag = "3")]
    pub target_port: u32,
    #[prost(enumeration = "Transport", tag = "4")]
    pub transport: i32,
    #[prost(string, tag = "5")]
    pub ja4: String,
    #[prost(string, tag = "6")]
    pub ja4s: String,
    #[prost(string, tag = "7")]
    pub tls_version: String,
    #[prost(string, tag = "8")]
    pub cipher_suite: String,
    #[prost(string, tag = "9")]
    pub sni: String,
    #[prost(string, tag = "10")]
    pub alpn: String,
    #[prost(string, tag = "11")]
    pub certificate_chain: String,
    #[prost(string, tag = "12")]
    pub cert_fingerprint: String,
    #[prost(string, tag = "13")]
    pub cert_subject_cn: String,
    #[prost(string, tag = "14")]
    pub cert_issuer_cn: String,
    #[prost(string, repeated, tag = "15")]
    pub cert_sans: Vec<String>,
    #[prost(string, tag = "16")]
    pub source_pop: String,
    #[prost(string, tag = "17")]
    pub observed_at: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct L7Banner {
    #[prost(string, tag = "1")]
    pub event_id: String,
    #[prost(bytes, tag = "2")]
    pub target_ip: Vec<u8>,
    #[prost(uint32, tag = "3")]
    pub target_port: u32,
    #[prost(enumeration = "Transport", tag = "4")]
    pub transport: i32,
    #[prost(string, tag = "5")]
    pub protocol: String,
    #[prost(enumeration = "DetectionMethod", tag = "6")]
    pub detection_method: i32,
    #[prost(bytes, tag = "7")]
    pub raw_banner: Vec<u8>,
    #[prost(string, tag = "8")]
    pub sanitized_banner: String,
    #[prost(string, tag = "9")]
    pub banner_hash: String,
    #[prost(string, tag = "10")]
    pub http_method: String,
    #[prost(uint32, tag = "11")]
    pub http_status_code: u32,
    #[prost(string, tag = "12")]
    pub http_server: String,
    #[prost(string, tag = "13")]
    pub ssh_version: String,
    #[prost(string, tag = "14")]
    pub ssh_key_exchange_algorithms: String,
    #[prost(string, tag = "15")]
    pub ssh_host_key_type: String,
    #[prost(string, tag = "16")]
    pub db_version: String,
    #[prost(string, tag = "17")]
    pub source_pop: String,
    #[prost(string, tag = "18")]
    pub observed_at: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct EnrichedHost {
    #[prost(string, tag = "1")]
    pub ip_address: String,
    #[prost(string, tag = "2")]
    pub host_id: String,
    #[prost(uint32, tag = "3")]
    pub asn: u32,
    #[prost(string, tag = "4")]
    pub organization: String,
    #[prost(string, tag = "5")]
    pub bgp_prefix: String,
    #[prost(string, tag = "6")]
    pub bgp_as_path: String,
    #[prost(string, tag = "7")]
    pub reverse_dns: String,
    #[prost(string, tag = "8")]
    pub forward_dns: String,
    #[prost(enumeration = "GeoTier", tag = "9")]
    pub geo_tier_used: i32,
    #[prost(double, tag = "10")]
    pub geo_latitude: f64,
    #[prost(double, tag = "11")]
    pub geo_longitude: f64,
    #[prost(string, tag = "12")]
    pub geo_city: String,
    #[prost(string, tag = "13")]
    pub geo_country: String,
    #[prost(float, tag = "14")]
    pub geo_confidence: f32,
    #[prost(int64, tag = "15")]
    pub first_seen: i64,
    #[prost(int64, tag = "16")]
    pub last_seen: i64,
    #[prost(int32, tag = "17")]
    pub scan_count: i32,
    #[prost(enumeration = "HostStatus", tag = "18")]
    pub status: i32,
    #[prost(uint32, tag = "20")]
    pub port: u32,
    #[prost(enumeration = "Transport", tag = "21")]
    pub transport: i32,
    #[prost(string, tag = "22")]
    pub reclassified_protocol: String,
    #[prost(enumeration = "DetectionMethod", tag = "23")]
    pub detection_method: i32,
    #[prost(string, tag = "24")]
    pub sanitized_banner: String,
    #[prost(string, tag = "25")]
    pub banner_hash: String,
    #[prost(string, tag = "30")]
    pub ja4: String,
    #[prost(string, tag = "31")]
    pub ja4s: String,
    #[prost(string, tag = "32")]
    pub tls_version: String,
    #[prost(string, tag = "33")]
    pub cipher_suite: String,
    #[prost(string, tag = "34")]
    pub sni: String,
    #[prost(string, tag = "35")]
    pub alpn: String,
    #[prost(string, tag = "40")]
    pub http_method: String,
    #[prost(uint32, tag = "41")]
    pub http_status_code: u32,
    #[prost(string, tag = "42")]
    pub http_server: String,
    #[prost(string, tag = "43")]
    pub ssh_version: String,
    #[prost(string, tag = "44")]
    pub ssh_key_exchange_algorithms: String,
    #[prost(string, tag = "45")]
    pub ssh_host_key_type: String,
    #[prost(string, tag = "50")]
    pub cert_sha256: String,
    #[prost(string, tag = "51")]
    pub cert_serial: String,
    #[prost(string, tag = "52")]
    pub cert_subject_cn: String,
    #[prost(string, tag = "53")]
    pub cert_issuer_cn: String,
    #[prost(string, tag = "54")]
    pub cert_organization: String,
    #[prost(string, repeated, tag = "55")]
    pub cert_sans: Vec<String>,
    #[prost(int64, tag = "56")]
    pub cert_not_before: i64,
    #[prost(int64, tag = "57")]
    pub cert_not_after: i64,
    #[prost(bool, tag = "58")]
    pub cert_is_expired: bool,
    #[prost(string, tag = "59")]
    pub cert_rpki_status: String,
    #[prost(string, tag = "60")]
    pub cert_roa_prefix: String,
    #[prost(string, repeated, tag = "70")]
    pub cves: Vec<String>,
    #[prost(float, repeated, tag = "71")]
    pub cvss_scores: Vec<f32>,
    #[prost(string, tag = "80")]
    pub source_pop: String,
    #[prost(string, tag = "81")]
    pub event_id: String,
}

// ─── LZR Classification Messages ──────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct ClassifyRequest {
    #[prost(bytes, tag = "1")]
    pub payload: Vec<u8>,
    #[prost(string, tag = "2")]
    pub tls_hint: String,
    #[prost(bytes, tag = "3")]
    pub source_ip: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClassifyResponse {
    #[prost(string, tag = "1")]
    pub protocol: String,
    #[prost(string, tag = "2")]
    pub category: String,
    #[prost(uint32, tag = "3")]
    pub priority: u32,
    #[prost(bool, tag = "4")]
    pub matched: bool,
    #[prost(uint64, tag = "5")]
    pub latency_us: u64,
    #[prost(uint32, tag = "6")]
    pub signature_count: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClassifyBatchRequest {
    #[prost(message, repeated, tag = "1")]
    pub requests: Vec<ClassifyRequest>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClassifyBatchResponse {
    #[prost(message, repeated, tag = "1")]
    pub responses: Vec<ClassifyResponse>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Empty {}

#[derive(Clone, PartialEq, Message)]
pub struct HealthCheckResponse {
    #[prost(bool, tag = "1")]
    pub ready: bool,
    #[prost(uint32, tag = "2")]
    pub signature_count: u32,
    #[prost(string, tag = "3")]
    pub version: String,
}