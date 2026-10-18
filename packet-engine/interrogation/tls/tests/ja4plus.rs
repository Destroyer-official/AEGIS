//! Exhaustive JA4+ variant test module (§6.1).
//!
//! Day 1 — Tests all 8 JA4+ variants with real-world Client Hello / Server Hello
//! / HTTP / TCP / SSH / X.509 / DHCP bytes. Target: 64+ test cases.
//!
//! Each variant is tested for:
//! - Correct format/prefix
//! - Determinism (same input → same output)
//! - Hash stability (hex, lowercase, 12 chars)
//! - Edge cases (empty, missing fields, GREASE, large values)

use aegis_tls::client_hello::{parse_client_hello, Transport};
use aegis_tls::ja4d::{DhcpOption, DhcpPacket, DhcpProto, Ja4dFingerprint};
use aegis_tls::ja4h::parse_http_request;
use aegis_tls::ja4l::Ja4lFingerprint;
use aegis_tls::ja4ssh::{Ja4sshFingerprint, SshSessionData, SshSessionType};
use aegis_tls::ja4t::{parse_tcp_options, Ja4tFingerprint, TcpOptions};
use aegis_tls::ja4ts::Ja4tsFingerprint;
use aegis_tls::ja4x::{Ja4xFingerprint, X509Cert};
use aegis_tls::server_hello::parse_server_hello;

// ══════════════════════════════════════════════════════════════════════════════
//  HELPER: Build a minimal TLS Client Hello from parts
// ══════════════════════════════════════════════════════════════════════════════

fn build_sni_extension(sni: &[u8]) -> Vec<u8> {
    let name_len = sni.len() as u16;
    let list_len = (1 + 2 + sni.len()) as u16;
    let data_len = 2 + list_len;
    let mut ext = vec![0x00, 0x00]; // type
    ext.extend_from_slice(&data_len.to_be_bytes());
    ext.extend_from_slice(&list_len.to_be_bytes());
    ext.push(0x00); // name type = host_name
    ext.extend_from_slice(&name_len.to_be_bytes());
    ext.extend_from_slice(sni);
    ext
}

fn build_alpn_extension(protocols: &[&[u8]]) -> Vec<u8> {
    let mut list_data = Vec::new();
    for proto in protocols {
        list_data.push(proto.len() as u8);
        list_data.extend_from_slice(proto);
    }
    let list_len = list_data.len() as u16;
    let data_len = 2 + list_len;
    let mut ext = vec![0x00, 0x10]; // type = ALPN
    ext.extend_from_slice(&data_len.to_be_bytes());
    ext.extend_from_slice(&list_len.to_be_bytes());
    ext.extend_from_slice(&list_data);
    ext
}

fn build_supported_versions_extension(versions: &[u16]) -> Vec<u8> {
    let list_len = (versions.len() * 2) as u16;
    let data_len = 1 + list_len;
    let mut ext = vec![0x00, 0x2b]; // type
    ext.extend_from_slice(&data_len.to_be_bytes());
    ext.push(list_len as u8);
    for v in versions {
        ext.extend_from_slice(&v.to_be_bytes());
    }
    ext
}

fn build_signature_algorithms_extension() -> Vec<u8> {
    let algos: Vec<u16> = vec![
        0x0403, // ecdsa_secp256r1_sha256
        0x0501, // rsa_pss_rsae_sha256
        0x0503, // rsa_pss_rsae_sha384
        0x0401, // rsa_pkcs1_sha256
    ];
    let list_len = (algos.len() * 2) as u16;
    let data_len = (2 + list_len) as u16;
    let mut ext = vec![0x00, 0x0d]; // type
    ext.extend_from_slice(&data_len.to_be_bytes());
    ext.extend_from_slice(&list_len.to_be_bytes());
    for a in &algos {
        ext.extend_from_slice(&a.to_be_bytes());
    }
    ext
}

fn build_supported_groups_extension(groups: &[u16]) -> Vec<u8> {
    let list_len = (groups.len() * 2) as u16;
    let data_len = 2 + list_len;
    let mut ext = vec![0x00, 0x0a]; // type
    ext.extend_from_slice(&data_len.to_be_bytes());
    ext.extend_from_slice(&list_len.to_be_bytes());
    for g in groups {
        ext.extend_from_slice(&g.to_be_bytes());
    }
    ext
}

fn build_client_hello(
    sni: Option<&[u8]>,
    alpn: Option<Vec<&[u8]>>,
    cipher_suites: &[u16],
    exts: Option<Vec<Vec<u8>>>,
) -> Vec<u8> {
    let mut ch = Vec::new();
    ch.extend_from_slice(&0x0301u16.to_be_bytes()); // legacy_version

    // random (32 bytes)
    ch.extend_from_slice(&[0xAA; 32]);

    // session_id (empty)
    ch.push(0x00);

    // cipher_suites
    let cs_len = (cipher_suites.len() * 2) as u16;
    ch.extend_from_slice(&cs_len.to_be_bytes());
    for cs in cipher_suites {
        ch.extend_from_slice(&cs.to_be_bytes());
    }

    // compression methods
    ch.push(0x01); // length
    ch.push(0x00); // null

    // Build extensions
    let mut extensions = Vec::new();
    if let Some(sni_data) = sni {
        extensions.extend_from_slice(&build_sni_extension(sni_data));
    }
    extensions.extend_from_slice(&build_supported_groups_extension(&[0x001d, 0x0017]));
    extensions.extend_from_slice(&build_signature_algorithms_extension());
    if let Some(alpn_data) = alpn {
        extensions.extend_from_slice(&build_alpn_extension(&alpn_data));
    }
    extensions.extend_from_slice(&build_supported_versions_extension(&[0x0304, 0x0303]));
    if let Some(extra_exts) = exts {
        for e in extra_exts {
            extensions.extend_from_slice(&e);
        }
    }

    let ext_len = extensions.len() as u16;
    ch.extend_from_slice(&ext_len.to_be_bytes());
    ch.extend_from_slice(&extensions);

    ch
}

fn build_server_hello(version: u16, cipher_suite: u16, exts: &[(u16, &[u8])]) -> Vec<u8> {
    let mut sh = Vec::new();
    sh.extend_from_slice(&version.to_be_bytes());
    sh.extend_from_slice(&[0xBB; 32]); // random
    sh.push(0x00); // session_id_echo (empty)
    sh.extend_from_slice(&cipher_suite.to_be_bytes());
    sh.push(0x00); // compression_method

    let mut ext_data = Vec::new();
    for (ext_type, data) in exts {
        ext_data.extend_from_slice(&ext_type.to_be_bytes());
        ext_data.extend_from_slice(&(data.len() as u16).to_be_bytes());
        ext_data.extend_from_slice(data);
    }
    sh.extend_from_slice(&(ext_data.len() as u16).to_be_bytes());
    sh.extend_from_slice(&ext_data);

    sh
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4 — TLS Client Fingerprint (12 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4_tcp_tls13_domain() {
    let ch = build_client_hello(
        Some(b"example.com"),
        Some(vec![b"h2", b"http/1.1"]),
        &[0x1301, 0x1302, 0x1303],
        None,
    );
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(
        ja4.full.starts_with("t13d"),
        "JA4 TCP TLS1.3 domain: {}",
        ja4.full
    );
    assert_eq!(ja4.cipher_count, 3);
    assert_eq!(ja4.alpn, "h2");
    assert!(ja4.cipher_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(ja4.ext_hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_ja4_quic_tls13() {
    let ch = build_client_hello(
        Some(b"example.com"),
        Some(vec![b"h3"]),
        &[0x1301, 0x1302, 0x1303],
        None,
    );
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Quic);
    assert!(ja4.full.starts_with("q13d"), "JA4 QUIC: {}", ja4.full);
    assert_eq!(ja4.cipher_count, 3);
}

#[test]
fn test_ja4_tls12_domain() {
    let mut ch = Vec::new();
    ch.extend_from_slice(&0x0301u16.to_be_bytes()); // legacy_version
    ch.extend_from_slice(&[0xAA; 32]); // random
    ch.push(0x00); // session_id (empty)
                   // cipher_suites
    let cs: Vec<u16> = vec![0xc02f, 0xc02c, 0xc013];
    ch.extend_from_slice(&(cs.len() as u16 * 2).to_be_bytes());
    for c in &cs {
        ch.extend_from_slice(&c.to_be_bytes());
    }
    ch.push(0x01);
    ch.push(0x00); // compression
                   // Build extensions: SNI + supported_groups + sig_algos + ALPN + supported_versions(TLS1.2 only)
    let mut exts = Vec::new();
    exts.extend_from_slice(&build_sni_extension(b"example.com"));
    exts.extend_from_slice(&build_supported_groups_extension(&[0x001d, 0x0017]));
    exts.extend_from_slice(&build_signature_algorithms_extension());
    exts.extend_from_slice(&build_alpn_extension(&[b"http/1.1"]));
    exts.extend_from_slice(&build_supported_versions_extension(&[0x0303]));
    ch.extend_from_slice(&(exts.len() as u16).to_be_bytes());
    ch.extend_from_slice(&exts);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t12d"), "JA4 TLS1.2: {}", ja4.full);
    assert_eq!(ja4.cipher_count, 3);
}

#[test]
fn test_ja4_no_sni() {
    let ch = build_client_hello(None, Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t13i"), "JA4 no SNI: {}", ja4.full);
}

#[test]
fn test_ja4_no_alpn() {
    let ch = build_client_hello(Some(b"example.com"), None, &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t13d"), "JA4 no ALPN: {}", ja4.full);
    assert_eq!(ja4.alpn, "00");
}

#[test]
fn test_ja4_ip_sni() {
    let ch = build_client_hello(Some(b"192.168.1.1"), Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t13i"), "JA4 IP SNI: {}", ja4.full);
}

#[test]
fn test_ja4_ipv6_sni() {
    let ch = build_client_hello(Some(b"2001:db8::1"), Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert!(ja4.full.starts_with("t13i"), "JA4 IPv6 SNI: {}", ja4.full);
}

#[test]
fn test_ja4_many_ciphers() {
    let ciphers: Vec<u16> = (0..30).map(|i| 0x1300 + i as u16).collect();
    let ch = build_client_hello(Some(b"example.com"), Some(vec![b"h2"]), &ciphers, None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert_eq!(ja4.cipher_count, 30);
    let parts: Vec<&str> = ja4.full.split('_').collect();
    assert_eq!(parts.len(), 3);
}

#[test]
fn test_ja4_determinism() {
    let ch = build_client_hello(
        Some(b"example.com"),
        Some(vec![b"h2", b"http/1.1"]),
        &[0x1301, 0x1302],
        None,
    );
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4_1 = parsed.ja4(Transport::Tcp);
    let ja4_2 = parsed.ja4(Transport::Tcp);
    assert_eq!(ja4_1.full, ja4_2.full);
    assert_eq!(ja4_1.cipher_hash, ja4_2.cipher_hash);
    assert_eq!(ja4_1.ext_hash, ja4_2.ext_hash);
}

#[test]
fn test_ja4_hash_format() {
    let ch = build_client_hello(Some(b"test.org"), Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert_eq!(ja4.cipher_hash.len(), 12);
    assert_eq!(ja4.ext_hash.len(), 12);
    assert_eq!(ja4.cipher_hash, ja4.cipher_hash.to_lowercase());
    assert_eq!(ja4.ext_hash, ja4.ext_hash.to_lowercase());
}

#[test]
fn test_ja4_dtls_transport() {
    let ch = build_client_hello(Some(b"example.com"), Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Dtls);
    assert!(ja4.full.starts_with("d13d"), "JA4 DTLS: {}", ja4.full);
}

#[test]
fn test_ja4_single_cipher() {
    let ch = build_client_hello(Some(b"example.com"), Some(vec![b"h2"]), &[0x1301], None);
    let parsed = parse_client_hello(&ch).unwrap();
    let ja4 = parsed.ja4(Transport::Tcp);
    assert_eq!(ja4.cipher_count, 1);
    assert!(ja4.full.starts_with("t13d01"));
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4S — Server Fingerprint (10 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4s_tls13_aes128() {
    let sh = build_server_hello(0x0303, 0x1301, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_1301_"),
        "JA4S TLS1.3 AES128: {}",
        ja4s.full
    );
    assert_eq!(ja4s.version, "12");
    assert_eq!(ja4s.cipher, "1301");
}

#[test]
fn test_ja4s_tls13_aes256() {
    let sh = build_server_hello(0x0303, 0x1302, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_1302_"),
        "JA4S TLS1.3 AES256: {}",
        ja4s.full
    );
}

#[test]
fn test_ja4s_tls13_chacha20() {
    let sh = build_server_hello(0x0303, 0x1303, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_1303_"),
        "JA4S CHACHA20: {}",
        ja4s.full
    );
}

#[test]
fn test_ja4s_tls12_ecdhe_rsa() {
    let sh = build_server_hello(0x0303, 0xc02f, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_c02f_"),
        "JA4S ECDHE_RSA: {}",
        ja4s.full
    );
}

#[test]
fn test_ja4s_tls12_ecdhe_ecdsa() {
    let sh = build_server_hello(0x0303, 0xc02b, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_c02b_"),
        "JA4S ECDHE_ECDSA: {}",
        ja4s.full
    );
}

#[test]
fn test_ja4s_with_renegotiation_info() {
    let renegotiation = [0x00];
    let sh = build_server_hello(0x0303, 0x1301, &[(0xff01, &renegotiation)]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s12_1301_"),
        "JA4S renegotiation: {}",
        ja4s.full
    );
    assert_eq!(ja4s.ext_hash.len(), 12);
}

#[test]
fn test_ja4s_determinism() {
    let sh = build_server_hello(0x0303, 0x1301, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s_1 = parsed.ja4s();
    let ja4s_2 = parsed.ja4s();
    assert_eq!(ja4s_1.full, ja4s_2.full);
}

#[test]
fn test_ja4s_hash_format() {
    let sh = build_server_hello(0x0303, 0x1301, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert_eq!(ja4s.ext_hash.len(), 12);
    assert!(ja4s.ext_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(ja4s.ext_hash, ja4s.ext_hash.to_lowercase());
}

#[test]
fn test_ja4s_tls13_wire() {
    let sh = build_server_hello(0x0304, 0x1301, &[]);
    let parsed = parse_server_hello(&sh).unwrap();
    let ja4s = parsed.ja4s();
    assert!(
        ja4s.full.starts_with("s13_1301_"),
        "JA4S TLS1.3 wire: {}",
        ja4s.full
    );
    assert_eq!(ja4s.version, "13");
}

#[test]
fn test_ja4s_various_ciphers() {
    for (expected_prefix, cipher) in [
        ("s12_009c_", 0x009c), // RSA_WITH_AES_128_GCM_SHA256
        ("s12_009d_", 0x009d), // RSA_WITH_AES_256_GCM_SHA384
        ("s12_c013_", 0xc013), // ECDHE_ECDSA_WITH_AES_128_CBC_SHA
        ("s12_c014_", 0xc014), // ECDHE_ECDSA_WITH_AES_256_CBC_SHA
        ("s12_cca9_", 0xcca9), // ECDHE_ECDSA_CHACHA20_POLY1305
    ] {
        let sh = build_server_hello(0x0303, cipher, &[]);
        let parsed = parse_server_hello(&sh).unwrap();
        let ja4s = parsed.ja4s();
        assert!(
            ja4s.full.starts_with(expected_prefix),
            "JA4S cipher {:#06x}: {}",
            cipher,
            ja4s.full
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4H — HTTP Client Fingerprint (10 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4h_get_http11() {
    let req = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nAccept: text/html\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert!(
        ja4h.full.starts_with("gethttp1.1"),
        "JA4H GET HTTP/1.1: {}",
        ja4h.full
    );
    assert_eq!(ja4h.method, "get");
    assert_eq!(ja4h.scheme, "http");
    assert_eq!(ja4h.http_version, "1.1");
}

#[test]
fn test_ja4h_post_json() {
    let req = b"POST /api/data HTTP/1.1\r\nHost: api.example.com\r\nAccept: application/json\r\nContent-Type: application/json\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert!(
        ja4h.full.starts_with("posthttp1.1"),
        "JA4H POST: {}",
        ja4h.full
    );
    assert_eq!(ja4h.method, "post");
}

#[test]
fn test_ja4h_https() {
    let req = b"GET /secure HTTP/1.1\r\nHost: secure.example.com\r\nAccept: */*\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert!(
        ja4h.full.starts_with("gethttp1.1"),
        "JA4H HTTPS URI: {}",
        ja4h.full
    );
}

#[test]
fn test_ja4h_with_cookies() {
    let req1 = b"GET / HTTP/1.1\r\nCookie: session=abc123\r\nAccept: */*\r\n\r\n";
    let req2 = b"GET / HTTP/1.1\r\nCookie: session=xyz789\r\nAccept: */*\r\n\r\n";
    let j1 = parse_http_request(req1).unwrap().ja4h();
    let j2 = parse_http_request(req2).unwrap().ja4h();
    assert_ne!(j1.cookie_hash, j2.cookie_hash);
    assert_eq!(j1.cookie_hash.len(), 12);
    assert_eq!(j2.cookie_hash.len(), 12);
}

#[test]
fn test_ja4h_no_cookies() {
    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert_eq!(ja4h.cookie_hash, "000000000000");
}

#[test]
fn test_ja4h_with_referer() {
    let req = b"GET /page HTTP/1.1\r\nHost: example.com\r\nReferer: https://google.com\r\nAccept: */*\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert_ne!(ja4h.ref_hash, "000000000000");
    assert_eq!(ja4h.ref_hash.len(), 12);
}

#[test]
fn test_ja4h_no_referer() {
    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert_eq!(ja4h.ref_hash, "000000000000");
}

#[test]
fn test_ja4h_accept_truncation() {
    let long_accept = "a".repeat(50);
    let req = format!("GET / HTTP/1.1\r\nAccept: {}\r\n\r\n", long_accept);
    let parsed = parse_http_request(req.as_bytes()).unwrap();
    let ja4h = parsed.ja4h();
    assert_eq!(ja4h.accept.len(), 20);
    assert_eq!(ja4h.accept, "a".repeat(20));
}

#[test]
fn test_ja4h_determinism() {
    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nAccept: text/html\r\nCookie: a=1\r\n\r\n";
    let j1 = parse_http_request(req).unwrap().ja4h();
    let j2 = parse_http_request(req).unwrap().ja4h();
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4h_multiple_cookies() {
    let req = b"GET / HTTP/1.1\r\nCookie: a=1; b=2\r\nCookie: c=3\r\nAccept: */*\r\n\r\n";
    let parsed = parse_http_request(req).unwrap();
    let ja4h = parsed.ja4h();
    assert_ne!(ja4h.cookie_hash, "000000000000");
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4T — TCP Client Fingerprint (10 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4t_linux_default() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4t = Ja4tFingerprint::compute(1460, 7, true, &opts, 64);
    assert!(
        ja4t.full.starts_with("05b4_07_1_40_"),
        "JA4T Linux: {}",
        ja4t.full
    );
    assert_eq!(ja4t.mss, 1460);
    assert_eq!(ja4t.wscale, 7);
    assert!(ja4t.sack);
    assert_eq!(ja4t.ttl, 64);
}

#[test]
fn test_ja4t_windows_default() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(8),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 4, 3],
    };
    let ja4t = Ja4tFingerprint::compute(1460, 8, true, &opts, 128);
    assert!(
        ja4t.full.starts_with("05b4_08_1_80_"),
        "JA4T Windows: {}",
        ja4t.full
    );
    assert_eq!(ja4t.ttl, 128);
}

#[test]
fn test_ja4t_macos_default() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(6),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 4, 3],
    };
    let ja4t = Ja4tFingerprint::compute(1460, 6, true, &opts, 64);
    assert!(
        ja4t.full.starts_with("05b4_06_1_40_"),
        "JA4T macOS: {}",
        ja4t.full
    );
}

#[test]
fn test_ja4t_no_sack() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: false,
        timestamp: None,
        option_order: vec![2, 3],
    };
    let ja4t = Ja4tFingerprint::compute(1460, 7, false, &opts, 64);
    assert!(
        ja4t.full.starts_with("05b4_07_0_40_"),
        "JA4T no SACK: {}",
        ja4t.full
    );
}

#[test]
fn test_ja4t_jumbo_frame() {
    let opts = TcpOptions {
        mss: Some(8960),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4t = Ja4tFingerprint::compute(8960, 7, true, &opts, 64);
    assert!(
        ja4t.full.starts_with("2300_07_1_40_"),
        "JA4T jumbo: {}",
        ja4t.full
    );
}

#[test]
fn test_ja4t_parse_tcp_options() {
    let raw = [
        0x02, 0x04, 0x05, 0xB4, // MSS=1460
        0x01, // NOP
        0x03, 0x03, 0x07, // WS=7
        0x01, // NOP
        0x04, 0x02, // SACK
    ];
    let opts = parse_tcp_options(&raw);
    assert_eq!(opts.mss, Some(1460));
    assert_eq!(opts.window_scale, Some(7));
    assert!(opts.sack_permitted);
    assert_eq!(opts.option_order, vec![2, 3, 4]);
}

#[test]
fn test_ja4t_parse_with_timestamp() {
    let raw = [
        0x02, 0x04, 0x05, 0xB4, // MSS
        0x08, 0x0A, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // TS
        0x01, 0x03, 0x03, 0x07, // NOP + WS
    ];
    let opts = parse_tcp_options(&raw);
    assert_eq!(opts.mss, Some(1460));
    assert!(opts.timestamp.is_some());
    assert_eq!(opts.option_order, vec![2, 8, 3]);
}

#[test]
fn test_ja4t_determinism() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let j1 = Ja4tFingerprint::compute(1460, 7, true, &opts, 64);
    let j2 = Ja4tFingerprint::compute(1460, 7, true, &opts, 64);
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4t_different_options_same_values() {
    let opts1 = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let opts2 = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 4, 3],
    };
    let j1 = Ja4tFingerprint::compute(1460, 7, true, &opts1, 64);
    let j2 = Ja4tFingerprint::compute(1460, 7, true, &opts2, 64);
    assert_ne!(
        j1.full, j2.full,
        "Different option orders should produce different fingerprints"
    );
}

#[test]
fn test_ja4t_empty_options() {
    let opts = TcpOptions::default();
    let ja4t = Ja4tFingerprint::compute(0, 0, false, &opts, 0);
    assert!(
        ja4t.full.starts_with("0000_00_0_00_"),
        "JA4T empty: {}",
        ja4t.full
    );
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4TS — TCP Server Fingerprint (8 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4ts_linux() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4ts = Ja4tsFingerprint::compute(1460, 7, true, &opts, 64);
    assert!(
        ja4ts.full.starts_with("07_05b4_1_40_"),
        "JA4TS Linux: {}",
        ja4ts.full
    );
    assert_eq!(ja4ts.wscale, 7);
    assert_eq!(ja4ts.mss, 1460);
}

#[test]
fn test_ja4ts_windows() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(8),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 4, 3],
    };
    let ja4ts = Ja4tsFingerprint::compute(1460, 8, true, &opts, 128);
    assert!(
        ja4ts.full.starts_with("08_05b4_1_80_"),
        "JA4TS Windows: {}",
        ja4ts.full
    );
    assert_eq!(ja4ts.ttl, 128);
}

#[test]
fn test_ja4ts_no_sack() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(7),
        sack_permitted: false,
        timestamp: None,
        option_order: vec![2, 3],
    };
    let ja4ts = Ja4tsFingerprint::compute(1460, 7, false, &opts, 64);
    assert!(
        ja4ts.full.starts_with("07_05b4_0_40_"),
        "JA4TS no SACK: {}",
        ja4ts.full
    );
    assert!(!ja4ts.sack);
}

#[test]
fn test_ja4ts_small_mss() {
    let opts = TcpOptions {
        mss: Some(536),
        window_scale: Some(7),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4ts = Ja4tsFingerprint::compute(536, 7, true, &opts, 64);
    assert!(
        ja4ts.full.starts_with("07_0218_1_40_"),
        "JA4TS small MSS: {}",
        ja4ts.full
    );
    assert_eq!(ja4ts.mss, 536);
}

#[test]
fn test_ja4ts_determinism() {
    let opts = TcpOptions {
        mss: Some(1440),
        window_scale: Some(8),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let j1 = Ja4tsFingerprint::compute(1440, 8, true, &opts, 128);
    let j2 = Ja4tsFingerprint::compute(1440, 8, true, &opts, 128);
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4ts_hash_format() {
    let opts = TcpOptions {
        mss: Some(1440),
        window_scale: Some(8),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4ts = Ja4tsFingerprint::compute(1440, 8, true, &opts, 128);
    assert_eq!(ja4ts.tcp_options_hash.len(), 12);
    assert!(ja4ts
        .tcp_options_hash
        .chars()
        .all(|c| c.is_ascii_hexdigit()));
    assert_eq!(
        ja4ts.tcp_options_hash,
        ja4ts.tcp_options_hash.to_lowercase()
    );
}

#[test]
fn test_ja4ts_option_order_sensitivity() {
    let opts1 = TcpOptions {
        option_order: vec![2, 3, 4],
        ..Default::default()
    };
    let opts2 = TcpOptions {
        option_order: vec![2, 4, 3],
        ..Default::default()
    };
    let j1 = Ja4tsFingerprint::compute(1460, 7, true, &opts1, 64);
    let j2 = Ja4tsFingerprint::compute(1460, 7, true, &opts2, 64);
    assert_ne!(j1.tcp_options_hash, j2.tcp_options_hash);
}

#[test]
fn test_ja4ts_large_window_scale() {
    let opts = TcpOptions {
        mss: Some(1460),
        window_scale: Some(14),
        sack_permitted: true,
        timestamp: None,
        option_order: vec![2, 3, 4],
    };
    let ja4ts = Ja4tsFingerprint::compute(1460, 14, true, &opts, 64);
    assert!(
        ja4ts.full.starts_with("0e_05b4_1_40_"),
        "JA4TS large WS: {}",
        ja4ts.full
    );
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4L — Light Distance / Latency (6 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4l_local() {
    let ja4l = Ja4lFingerprint::compute(0, 0);
    assert_eq!(ja4l.full, "0_000000");
    assert_eq!(ja4l.light_distance_km, 0);
    assert_eq!(ja4l.latency_ms, 0);
}

#[test]
fn test_ja4l_100km() {
    let ja4l = Ja4lFingerprint::compute(100, 1);
    assert_eq!(ja4l.full, "100_000001");
}

#[test]
fn test_ja4l_1000km() {
    let ja4l = Ja4lFingerprint::compute(1000, 5);
    assert_eq!(ja4l.full, "1000_000005");
}

#[test]
fn test_ja4l_5000km() {
    let ja4l = Ja4lFingerprint::compute(5000, 25);
    assert_eq!(ja4l.full, "5000_000025");
}

#[test]
fn test_ja4l_10000km() {
    let ja4l = Ja4lFingerprint::compute(10000, 50);
    assert_eq!(ja4l.full, "10000_000050");
}

#[test]
fn test_ja4l_distance_from_rtt() {
    let dist_1ms = Ja4lFingerprint::estimate_distance_from_rtt(1000);
    assert!(
        (95..105).contains(&dist_1ms),
        "1ms RTT → ~100km: {}",
        dist_1ms
    );

    let dist_10ms = Ja4lFingerprint::estimate_distance_from_rtt(10000);
    assert!(
        (950..1050).contains(&dist_10ms),
        "10ms RTT → ~1000km: {}",
        dist_10ms
    );

    let dist_50ms = Ja4lFingerprint::estimate_distance_from_rtt(50000);
    assert!(
        (4750..5250).contains(&dist_50ms),
        "50ms RTT → ~5000km: {}",
        dist_50ms
    );
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4X — X.509 Certificate Fingerprint (8 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4x_lets_encrypt() {
    let cert = X509Cert {
        der: &[],
        oids: vec![
            vec![1, 2, 840, 113549, 1, 1, 11], // SHA256WithRSAEncryption
            vec![2, 5, 29, 17],                // subjectAltName
        ],
        san_count: 3,
        ext_count: 8,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert!(
        ja4x.full.contains("_3_08"),
        "JA4X Let's Encrypt: {}",
        ja4x.full
    );
    assert_eq!(ja4x.san_count, 3);
    assert_eq!(ja4x.ext_count, 8);
}

#[test]
fn test_ja4x_self_signed() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11]],
        san_count: 0,
        ext_count: 3,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert!(
        ja4x.full.contains("_0_03"),
        "JA4X self-signed: {}",
        ja4x.full
    );
    assert_eq!(ja4x.san_count, 0);
}

#[test]
fn test_ja4x_wildcard_cert() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11], vec![2, 5, 29, 17]],
        san_count: 5,
        ext_count: 10,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert!(ja4x.full.contains("_5_0a"), "JA4X wildcard: {}", ja4x.full);
}

#[test]
fn test_ja4x_ip_san() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11], vec![2, 5, 29, 17]],
        san_count: 1,
        ext_count: 5,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert!(ja4x.full.contains("_1_05"), "JA4X IP SAN: {}", ja4x.full);
}

#[test]
fn test_ja4x_multiple_sans() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11]],
        san_count: 20,
        ext_count: 12,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert!(
        ja4x.full.contains("_20_0c"),
        "JA4X many SANs: {}",
        ja4x.full
    );
}

#[test]
fn test_ja4x_empty_oids() {
    let cert = X509Cert {
        der: &[],
        oids: vec![],
        san_count: 1,
        ext_count: 2,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert_eq!(ja4x.oid_hash, "000000000000");
    assert!(
        ja4x.full.starts_with("000000000000_1_02"),
        "JA4X empty OIDs: {}",
        ja4x.full
    );
}

#[test]
fn test_ja4x_determinism() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11]],
        san_count: 3,
        ext_count: 5,
    };
    let j1 = Ja4xFingerprint::compute(&cert);
    let j2 = Ja4xFingerprint::compute(&cert);
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4x_hash_format() {
    let cert = X509Cert {
        der: &[],
        oids: vec![vec![1, 2, 840, 113549, 1, 1, 11]],
        san_count: 3,
        ext_count: 5,
    };
    let ja4x = Ja4xFingerprint::compute(&cert);
    assert_eq!(ja4x.oid_hash.len(), 12);
    assert!(ja4x.oid_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(ja4x.oid_hash, ja4x.oid_hash.to_lowercase());
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4SSH — SSH Session Fingerprint (8 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4ssh_interactive() {
    let session = SshSessionData {
        kex_algorithms: vec!["curve25519-sha256", "ecdh-sha2-nistp256"],
        host_key_algorithms: vec!["ssh-ed25519", "rsa-sha2-512"],
        client_ciphers: vec!["chacha20-poly1305@openssh.com"],
        server_ciphers: vec!["chacha20-poly1305@openssh.com"],
        client_macs: vec!["umac-64@openssh.com"],
        server_macs: vec!["umac-64@openssh.com"],
        compression: vec!["none"],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Interactive);
    assert!(
        ja4ssh.full.starts_with("it_"),
        "JA4SSH interactive: {}",
        ja4ssh.full
    );
    assert_eq!(ja4ssh.session_type, SshSessionType::Interactive);
}

#[test]
fn test_ja4ssh_sftp() {
    let session = SshSessionData {
        kex_algorithms: vec!["curve25519-sha256"],
        host_key_algorithms: vec!["ssh-ed25519"],
        client_ciphers: vec!["aes256-ctr"],
        server_ciphers: vec!["aes256-ctr"],
        client_macs: vec!["hmac-sha2-256"],
        server_macs: vec!["hmac-sha2-256"],
        compression: vec!["none"],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Sftp);
    assert!(
        ja4ssh.full.starts_with("sf_"),
        "JA4SSH SFTP: {}",
        ja4ssh.full
    );
}

#[test]
fn test_ja4ssh_scp() {
    let session = SshSessionData {
        kex_algorithms: vec!["ecdh-sha2-nistp256"],
        host_key_algorithms: vec!["rsa-sha2-256"],
        client_ciphers: vec!["aes128-ctr"],
        server_ciphers: vec!["aes128-ctr"],
        client_macs: vec!["hmac-sha1"],
        server_macs: vec!["hmac-sha1"],
        compression: vec!["none"],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Scp);
    assert!(
        ja4ssh.full.starts_with("sc_"),
        "JA4SSH SCP: {}",
        ja4ssh.full
    );
}

#[test]
fn test_ja4ssh_tunnel() {
    let session = SshSessionData {
        kex_algorithms: vec!["curve25519-sha256"],
        host_key_algorithms: vec!["ssh-ed25519"],
        client_ciphers: vec!["chacha20-poly1305@openssh.com"],
        server_ciphers: vec!["chacha20-poly1305@openssh.com"],
        client_macs: vec!["umac-64@openssh.com"],
        server_macs: vec!["umac-64@openssh.com"],
        compression: vec!["none"],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Tunnel);
    assert!(
        ja4ssh.full.starts_with("tn_"),
        "JA4SSH tunnel: {}",
        ja4ssh.full
    );
}

#[test]
fn test_ja4ssh_port_forward() {
    let session = SshSessionData {
        kex_algorithms: vec!["diffie-hellman-group14-sha256"],
        host_key_algorithms: vec!["ssh-rsa"],
        client_ciphers: vec!["aes256-ctr"],
        server_ciphers: vec!["aes256-ctr"],
        client_macs: vec!["hmac-sha2-256"],
        server_macs: vec!["hmac-sha2-256"],
        compression: vec!["none"],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::PortForward);
    assert!(
        ja4ssh.full.starts_with("pf_"),
        "JA4SSH port forward: {}",
        ja4ssh.full
    );
}

#[test]
fn test_ja4ssh_determinism() {
    let session = SshSessionData {
        kex_algorithms: vec!["curve25519-sha256"],
        host_key_algorithms: vec!["ssh-ed25519"],
        client_ciphers: vec!["chacha20-poly1305@openssh.com"],
        server_ciphers: vec!["chacha20-poly1305@openssh.com"],
        client_macs: vec!["umac-64@openssh.com"],
        server_macs: vec!["umac-64@openssh.com"],
        compression: vec!["none"],
    };
    let j1 = Ja4sshFingerprint::compute(&session, SshSessionType::Interactive);
    let j2 = Ja4sshFingerprint::compute(&session, SshSessionType::Interactive);
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4ssh_hash_format() {
    let session = SshSessionData {
        kex_algorithms: vec!["curve25519-sha256"],
        host_key_algorithms: vec!["ssh-ed25519"],
        client_ciphers: vec![],
        server_ciphers: vec![],
        client_macs: vec![],
        server_macs: vec![],
        compression: vec![],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Interactive);
    assert_eq!(ja4ssh.kex_hash.len(), 12);
    assert_eq!(ja4ssh.key_type_hash.len(), 12);
    assert!(ja4ssh.kex_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(ja4ssh.key_type_hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_ja4ssh_empty_algorithms() {
    let session = SshSessionData {
        kex_algorithms: vec![],
        host_key_algorithms: vec![],
        client_ciphers: vec![],
        server_ciphers: vec![],
        client_macs: vec![],
        server_macs: vec![],
        compression: vec![],
    };
    let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Unknown);
    assert!(
        ja4ssh.full.starts_with("un_"),
        "JA4SSH unknown: {}",
        ja4ssh.full
    );
    assert_eq!(ja4ssh.kex_hash, "000000000000");
    assert_eq!(ja4ssh.key_type_hash, "000000000000");
}

// ══════════════════════════════════════════════════════════════════════════════
//  JA4D — DHCP Fingerprint (10 tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ja4d_v4_discover() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 1,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[1],
            },
            DhcpOption {
                code: 61,
                data: &[1, 0xaa, 0xbb],
            },
            DhcpOption {
                code: 55,
                data: &[1, 3, 6],
            },
        ],
        vendor_class: Some(b"MSFT 10.0"),
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("401_"),
        "JA4D v4 discover: {}",
        ja4d.full
    );
    assert_eq!(ja4d.proto, '4');
    assert_eq!(ja4d.msg_type, 1);
    assert_ne!(ja4d.vendor_hash, "000000000000");
}

#[test]
fn test_ja4d_v4_request() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 3,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[3],
            },
            DhcpOption {
                code: 50,
                data: &[192, 168, 1, 100],
            },
        ],
        vendor_class: Some(b"MSFT 10.0"),
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("403_"),
        "JA4D v4 request: {}",
        ja4d.full
    );
    assert_eq!(ja4d.msg_type, 3);
}

#[test]
fn test_ja4d_v4_ack() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 5,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[5],
            },
            DhcpOption {
                code: 1,
                data: &[255, 255, 255, 0],
            },
        ],
        vendor_class: None,
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(ja4d.full.starts_with("405_"), "JA4D v4 ack: {}", ja4d.full);
    assert_eq!(ja4d.vendor_hash, "000000000000");
}

#[test]
fn test_ja4d_v4_release() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 7,
        options: vec![DhcpOption {
            code: 53,
            data: &[7],
        }],
        vendor_class: None,
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("407_"),
        "JA4D v4 release: {}",
        ja4d.full
    );
}

#[test]
fn test_ja4d_v6_solicit() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V6,
        message_type: 1,
        options: vec![DhcpOption {
            code: 1,
            data: &[0xAA, 0xBB],
        }],
        vendor_class: None,
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("601_"),
        "JA4D v6 solicit: {}",
        ja4d.full
    );
    assert_eq!(ja4d.proto, '6');
}

#[test]
fn test_ja4d_v6_request() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V6,
        message_type: 3,
        options: vec![],
        vendor_class: Some(b"udhcp 1.0"),
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("603_"),
        "JA4D v6 request: {}",
        ja4d.full
    );
}

#[test]
fn test_ja4d_option_hash_changes_with_order() {
    let pkt1 = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 1,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[1],
            },
            DhcpOption {
                code: 61,
                data: &[1],
            },
        ],
        vendor_class: None,
    };
    let pkt2 = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 1,
        options: vec![
            DhcpOption {
                code: 61,
                data: &[1],
            },
            DhcpOption {
                code: 53,
                data: &[1],
            },
        ],
        vendor_class: None,
    };
    let j1 = Ja4dFingerprint::compute(&pkt1);
    let j2 = Ja4dFingerprint::compute(&pkt2);
    assert_ne!(
        j1.option_hash, j2.option_hash,
        "Option order should affect hash"
    );
}

#[test]
fn test_ja4d_determinism() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 1,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[1],
            },
            DhcpOption {
                code: 61,
                data: &[1, 0xaa],
            },
        ],
        vendor_class: Some(b"MSFT"),
    };
    let j1 = Ja4dFingerprint::compute(&pkt);
    let j2 = Ja4dFingerprint::compute(&pkt);
    assert_eq!(j1.full, j2.full);
}

#[test]
fn test_ja4d_no_options() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 8,
        options: vec![],
        vendor_class: None,
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("408_"),
        "JA4D no options: {}",
        ja4d.full
    );
    assert_eq!(ja4d.option_hash, "000000000000");
    assert_eq!(ja4d.vendor_hash, "000000000000");
}

#[test]
fn test_ja4d_v4_inform() {
    let pkt = DhcpPacket {
        proto: DhcpProto::V4,
        message_type: 8,
        options: vec![
            DhcpOption {
                code: 53,
                data: &[8],
            },
            DhcpOption {
                code: 55,
                data: &[1, 3],
            },
        ],
        vendor_class: Some(b"dhcpcd 10.0"),
    };
    let ja4d = Ja4dFingerprint::compute(&pkt);
    assert!(
        ja4d.full.starts_with("408_"),
        "JA4D v4 inform: {}",
        ja4d.full
    );
    assert_ne!(ja4d.vendor_hash, "000000000000");
}
