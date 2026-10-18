use aegis_lzr::LzrEngine;
use aegis_tls::{client_hello::Transport, ja4::Ja4Fingerprint, parse_client_hello};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Sample TLS Client Hello (Chrome-like) — 8 cipher suites, SNI, ALPN h2+http/1.1
const SAMPLE_CLIENT_HELLO: &[u8] = &[
    0x03, 0x01, // legacy_version
    // random (32 bytes)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x00, // session_id (empty)
    0x00, 0x10, // cipher_suites (16 bytes = 8 suites)
    0x13, 0x01, 0x13, 0x02, 0x13, 0x03, 0xc0, 0x2b, 0xc0, 0x2f, 0xc0, 0x2c, 0xc0, 0x30, 0xcc, 0xa9,
    0x01, 0x00, // compression_methods
    0x00, 0x39, // extensions length (57)
    // server_name: example.com
    0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x00, 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
    b'.', b'c', b'o', b'm', // signature_algorithms
    0x00, 0x0d, 0x00, 0x06, 0x00, 0x04, 0x04, 0x03, 0x05, 0x01, // ALPN: h2, http/1.1
    0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, b'h', b'2', 0x08, b'h', b't', b't', b'p', b'/', b'1',
    b'.', b'1', // supported_versions: TLS 1.3, TLS 1.2
    0x00, 0x2b, 0x00, 0x05, 0x04, 0x03, 0x04, 0x03, 0x03,
];

/// Sample HTTP response payload for LZR
const SAMPLE_HTTP: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1234\r\nServer: nginx/1.18.0\r\n\r\n<!DOCTYPE html>\r\n<html>\r\n<head><title>Test Page</title></head>\r\n<body>\r\n<h1>Hello World</h1>\r\n</body>\r\n</html>";

/// Sample SSH banner for LZR
const SAMPLE_SSH: &[u8] = b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1\r\n";

fn bench_tls_ja4_lzr(c: &mut Criterion) {
    // Pre-create engines
    let lzr_engine = LzrEngine::new();

    // Parse Client Hello once for reuse
    let client_hello = parse_client_hello(SAMPLE_CLIENT_HELLO).unwrap();

    let mut group = c.benchmark_group("tls_ja4_lzr_pipeline");

    // Bench 1: Parse Client Hello only (TLS record → ClientHello)
    group.bench_function("tls_parse_client_hello", |b| {
        b.iter(|| parse_client_hello(black_box(SAMPLE_CLIENT_HELLO)).unwrap())
    });

    // Bench 2: JA4 fingerprint computation only
    group.bench_function("ja4_compute", |b| {
        b.iter(|| Ja4Fingerprint::compute(black_box(&client_hello), Transport::Tcp))
    });

    // Bench 3: Full TLS → JA4 pipeline
    group.bench_function("tls_ja4_pipeline", |b| {
        b.iter(|| {
            let ch = parse_client_hello(black_box(SAMPLE_CLIENT_HELLO)).unwrap();
            Ja4Fingerprint::compute(black_box(&ch), Transport::Tcp)
        })
    });

    // Bench 4: LZR classify only (HTTP payload)
    group.bench_function("lzr_classify_http", |b| {
        b.iter(|| lzr_engine.classify(black_box(SAMPLE_HTTP)))
    });

    // Bench 5: LZR classify only (SSH payload)
    group.bench_function("lzr_classify_ssh", |b| {
        b.iter(|| lzr_engine.classify(black_box(SAMPLE_SSH)))
    });

    // Bench 6: Full pipeline TLS → JA4 → LZR (using JA4 string as LZR input)
    group.bench_function("tls_ja4_lzr_pipeline", |b| {
        b.iter(|| {
            // Step 1: Parse Client Hello
            let ch = parse_client_hello(black_box(SAMPLE_CLIENT_HELLO)).unwrap();
            // Step 2: Compute JA4
            let ja4 = Ja4Fingerprint::compute(black_box(&ch), Transport::Tcp);
            // Step 3: Feed JA4 string to LZR (simulating banner-based classification)
            // Note: LZR expects banner bytes; we use the JA4 string as a proxy
            let _ = lzr_engine.classify(black_box(ja4.full.as_bytes()));
        })
    });

    // Bench 7: Full realistic pipeline - TLS Client Hello + HTTP banner → LZR
    group.bench_function("tls_ja4_http_lzr_pipeline", |b| {
        b.iter(|| {
            // TLS handshake: parse CH + compute JA4
            let ch = parse_client_hello(black_box(SAMPLE_CLIENT_HELLO)).unwrap();
            let _ja4 = Ja4Fingerprint::compute(black_box(&ch), Transport::Tcp);
            // L7 banner classification
            lzr_engine.classify(black_box(SAMPLE_HTTP))
        })
    });

    // Bench 8: TLS + SSH banner → LZR
    group.bench_function("tls_ja4_ssh_lzr_pipeline", |b| {
        b.iter(|| {
            let ch = parse_client_hello(black_box(SAMPLE_CLIENT_HELLO)).unwrap();
            let _ja4 = Ja4Fingerprint::compute(black_box(&ch), Transport::Tcp);
            lzr_engine.classify(black_box(SAMPLE_SSH))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_tls_ja4_lzr);
criterion_main!(benches);
