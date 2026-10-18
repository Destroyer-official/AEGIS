//! Integration tests for gRPC / HTTP/2 ALPN negotiation (§6.3).
//!
//! Tests the complete gRPC interrogation sequence:
//! 1. TLS Client Hello with ALPN h2+grpc
//! 2. HTTP/2 connection preface
//! 3. SETTINGS frame

use aegis_grpc::alpn::GrpcAlpn;
use aegis_grpc::http2::Http2Preface;
use aegis_grpc::settings::SettingsFrame;

/// Simulate a complete gRPC TLS handshake sequence.
#[test]
fn test_grpc_interrogation_sequence() {
    // Step 1: TLS Client Hello ALPN extension
    let alpn_ext = GrpcAlpn::build_extension();
    assert_eq!(alpn_ext.len(), 14);
    let ext_type = u16::from_be_bytes([alpn_ext[0], alpn_ext[1]]);
    assert_eq!(ext_type, 0x0010);

    // Step 2: After TLS handshake, send HTTP/2 preface
    let preface = Http2Preface::bytes();
    assert_eq!(preface.len(), 24);

    // Step 3: Send SETTINGS frame
    let settings = SettingsFrame::grpc_default();
    let settings_frame = settings.serialize();
    assert_eq!(settings_frame.len(), 39); // 9 header + 30 payload

    // Verify the complete post-TLS sequence is well-formed
    let mut post_tls = Vec::new();
    post_tls.extend_from_slice(preface);
    post_tls.extend_from_slice(&settings_frame);

    // Preface should be first 24 bytes
    assert!(Http2Preface::validate(&post_tls));
    // Settings frame should follow
    let frame_type = post_tls[24 + 3];
    assert_eq!(frame_type, 0x04); // SETTINGS
}

/// Verify ALPN data contains exactly h2 and grpc.
#[test]
fn test_alpn_h2_grpc_only() {
    let protos = GrpcAlpn::parse_alpn_list(&GrpcAlpn::ALPN_DATA);
    assert_eq!(protos.len(), 2);
    assert_eq!(protos[0], "h2");
    assert_eq!(protos[1], "grpc");
}

/// Test SETTINGS frame with all required gRPC parameters.
#[test]
fn test_grpc_settings_frame_structure() {
    let sf = SettingsFrame::grpc_default();
    let frame = sf.serialize();

    // Verify SETTINGS frame type
    assert_eq!(frame[3], 0x04);

    // Verify stream identifier is 0 (connection-level)
    assert_eq!(frame[5], 0x00);
    assert_eq!(frame[6], 0x00);
    assert_eq!(frame[7], 0x00);
    assert_eq!(frame[8], 0x00);

    // Parse and verify individual settings
    let parsed = SettingsFrame::parse_body(&frame[9..]).unwrap();
    let settings: std::collections::HashMap<u16, u32> =
        parsed.settings.iter().map(|s| (s.id, s.value)).collect();

    // ENABLE_PUSH = 0 (gRPC doesn't use server push)
    assert_eq!(settings.get(&0x0002), Some(&0));
    // MAX_CONCURRENT_STREAMS = 100
    assert_eq!(settings.get(&0x0003), Some(&100));
    // INITIAL_WINDOW_SIZE = 1MB
    assert_eq!(settings.get(&0x0004), Some(&1048576));
}

/// Test that the complete gRPC probe bytes are correct for sending on the wire.
#[test]
fn test_grpc_probe_wire_bytes() {
    let mut probe = Vec::new();

    // HTTP/2 connection preface
    probe.extend_from_slice(Http2Preface::bytes());

    // SETTINGS frame
    let settings = SettingsFrame::grpc_default();
    probe.extend_from_slice(&settings.serialize());

    // Total: 24 (preface) + 39 (settings) = 63 bytes
    assert_eq!(probe.len(), 63);

    // Verify preface magic
    assert_eq!(&probe[..24], b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");

    // Verify SETTINGS frame length field (30 bytes payload)
    assert_eq!(probe[24], 0x00);
    assert_eq!(probe[25], 0x00);
    assert_eq!(probe[26], 0x1e); // 30 decimal
}

// ══════════════════════════════════════════════════════════════════════════════
//  Day 3: gRPC ALPN Mutual Integration — End-to-End TLS→JA4→gRPC Pipeline
// ══════════════════════════════════════════════════════════════════════════════

/// Simulate a real gRPC-over-TLS handshake with JA4 extraction.
///
/// This test exercises the full pipeline:
/// 1. Build a realistic TLS 1.3 Client Hello (like grpc-go / envoy would send)
/// 2. Parse the Client Hello with aegis-tls
/// 3. Extract JA4 fingerprint
/// 4. Verify ALPN contains h2+grpc
/// 5. Build the post-TLS gRPC preface + SETTINGS
/// 6. Validate the complete wire sequence
#[test]
fn test_grpc_alpn_ja4_pipeline() {
    // ── Step 1: Build a realistic gRPC Client Hello ──
    let mut ch = Vec::new();
    ch.extend_from_slice(&0x0301u16.to_be_bytes()); // legacy_version

    // Random (32 bytes)
    ch.extend_from_slice(&[
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ]);

    // Session ID (empty)
    ch.push(0x00);

    // Cipher suites (grpc-go default: TLS 1.3 ciphers only)
    let cipher_suites: Vec<u16> = vec![
        0x1301, // TLS_AES_128_GCM_SHA256
        0x1302, // TLS_AES_256_GCM_SHA384
        0x1303, // TLS_CHACHA20_POLY1305_SHA256
    ];
    ch.extend_from_slice(&(cipher_suites.len() as u16 * 2).to_be_bytes());
    for cs in &cipher_suites {
        ch.extend_from_slice(&cs.to_be_bytes());
    }

    // Compression methods
    ch.push(0x01);
    ch.push(0x00);

    // Extensions
    let mut exts = Vec::new();

    // SNI: grpc.io
    let sni = b"grpc.io";
    let sni_list_len = (1 + 2 + sni.len()) as u16;
    let sni_data_len = 2 + sni_list_len;
    exts.extend_from_slice(&0x0000u16.to_be_bytes()); // type
    exts.extend_from_slice(&sni_data_len.to_be_bytes());
    exts.extend_from_slice(&sni_list_len.to_be_bytes());
    exts.push(0x00); // host_name
    exts.extend_from_slice(&(sni.len() as u16).to_be_bytes());
    exts.extend_from_slice(sni);

    // Supported groups
    let groups: Vec<u16> = vec![0x001d, 0x0017]; // x25519, secp256r1
    let groups_list_len = (groups.len() * 2) as u16;
    let groups_data_len = (2 + groups_list_len) as u16;
    exts.extend_from_slice(&0x000au16.to_be_bytes());
    exts.extend_from_slice(&groups_data_len.to_be_bytes());
    exts.extend_from_slice(&groups_list_len.to_be_bytes());
    for g in &groups {
        exts.extend_from_slice(&g.to_be_bytes());
    }

    // Signature algorithms
    let sig_algos: Vec<u16> = vec![0x0403, 0x0501, 0x0503];
    let sig_list_len = (sig_algos.len() * 2) as u16;
    let sig_data_len = (2 + sig_list_len) as u16;
    exts.extend_from_slice(&0x000du16.to_be_bytes());
    exts.extend_from_slice(&sig_data_len.to_be_bytes());
    exts.extend_from_slice(&sig_list_len.to_be_bytes());
    for a in &sig_algos {
        exts.extend_from_slice(&a.to_be_bytes());
    }

    // ALPN: h2 + grpc (using GrpcAlpn builder)
    exts.extend_from_slice(&GrpcAlpn::build_extension());

    // Supported versions: TLS 1.3 + TLS 1.2
    exts.extend_from_slice(&[0x00, 0x2b, 0x00, 0x05, 0x04, 0x03, 0x04, 0x03, 0x03]);

    // Key share (x25519)
    let key_share_entry: Vec<u8> = vec![0x00, 0x1d, 0x00, 0x20]; // group + key_exchange_len
    let ks_data_len = key_share_entry.len() as u16;
    exts.extend_from_slice(&0x0033u16.to_be_bytes());
    exts.extend_from_slice(&ks_data_len.to_be_bytes());
    exts.extend_from_slice(&key_share_entry);

    // extensions length + data
    ch.extend_from_slice(&(exts.len() as u16).to_be_bytes());
    ch.extend_from_slice(&exts);

    // ── Step 2: Parse the Client Hello ──
    let parsed = aegis_tls::client_hello::parse_client_hello(&ch).unwrap();

    // ── Step 3: Extract JA4 fingerprint ──
    let ja4 = parsed.ja4(aegis_tls::client_hello::Transport::Tcp);

    // Verify JA4 format
    assert!(
        ja4.full.starts_with("t13d"),
        "JA4 should be t13d: {}",
        ja4.full
    );
    assert_eq!(ja4.cipher_count, 3);
    assert_eq!(ja4.alpn, "h2");

    // ── Step 4: Verify ALPN contains h2+grpc ──
    let alpn_data = parsed.alpn.unwrap();
    let protos = GrpcAlpn::parse_alpn_list(alpn_data);
    assert_eq!(protos.len(), 2);
    assert_eq!(protos[0], "h2");
    assert_eq!(protos[1], "grpc");

    // ── Step 5: Build post-TLS gRPC sequence ──
    let preface = Http2Preface::bytes();
    let settings = SettingsFrame::grpc_default();
    let settings_frame = settings.serialize();

    // Verify preface
    assert!(Http2Preface::validate(preface));

    // Verify SETTINGS
    assert_eq!(settings_frame[3], 0x04); // SETTINGS type
    assert_eq!(settings_frame.len(), 39); // 9 header + 30 payload

    // ── Step 6: Validate complete wire sequence ──
    let mut wire = Vec::new();
    wire.extend_from_slice(preface);
    wire.extend_from_slice(&settings_frame);
    assert_eq!(wire.len(), 63);

    // First 24 bytes must be the preface
    assert_eq!(&wire[..24], b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
    // Byte 24-26: frame length = 30
    assert_eq!(wire[24], 0x00);
    assert_eq!(wire[25], 0x00);
    assert_eq!(wire[26], 0x1e);
    // Byte 27: frame type = SETTINGS
    assert_eq!(wire[27], 0x04);
}

/// Test gRPC Client Hello from different clients (grpc-go, envoy, grpcurl).
#[test]
fn test_grpc_client_variants() {
    // ── grpc-go style (modern) ──
    let grpc_go_alpn = GrpcAlpn::parse_alpn_list(&GrpcAlpn::ALPN_DATA);
    assert!(grpc_go_alpn.contains(&"h2".to_string()));
    assert!(grpc_go_alpn.contains(&"grpc".to_string()));

    // ── Build extension and re-parse ──
    let ext = GrpcAlpn::build_extension();
    let ext_data = &ext[4..]; // skip type + length
    let reparsed = GrpcAlpn::parse_alpn_list(ext_data);
    assert_eq!(grpc_go_alpn, reparsed);
}

/// Test gRPC ALPN mutual: build extension from scratch, parse, compare.
#[test]
fn test_grpc_alpn_roundtrip() {
    let ext = GrpcAlpn::build_extension();

    // Verify extension type
    let ext_type = u16::from_be_bytes([ext[0], ext[1]]);
    assert_eq!(ext_type, 0x0010);

    // Verify data length
    let data_len = u16::from_be_bytes([ext[2], ext[3]]);
    assert_eq!(data_len as usize, GrpcAlpn::ALPN_DATA.len());

    // Parse the ALPN list from the extension data
    let protos = GrpcAlpn::parse_alpn_list(&ext[4..]);
    assert_eq!(protos, vec!["h2", "grpc"]);

    // Build the same extension manually and verify byte-identical
    let mut manual_ext = Vec::new();
    manual_ext.extend_from_slice(&0x0010u16.to_be_bytes());
    let data_len = GrpcAlpn::ALPN_DATA.len() as u16;
    manual_ext.extend_from_slice(&data_len.to_be_bytes());
    manual_ext.extend_from_slice(&GrpcAlpn::ALPN_DATA);
    assert_eq!(ext.to_vec(), manual_ext);
}

/// Test that SETTINGS frame round-trips correctly through parse.
#[test]
fn test_grpc_settings_roundtrip() {
    let sf = SettingsFrame::grpc_default();
    let frame = sf.serialize();
    let parsed = SettingsFrame::parse_body(&frame[9..]).unwrap();

    assert_eq!(parsed.settings.len(), 5);
    for (original, parsed_s) in sf.settings.iter().zip(parsed.settings.iter()) {
        assert_eq!(original, parsed_s);
    }
}

/// Test gRPC preface validation edge cases.
#[test]
fn test_grpc_preface_validation() {
    // Valid preface
    assert!(Http2Preface::validate(Http2Preface::bytes()));

    // Invalid: wrong magic
    let mut bad = vec![0u8; 24];
    bad[..13].copy_from_slice(b"PRI * HTTP/1.");
    assert!(!Http2Preface::validate(&bad));

    // Invalid: too short
    assert!(!Http2Preface::validate(&[0u8; 10]));

    // Invalid: empty
    assert!(!Http2Preface::validate(&[]));
}

/// Test complete gRPC handshake with extra SETTINGS (custom parameters).
#[test]
fn test_grpc_custom_settings() {
    let settings = SettingsFrame {
        settings: vec![
            aegis_grpc::settings::Setting {
                id: 0x0002,
                value: 0,
            }, // ENABLE_PUSH
            aegis_grpc::settings::Setting {
                id: 0x0003,
                value: 200,
            }, // MAX_CONCURRENT_STREAMS
            aegis_grpc::settings::Setting {
                id: 0x0004,
                value: 2097152,
            }, // INITIAL_WINDOW_SIZE = 2MB
            aegis_grpc::settings::Setting {
                id: 0x0005,
                value: 32768,
            }, // MAX_FRAME_SIZE = 32KB
            aegis_grpc::settings::Setting {
                id: 0x0006,
                value: 131072,
            }, // MAX_HEADER_LIST_SIZE = 128KB
        ],
    };
    let frame = settings.serialize();
    let parsed = SettingsFrame::parse_body(&frame[9..]).unwrap();

    // Verify custom values
    let map: std::collections::HashMap<u16, u32> =
        parsed.settings.iter().map(|s| (s.id, s.value)).collect();
    assert_eq!(map.get(&0x0003), Some(&200));
    assert_eq!(map.get(&0x0004), Some(&2097152));
    assert_eq!(map.get(&0x0005), Some(&32768));
    assert_eq!(map.get(&0x0006), Some(&131072));
}
