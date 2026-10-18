use aegis_lzr::LzrEngine;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SAMPLE_HTTP: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1234\r\nServer: nginx/1.18.0\r\n\r\n<!DOCTYPE html>\r\n<html>\r\n<head><title>Test</title></head>\r\n<body><h1>Hello</h1></body>\r\n</html>";

const SAMPLE_SSH: &[u8] = b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1\r\n";

const SAMPLE_SMTP: &[u8] = b"220 mail.example.com ESMTP Postfix (Ubuntu)\r\n";

const SAMPLE_MYSQL: &[u8] = &[
    0x00, 0x00, 0x00, 0x00, 0x0a, 0x38, 0x2e, 0x30, 0x2e, 0x33, 0x33, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const SAMPLE_OLLAMA: &[u8] = br#"{"version": "0.1.14", "commit": "abc123"}"#;

const SAMPLE_GRPC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

fn lzr_benchmark(c: &mut Criterion) {
    let engine = LzrEngine::new();

    c.bench_function("lzr_classify_http", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_HTTP)))
    });

    c.bench_function("lzr_classify_ssh", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_SSH)))
    });

    c.bench_function("lzr_classify_smtp", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_SMTP)))
    });

    c.bench_function("lzr_classify_mysql", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_MYSQL)))
    });

    c.bench_function("lzr_classify_ollama", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_OLLAMA)))
    });

    c.bench_function("lzr_classify_grpc", |b| {
        b.iter(|| engine.classify(black_box(SAMPLE_GRPC)))
    });
}

criterion_group!(benches, lzr_benchmark);
criterion_main!(benches);
