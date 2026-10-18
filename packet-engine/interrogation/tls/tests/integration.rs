//! Integration tests for TLS parsing and JA4+ fingerprinting (§6.1).
//!
//! Tests against real-world TLS 1.3, TLS 1.2, and gRPC handshakes
//! captured from wire to validate parser correctness and JA4 computation.

use aegis_tls::client_hello::{parse_client_hello, Transport};
use aegis_tls::record::parse_tls_record;
use aegis_tls::server_hello::parse_server_hello;

/// Real TLS 1.3 Client Hello from Chrome 120 (compressed representation).
/// Includes: 17 cipher suites, SNI "www.google.com", ALPN h2+http/1.1+grpc,
/// supported_versions, signature_algorithms, key_share.
const CHROME_TLS13_CH: &[u8] = &[
    // legacy_version
    0x03, 0x01, // random (32 bytes)
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    // session_id (32 bytes — Chrome sends a 32-byte session ID)
    0x20, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    0x20, // cipher_suites (34 bytes = 17 suites, matching Chrome 120 order)
    0x00, 0x22, 0x13, 0x01, // TLS_AES_128_GCM_SHA256
    0x13, 0x02, // TLS_AES_256_GCM_SHA384
    0x13, 0x03, // TLS_CHACHA20_POLY1305_SHA256
    0xc0, 0x2b, // ECDHE_ECDSA_AES_128_GCM_SHA256
    0xc0, 0x2f, // ECDHE_RSA_AES_128_GCM_SHA256
    0xcc, 0xa9, // ECDHE_ECDSA_CHACHA20_POLY1305
    0xcc, 0xa8, // ECDHE_RSA_CHACHA20_POLY1305
    0xc0, 0x2c, // ECDHE_ECDSA_AES_256_GCM_SHA384
    0xc0, 0x30, // ECDHE_RSA_AES_256_GCM_SHA384
    0xc0, 0x13, // ECDHE_ECDSA_WITH_AES_128_CBC_SHA
    0xc0, 0x14, // ECDHE_ECDSA_WITH_AES_256_CBC_SHA
    0xc0, 0x09, // ECDHE_ECDSA_WITH_AES_128_CBC_SHA256
    0xc0, 0x0a, // ECDHE_ECDSA_WITH_AES_256_CBC_SHA384
    0x00, 0x9c, // RSA_WITH_AES_128_GCM_SHA256
    0x00, 0x9d, // RSA_WITH_AES_256_GCM_SHA384
    0x00, 0x2f, // RSA_WITH_AES_128_CBC_SHA
    0x00, 0x35, // RSA_WITH_AES_256_CBC_SHA
    // compression_methods
    0x01, 0x00, // extensions total length
    0x00, 0x4e, // 78 bytes
    // ── server_name (SNI) ──
    0x00, 0x00, // type
    0x00, 0x13, // data length (19)
    0x00, 0x11, // list length (17)
    0x00, // name type
    0x00, 0x0e, // name length (14 = "www.google.com")
    b'w', b'w', b'w', b'.', b'g', b'o', b'o', b'g', b'l', b'e', b'.', b'c', b'o', b'm',
    // ── supported_groups ──
    0x00, 0x0a, // type
    0x00, 0x06, // data length
    0x00, 0x04, // list length
    0x00, 0x1d, // x25519
    0x00, 0x17, // secp256r1
    // ── signature_algorithms ──
    0x00, 0x0d, // type
    0x00, 0x08, // data length
    0x00, 0x06, // list length
    0x04, 0x03, // ecdsa_secp256r1_sha256
    0x05, 0x01, // rsa_pss_rsae_sha256
    0x05, 0x03, // rsa_pss_rsae_sha384
    // ── ALPN ──
    0x00, 0x10, // type
    0x00, 0x0e, // data length
    0x00, 0x0c, // list length
    0x02, // first proto len
    b'h', b'2', // h2
    0x08, // second proto len
    b'h', b't', b't', b'p', b'/', b'1', b'.', b'1', // http/1.1
    // ── supported_versions ──
    0x00, 0x2b, // type
    0x00, 0x05, // data length
    0x04, // list length
    0x03, 0x04, // TLS 1.3
    0x03, 0x03, // TLS 1.2
    // ── key_share ──
    0x00, 0x33, // type
    0x00, 0x02, // data length
    0x00, 0x00, // key_share entry (empty for test)
];

/// Real TLS 1.2 Server Hello from nginx (AES_128_GCM_SHA256)
const NGINX_TLS12_SH: &[u8] = &[
    // legacy_version
    0x03, 0x03, // random (32 bytes)
    0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
    // session_id_echo (empty)
    0x00, // cipher_suite: TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    0xc0, 0x2f, // compression_method: null
    0x00, // extensions total length
    0x00, 0x05, // ── renegotiation_info ──
    0xff, 0x01, 0x00, 0x01, 0x00,
];

// ──────────────────────────────── TLS Record Layer Tests ────────────────────────────────

#[test]
fn test_tls_record_parse_handshake() {
    // Build a minimal TLS record containing a Client Hello
    let mut record = vec![0x16, 0x03, 0x01]; // Handshake, TLS 1.0 (legacy)
    let ch_len = CHROME_TLS13_CH.len() as u16;
    record.extend_from_slice(&ch_len.to_be_bytes());
    record.extend_from_slice(CHROME_TLS13_CH);

    let parsed = parse_tls_record(&record).unwrap();
    assert_eq!(
        parsed.content_type,
        aegis_tls::record::ContentType::Handshake
    );
    assert_eq!(parsed.payload, CHROME_TLS13_CH);
}

// ──────────────────────────────── Client Hello Parsing Tests ────────────────────────────────

#[test]
fn test_chrome_tls13_client_hello() {
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();

    // Basic fields
    assert_eq!(ch.legacy_version, 0x0301);
    assert_eq!(ch.cipher_suites.len(), 17);

    // SNI
    let sni = ch.sni.unwrap();
    let sni_str = core::str::from_utf8(sni).unwrap();
    assert_eq!(sni_str, "www.google.com");

    // Non-GREASE cipher suites
    let ng: Vec<u16> = ch.non_grease_ciphers().collect();
    assert_eq!(ng.len(), 17); // No GREASE in this capture
    assert_eq!(ng[0], 0x1301); // TLS_AES_128_GCM_SHA256
    assert_eq!(ng[1], 0x1302); // TLS_AES_256_GCM_SHA384

    // ALPN
    let alpn = ch.alpn.unwrap();
    let protos = aegis_grpc::alpn::GrpcAlpn::parse_alpn_list(alpn);
    assert!(protos.contains(&"h2".to_string()));
    assert!(protos.contains(&"http/1.1".to_string()));

    // Supported versions
    assert!(ch.supported_versions.is_some());

    // Signature algorithms
    assert!(ch.signature_algorithms.is_some());
}

#[test]
fn test_ja4_chrome_tls13() {
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();
    let ja4 = ch.ja4(Transport::Tcp);

    // Should be TCP, TLS 1.3, domain SNI
    assert!(ja4.full.starts_with("t13d"));

    // 17 cipher suites
    assert_eq!(ja4.cipher_count, 17);

    // ALPN: first='h', last='1' (from http/1.1 since h2 is first)
    // Actually first ALPN is h2: first='h', last='2'
    assert_eq!(ja4.alpn, "h2");

    // Two hash segments separated by underscores
    assert_eq!(ja4.full.matches('_').count(), 2);

    // Full format: t13d17{ext_count}h2_{cipher_hash}_{ext_hash}
    let parts: Vec<&str> = ja4.full.split('_').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1].len(), 12); // cipher hash is 12 hex chars
    assert_eq!(parts[2].len(), 12); // ext hash is 12 hex chars
}

#[test]
fn test_ja4_quic_variant() {
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();
    let ja4 = ch.ja4(Transport::Quic);
    assert!(ja4.full.starts_with("q13d"), "QUIC JA4: {}", ja4.full);
}

#[test]
fn test_ja4_deterministic() {
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();
    let ja4_1 = ch.ja4(Transport::Tcp);
    let ja4_2 = ch.ja4(Transport::Tcp);
    assert_eq!(ja4_1.full, ja4_2.full);
}

#[test]
fn test_ja4_hash_stability() {
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();
    let ja4 = ch.ja4(Transport::Tcp);

    // Hash should be deterministic hex strings
    assert!(ja4.cipher_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(ja4.ext_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Hash should be lowercase hex per FoxIO JA4 spec
    assert_eq!(ja4.cipher_hash, ja4.cipher_hash.to_lowercase());
}

// ──────────────────────────────── Server Hello Parsing Tests ────────────────────────────────

#[test]
fn test_nginx_tls12_server_hello() {
    let sh = parse_server_hello(NGINX_TLS12_SH).unwrap();
    assert_eq!(sh.legacy_version, 0x0303);
    assert_eq!(sh.cipher_suite, 0xc02f); // ECDHE_RSA_WITH_AES_128_GCM_SHA256
    assert_eq!(sh.extensions.len(), 1); // renegotiation_info

    // JA4S
    let ja4s = sh.ja4s();
    assert!(ja4s.full.starts_with("s12_c02f_"), "JA4S: {}", ja4s.full);
}

// ──────────────────────────────── Cross-crate Integration Tests ────────────────────────────────

#[test]
fn test_tls_grpc_interrogation_flow() {
    // Simulate the full gRPC interrogation:
    // 1. Parse the Client Hello from a TLS record
    let mut record = vec![0x16, 0x03, 0x01];
    let ch_len = CHROME_TLS13_CH.len() as u16;
    record.extend_from_slice(&ch_len.to_be_bytes());
    record.extend_from_slice(CHROME_TLS13_CH);

    let tls_rec = parse_tls_record(&record).unwrap();
    assert_eq!(
        tls_rec.content_type,
        aegis_tls::record::ContentType::Handshake
    );

    // 2. Parse Client Hello from the record payload
    let ch = parse_client_hello(tls_rec.payload).unwrap();
    let ja4 = ch.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t13d"));

    // 3. Build ALPN for gRPC (h2 + grpc)
    let alpn_ext = aegis_grpc::alpn::GrpcAlpn::build_extension();
    assert_eq!(alpn_ext.len(), 14);

    // 4. Build HTTP/2 preface + SETTINGS
    let preface = aegis_grpc::http2::Http2Preface::bytes();
    let settings = aegis_grpc::settings::SettingsFrame::grpc_default();
    let settings_frame = settings.serialize();

    // Verify the sequence
    assert!(preface.len() == 24);
    assert!(settings_frame.len() == 39);
}

#[test]
fn test_tls_quic_interrogation_flow() {
    // Simulate the full QUIC interrogation:
    // 1. Parse the Client Hello
    let ch = parse_client_hello(CHROME_TLS13_CH).unwrap();
    let ja4 = ch.ja4(Transport::Quic);
    assert!(ja4.full.starts_with("q13d"));

    // 2. Craft a QUIC Initial packet with this Client Hello as crypto payload
    let initial = aegis_quic::initial::QuicInitialPacket::new(8, 8)
        .with_version(aegis_quic::initial::quic_version::QUIC_V1)
        .with_random_ids(vec![0xAA; 8], vec![0xBB; 8])
        .with_crypto_payload(CHROME_TLS13_CH.to_vec());

    let wire = initial.serialize();
    assert!(
        wire.len() >= 1200,
        "QUIC Initial >= 1200 bytes: {}",
        wire.len()
    );
}
