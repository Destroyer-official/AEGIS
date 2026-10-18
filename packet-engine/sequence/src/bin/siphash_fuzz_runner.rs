//! Standalone fuzz runner for SipHash-2-4
//! Runs edge case tests in a loop for extended periods (simulates cargo-fuzz on Windows)
//! Usage: cargo run --release --bin siphash_fuzz_runner -- [iterations] [duration_secs]

use aegis_sequence::{siphash24_bytes, SeqKey};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// Simple Xorshift64* PRNG
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn next_u32(&mut self) -> u32 {
        self.next() as u32
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000);
    let duration_secs = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(600); // default 10 minutes

    println!(
        "Starting SipHash-2-4 fuzz run: {} iterations or {} seconds",
        iterations, duration_secs
    );

    let start = Instant::now();
    let mut passed = 0u64;
    let mut failed = 0u64;

    // Test keys: zero, max, random, edge cases
    let test_keys = [
        [0u8; 16],    // zero key
        [0xFFu8; 16], // max key
        [0x42u8; 16], // pattern key
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ], // sequential
    ];

    let mut rng = XorShift64::new(0xDEADBEEFCAFEBABE);

    for i in 0..iterations {
        if start.elapsed() >= Duration::from_secs(duration_secs) {
            println!("Time limit reached ({}s)", duration_secs);
            break;
        }

        // Pick a test key
        let key_bytes = test_keys[i % test_keys.len()];
        let key = SeqKey::from_bytes(&key_bytes);

        // Generate random data length (0 to 1024 bytes)
        let data_len = (rng.next() % 1025) as usize;
        let mut data = vec![0u8; data_len];
        for b in &mut data {
            *b = rng.next() as u8;
        }

        // Test 1: Basic hash/encode
        let h1 = key.hash(&data);
        let s1 = key.encode(&data);
        let h1b = siphash24_bytes(&key_bytes, &data);
        if h1 != h1b || s1 != (h1 as u32) {
            failed += 1;
            eprintln!("FAIL iter {}: hash mismatch", i);
            continue;
        }

        // Test 2: Empty data
        let h_empty = key.hash(b"");
        if h_empty != siphash24_bytes(&key_bytes, b"") {
            failed += 1;
            eprintln!("FAIL iter {}: empty data mismatch", i);
            continue;
        }

        // Test 3: IPv4 encoding
        let ip = Ipv4Addr::from(rng.next_u32());
        let port = (rng.next() % 65536) as u16;
        let timestamp = rng.next_u32();
        let seq = key.encode_ipv4(ip, port, timestamp);

        if !key.validate_ipv4(ip, port, timestamp, seq) {
            failed += 1;
            eprintln!("FAIL iter {}: validate_ipv4 failed", i);
            continue;
        }

        // Test 4: validate with wrong timestamp
        if timestamp != 0 && key.validate_ipv4(ip, port, timestamp.wrapping_sub(1), seq) {
            failed += 1;
            eprintln!("FAIL iter {}: wrong timestamp passed", i);
            continue;
        }

        // Test 5: Full IPv4 encoding
        let src_ip = Ipv4Addr::from(rng.next_u32());
        let src_port = (rng.next() % 65536) as u16;
        let full_seq = key.encode_full_ipv4(ip, port, src_ip, src_port, timestamp);
        if !key.validate_full_ipv4(ip, port, src_ip, src_port, timestamp, full_seq) {
            failed += 1;
            eprintln!("FAIL iter {}: validate full failed", i);
            continue;
        }

        // Test 6: IPv6 encoding
        let ip6 = std::net::Ipv6Addr::from([
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
        ]);
        let seq6 = key.encode_ipv6(ip6, port, timestamp);
        if !key.validate_ipv6(ip6, port, timestamp, seq6) {
            failed += 1;
            eprintln!("FAIL iter {}: IPv6 validate failed", i);
            continue;
        }

        // Test 7: Window validation
        let _found = key.validate_ipv4_with_window(ip, port, seq, timestamp, 10);

        passed += 1;

        if i % 1_000_000 == 0 && i > 0 {
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "  {}M iterations: {:.1}M ops/sec, passed={}, failed={}",
                i / 1_000_000,
                passed as f64 / elapsed,
                passed,
                failed
            );
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "Fuzz run complete: {:.1}s, {} iterations, {:.1}M ops/sec",
        elapsed,
        passed,
        passed as f64 / elapsed
    );
    println!("Passed: {}, Failed: {}", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
