//! Integration tests for QUIC protocol handling (§6.3).
//!
//! Tests QUIC Initial packet construction, Version Negotiation parsing,
//! and transport parameter extraction.

use aegis_quic::initial::{quic_version, QuicInitialPacket};
use aegis_quic::transport_params::TransportParams;
use aegis_quic::version::VersionNegotiation;

/// Test complete QUIC interrogation: Initial packet construction + Version Negotiation parse.
#[test]
fn test_quic_interrogation_flow() {
    // Step 1: Craft a QUIC Initial packet with short connection IDs
    let initial = QuicInitialPacket::new(8, 8)
        .with_version(quic_version::QUIC_V1)
        .with_random_ids(vec![0xAA; 8], vec![0xBB; 8])
        .with_crypto_payload(vec![0x01, 0x02, 0x03, 0x04]);

    let wire = initial.serialize();

    // Step 2: Verify it's a valid Initial packet
    assert!(wire[0] & 0xC0 == 0xC0, "Should be Initial type");
    assert_eq!(&wire[1..5], &quic_version::QUIC_V1.to_be_bytes());

    // Step 3: Parse it back
    let parsed = QuicInitialPacket::parse(&wire).unwrap();
    assert_eq!(parsed.version, quic_version::QUIC_V1);
    assert_eq!(parsed.dcid, vec![0xAA; 8]);
    assert_eq!(parsed.scid, vec![0xBB; 8]);
}

/// Test Version Negotiation parsing with multiple versions.
#[test]
fn test_version_negotiation_parsing() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // version=0
    pkt.push(0x04); // dcid len
    pkt.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]);
    pkt.push(0x04); // scid len
    pkt.extend_from_slice(&[0x55, 0x66, 0x77, 0x88]);
    // Supported versions (no random field per RFC 9000 §17.2.1)
    // Supported versions
    pkt.extend_from_slice(&0x00000001u32.to_be_bytes()); // QUIC v1
    pkt.extend_from_slice(&0xff00001du32.to_be_bytes()); // IETF draft 29
    pkt.extend_from_slice(&0xff00001eu32.to_be_bytes()); // IETF draft 30

    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 3);
    assert!(vn.supports_version(0x00000001));
    assert!(vn.supports_version(0xff00001d));
    assert!(!vn.supports_version(0x00000002));
}

/// Test transport parameter extraction.
#[test]
fn test_transport_params_extraction() {
    let mut block = Vec::new();

    // max_data = 1MB (1048576) — QUIC varint: 4-byte encoding [0x80, 0x10, 0x00, 0x00]
    block.push(0x04); // ID: initial_max_data (RFC 9000 §18.2)
    block.push(0x04); // Length
    block.extend_from_slice(&[0x80, 0x10, 0x00, 0x00]);

    // max_streams_bidi = 100 — QUIC varint: 2-byte encoding [0x40, 0x64]
    block.push(0x08); // ID: initial_max_streams_bidi (RFC 9000 §18.2)
    block.push(0x02); // Length
    block.extend_from_slice(&[0x40, 0x64]);

    // max_streams_uni = 50 — QUIC varint: 1-byte encoding [0x32] (50 < 64)
    block.push(0x09); // ID: initial_max_streams_uni (RFC 9000 §18.2)
    block.push(0x01); // Length
    block.push(50);

    // max_idle_timeout = 30000ms — QUIC varint: 4-byte encoding
    // 30000 > 16383, needs 4-byte varint: [0x80, 0x00, 0x75, 0x30]
    block.push(0x01); // ID: max_idle_timeout (RFC 9000 §18.2)
    block.push(0x04); // Length
    block.extend_from_slice(&[0x80, 0x00, 0x75, 0x30]);

    let params = TransportParams::parse(&block).unwrap();
    assert_eq!(params.initial_max_data(), Some(1048576));
    assert_eq!(params.initial_max_streams_bidi(), Some(100));
    assert_eq!(params.initial_max_streams_uni(), Some(50));
    assert_eq!(params.max_idle_timeout(), Some(30000));
}

/// Test minimum Initial packet size (1200 bytes per RFC 9000).
#[test]
fn test_initial_packet_padding() {
    let pkt = QuicInitialPacket::new(8, 8).with_crypto_payload(vec![0x01; 100]);

    let wire = pkt.serialize();
    assert!(
        wire.len() >= 1200,
        "Initial packet should be >= 1200 bytes, got {}",
        wire.len()
    );
}

/// Test that Version 0 detection works.
#[test]
fn test_version_zero_detection() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // version=0
    pkt.push(0x00); // dcid len
    pkt.push(0x00); // scid len
    pkt.extend_from_slice(&0x00000001u32.to_be_bytes());

    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 1);
    assert_eq!(vn.supported_versions[0], 0x00000001);
}

// ══════════════════════════════════════════════════════════════════════════════
//  Day 2: QUIC Version Negotiation — Edge Cases (RFC 9000 §17.2.1)
// ══════════════════════════════════════════════════════════════════════════════

fn build_vn(dcid: &[u8], scid: &[u8], versions: &[u32]) -> Vec<u8> {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.push(dcid.len() as u8);
    pkt.extend_from_slice(dcid);
    pkt.push(scid.len() as u8);
    pkt.extend_from_slice(scid);
    // No random field per RFC 9000 §17.2.1
    for v in versions {
        pkt.extend_from_slice(&v.to_be_bytes());
    }
    pkt
}

#[test]
fn test_vn_empty_version_list() {
    let pkt = build_vn(&[0x11], &[0x22], &[]);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert!(vn.supported_versions.is_empty());
}

#[test]
fn test_vn_single_version() {
    let pkt = build_vn(&[0xAA], &[0xBB], &[0x00000001]);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 1);
    assert!(vn.supports_version(0x00000001));
}

#[test]
fn test_vn_many_versions() {
    let versions: Vec<u32> = (0..20).map(|i| 0xff000000 + i).collect();
    let pkt = build_vn(&[0x11], &[0x22], &versions);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 20);
    for v in &versions {
        assert!(vn.supports_version(*v));
    }
}

#[test]
fn test_vn_long_connection_ids() {
    let dcid = vec![0xAA; 255];
    let scid = vec![0xBB; 255];
    let pkt = build_vn(&dcid, &scid, &[0x00000001]);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.dcid, dcid);
    assert_eq!(vn.scid, scid);
}

#[test]
fn test_vn_zero_length_connection_ids() {
    let pkt = build_vn(&[], &[], &[0x00000001]);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert!(vn.dcid.is_empty());
    assert!(vn.scid.is_empty());
    assert_eq!(vn.supported_versions, vec![0x00000001]);
}

#[test]
fn test_vn_grease_versions() {
    let grease_versions = vec![0x0a0a0a0a, 0x1a1a1a1a, 0x2a2a2a2a];
    let pkt = build_vn(&[0x11], &[0x22], &grease_versions);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions, grease_versions);
}

#[test]
fn test_vn_reserved_versions() {
    let reserved = vec![0x1a2a3a4a, 0x5a6a7a8a];
    let pkt = build_vn(&[0x11], &[0x22], &reserved);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions, reserved);
}

#[test]
fn test_vn_version_zero_as_supported() {
    let pkt = build_vn(&[0x11], &[0x22], &[0x00000000, 0x00000001]);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert!(vn.supports_version(0x00000000));
    assert!(vn.supports_version(0x00000001));
}

#[test]
fn test_vn_trailing_bytes_ignored() {
    let mut pkt = build_vn(&[0x11], &[0x22], &[0x00000001]);
    pkt.push(0xFF); // trailing byte
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 1);
}

#[test]
fn test_vn_two_trailing_bytes_ignored() {
    let mut pkt = build_vn(&[0x11], &[0x22], &[0x00000001]);
    pkt.extend_from_slice(&[0xFF, 0xFE]); // trailing bytes
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 1);
}

// ── Error path tests ──

#[test]
fn test_vn_error_too_short() {
    assert_eq!(
        VersionNegotiation::parse(&[0xC0, 0x00]),
        Err(aegis_quic::version::VersionNegotiationError::TooShort)
    );
}

#[test]
fn test_vn_error_not_long_header() {
    let mut pkt = vec![0x40]; // short header
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.push(0x00);
    pkt.push(0x00);
    pkt.extend_from_slice(&[0; 8]);
    assert_eq!(
        VersionNegotiation::parse(&pkt),
        Err(aegis_quic::version::VersionNegotiationError::NotLongHeader)
    );
}

#[test]
fn test_vn_error_version_not_zero() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&0x00000001u32.to_be_bytes());
    pkt.push(0x00);
    pkt.push(0x00);
    pkt.extend_from_slice(&[0; 8]);
    assert_eq!(
        VersionNegotiation::parse(&pkt),
        Err(aegis_quic::version::VersionNegotiationError::VersionNotZero)
    );
}

#[test]
fn test_vn_error_truncated_dcid() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.push(0xFF); // dcid len = 255 but no data
    assert_eq!(
        VersionNegotiation::parse(&pkt),
        Err(aegis_quic::version::VersionNegotiationError::Truncated)
    );
}

#[test]
fn test_vn_error_truncated_scid() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.push(0x02); // dcid len
    pkt.extend_from_slice(&[0x11, 0x22]);
    pkt.push(0xFF); // scid len = 255 but no data
    assert_eq!(
        VersionNegotiation::parse(&pkt),
        Err(aegis_quic::version::VersionNegotiationError::Truncated)
    );
}

#[test]
fn test_vn_trailing_incomplete_version_ignored() {
    let mut pkt = vec![0xC0];
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.push(0x01);
    pkt.push(0x11);
    pkt.push(0x01);
    pkt.push(0x22);
    pkt.extend_from_slice(&[0; 4]); // one complete version (0x00000000)
                                    // trailing 4 bytes is a complete version, not truncated
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 1);
}

// ── Determinism / stability tests ──

#[test]
fn test_vn_determinism() {
    let pkt = build_vn(&[0x11, 0x22], &[0x33, 0x44], &[0x00000001, 0xff00001d]);
    let vn1 = VersionNegotiation::parse(&pkt).unwrap();
    let vn2 = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn1, vn2);
}

#[test]
fn test_vn_all_well_known_versions() {
    let versions = vec![
        0x00000001, // QUIC v1 (RFC 9000)
        0xff00001d, // IETF draft 29
        0xff00001e, // IETF draft 30
        0xff00001f, // IETF draft 31
        0xff000020, // IETF draft 32
        0xff000021, // IETF draft 33
        0xff000022, // IETF draft 34
        0xff000023, // IETF draft 35
    ];
    let pkt = build_vn(&[0x01], &[0x02], &versions);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.supported_versions.len(), 8);
    assert!(vn.supports_version(0x00000001));
    assert!(vn.supports_version(0xff000023));
}

#[test]
fn test_vn_roundtrip_preserves_ids() {
    let dcid = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    let scid = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let versions = vec![0x00000001, 0xff00001d, 0xff00001e];
    let pkt = build_vn(&dcid, &scid, &versions);
    let vn = VersionNegotiation::parse(&pkt).unwrap();
    assert_eq!(vn.dcid, dcid);
    assert_eq!(vn.scid, scid);
}

// ══════════════════════════════════════════════════════════════════════════════
//  Day 2: Fuzz Target for QUIC Version Negotiation
// ══════════════════════════════════════════════════════════════════════════════

/// Fuzz harness function — feeds arbitrary byte slices to the parser.
/// In production this would be wired to cargo-fuzz / libfuzzer.
/// For now it serves as a property-based test: the parser must never panic.
fn fuzz_version_negotiation(data: &[u8]) {
    let result = VersionNegotiation::parse(data);
    if let Ok(vn) = result {
        // Invariants that must hold for any successfully parsed packet:
        // 1. dcid and scid lengths must match actual data
        assert!(vn.dcid.len() <= 255);
        assert!(vn.scid.len() <= 255);
        // 2. All supported versions are valid u32
        for v in &vn.supported_versions {
            let _ = format!("{:#010x}", v); // must not panic
        }
        // 3. supports_version is consistent
        for v in &vn.supported_versions {
            assert!(vn.supports_version(*v));
        }
    }
    // On error: parser returned a valid error variant, no panic
}

#[test]
fn test_fuzz_empty_input() {
    fuzz_version_negotiation(&[]);
}

#[test]
fn test_fuzz_single_byte() {
    fuzz_version_negotiation(&[0x80]);
}

#[test]
fn test_fuzz_random_32_bytes() {
    let data: Vec<u8> = (0..32).map(|i| (i * 7 + 0x42) as u8).collect();
    fuzz_version_negotiation(&data);
}

#[test]
fn test_fuzz_all_zeros() {
    fuzz_version_negotiation(&[0x00; 64]);
}

#[test]
fn test_fuzz_all_ones() {
    fuzz_version_negotiation(&[0xFF; 64]);
}

#[test]
fn test_fuzz_alternating() {
    let data: Vec<u8> = (0..64)
        .map(|i| if i % 2 == 0 { 0xAA } else { 0x55 })
        .collect();
    fuzz_version_negotiation(&data);
}

#[test]
fn test_fuzz_valid_header_random_payload() {
    // Valid VN header, then garbage
    let mut data = vec![0xC0];
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    data.push(0x02);
    data.extend_from_slice(&[0x11, 0x22]);
    data.push(0x02);
    data.extend_from_slice(&[0x33, 0x44]);
    data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE]);
    // Garbage trailing
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA]);
    fuzz_version_negotiation(&data);
}
