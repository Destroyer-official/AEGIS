//! # aegis-proto
//!
//! Canonical protobuf types for the AEGIS intelligence-driven internet scanner.
//!
//! This crate is the **single source of truth** for all cross-crate message types.
//! Every component in the AEGIS workspace depends on this package for shared types.
//!
//! ## Message Types
//!
//! | Type | Kafka Topic | Direction |
//! |------|-------------|-----------|
//! | [`ScanCommand`] | `scan-commands` | Targeting → Scanner |
//! | [`L4Response`] | `raw-l4-scans` | Scanner → Kafka |
//! | [`TLSFingerprint`] | `tls-fingerprints` | Scanner → Kafka |
//! | [`L7Banner`] | `l7-banners` | Scanner → Kafka |
//! | [`EnrichedHost`] | — | Flink → Iceberg |
//!
//! ## Version
//!
//! v0.1.0 — Frozen 2026-08-30. See `aegis_types.proto` for field-level guarantees.

#[path = "aegis_types.rs"]
pub mod types;

// Re-export all types at crate root for convenience
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn scan_command_roundtrip() {
        let cmd = ScanCommand {
            command_id: 42,
            target_ip: vec![192, 168, 1, 1],
            port: 443,
            transport: Transport::Tcp as i32,
            priority: 0.95,
            source: TargetSource::GpsCoOccurrence as i32,
            conditional_probability: 0.87,
            asn: 13335,
            feature_tuple: vec![443, 80],
            generated_at: "2026-08-30T10:00:00Z".into(),
            cidr_block: "192.168.0.0/16".into(),
            asn_name: "CLOUDFLARENET".into(),
        };

        let mut buf = Vec::new();
        cmd.encode(&mut buf).unwrap();

        let decoded = ScanCommand::decode(buf.as_slice()).unwrap();
        assert_eq!(decoded.command_id, 42);
        assert_eq!(decoded.port, 443);
        assert_eq!(decoded.asn, 13335);
        assert_eq!(decoded.transport, Transport::Tcp as i32);
    }

    #[test]
    fn l4_response_defaults() {
        let resp = L4Response::default();
        assert_eq!(resp.target_port, 0);
        assert_eq!(resp.result, L4Result::Unspecified as i32);
    }

    #[test]
    fn tls_fingerprint_ja4_format() {
        let fp = TLSFingerprint {
            ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1".into(),
            ..Default::default()
        };
        assert!(fp.ja4.starts_with('t'));
        assert!(fp.ja4.contains('_'));
    }

    #[test]
    fn l7_banner_protocol_detection() {
        let banner = L7Banner {
            protocol: "SSH".into(),
            detection_method: DetectionMethod::LzrHeuristic as i32,
            ..Default::default()
        };
        assert_eq!(banner.protocol, "SSH");
        assert_eq!(banner.detection_method, DetectionMethod::LzrHeuristic as i32);
    }

    #[test]
    fn enriched_host_iceberg_schema() {
        let host = EnrichedHost {
            ip_address: "1.2.3.4".into(),
            host_id: "test-uuid".into(),
            asn: 13335,
            status: HostStatus::Active as i32,
            geo_tier_used: GeoTier::Cryptographic as i32,
            geo_latitude: 37.7749,
            geo_longitude: -122.4194,
            geo_confidence: 1.0,
            ..Default::default()
        };

        let mut buf = Vec::new();
        host.encode(&mut buf).unwrap();
        let decoded = EnrichedHost::decode(buf.as_slice()).unwrap();
        assert_eq!(decoded.ip_address, "1.2.3.4");
        assert_eq!(decoded.asn, 13335);
        assert!((decoded.geo_latitude - 37.7749).abs() < f64::EPSILON);
    }
}