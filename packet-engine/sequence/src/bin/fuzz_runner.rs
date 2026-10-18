//! Standalone fuzz runner for Windows (no ASan required).
//! Runs all fuzz targets with random data for verification.
//! Usage: cargo run --release --bin fuzz_runner -- [iterations]

use aegis_sequence::pipeline::ProbePipeline;
use aegis_sequence::SeqKey;
use aegis_traversal::{ipv4_to_octets, CyclicPermutation, ShardSet, MAX_IPV4};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations: usize = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let time_limit_secs: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);

    println!(
        "AEGIS Fuzz Runner: {} iterations or {} seconds",
        iterations, time_limit_secs
    );
    let start = Instant::now();

    let mut passed = 0u64;
    let mut failed = 0u64;

    for i in 0..iterations {
        if start.elapsed().as_secs() >= time_limit_secs {
            println!("Time limit reached at iteration {}", i);
            break;
        }

        // Generate pseudo-random data from iteration counter
        let data = generate_data(i as u64);

        // Run all fuzz targets
        let results = [
            fuzz_siphash(&data),
            fuzz_pipeline(&data),
            fuzz_permutation(&data),
            fuzz_shard(&data),
            fuzz_ipv4(&data),
        ];

        if results.iter().all(|r| *r) {
            passed += 1;
        } else {
            failed += 1;
            eprintln!("FAIL at iteration {}: {:?}", i, results);
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = passed as f64 / elapsed.as_secs_f64();
    println!(
        "Fuzz run complete: {:.1}s, {} iterations, {:.1}M ops/sec, passed: {}, failed: {}",
        elapsed.as_secs_f64(),
        passed + failed,
        ops_per_sec / 1_000_000.0,
        passed,
        failed
    );

    if failed > 0 {
        std::process::exit(1);
    }
}

fn generate_data(seed: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(32);
    let mut x = seed;
    for _ in 0..32 {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((x >> 33) as u8);
    }
    data
}

fn fuzz_siphash(data: &[u8]) -> bool {
    if data.len() < 16 {
        return true;
    }
    let mut key_bytes = [0u8; 16];
    key_bytes.copy_from_slice(&data[..16]);
    let key = SeqKey::from_bytes(&key_bytes);

    let rest = &data[16..];
    let port = if rest.len() >= 2 {
        u16::from_le_bytes([rest[0], rest[1]])
    } else {
        443
    };
    let ts = if rest.len() >= 6 {
        u32::from_le_bytes([rest[2], rest[3], rest[4], rest[5]])
    } else {
        1234567890
    };

    let seq = key.encode_ipv4(std::net::Ipv4Addr::new(8, 8, 8, 8), port, ts);
    key.validate_ipv4(std::net::Ipv4Addr::new(8, 8, 8, 8), port, ts, seq)
}

fn fuzz_pipeline(data: &[u8]) -> bool {
    if data.len() < 24 {
        return true;
    }
    let mut key_bytes = [0u8; 16];
    key_bytes.copy_from_slice(&data[..16]);
    let key = SeqKey::from_bytes(&key_bytes);

    let port = u16::from_le_bytes([data[16], data[17]]);
    let ts = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
    let seed = u64::from_le_bytes(data[16..24].try_into().unwrap());
    let seed = if seed == 0 { 1 } else { seed.min(4294967310) };

    if let Ok(perm) = CyclicPermutation::new_default(seed) {
        let mut pipeline = ProbePipeline::new(key, perm, ts);
        for _ in 0..50 {
            if let Some(target) = pipeline.next(port) {
                let _ = pipeline.validate(&target, target.seq.wrapping_add(1));
            } else {
                break;
            }
        }
        true
    } else {
        true
    }
}

fn fuzz_permutation(data: &[u8]) -> bool {
    if data.len() < 24 {
        return true;
    }
    let p = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let g = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let seed = u64::from_le_bytes(data[16..24].try_into().unwrap());

    if let Ok(mut perm) = CyclicPermutation::new(p, g, seed) {
        for _ in 0..100 {
            if perm.is_complete() {
                break;
            }
            let _ = perm.next();
        }
        perm.reset();
        assert_eq!(perm.steps(), 0);
    }
    true
}

fn fuzz_shard(data: &[u8]) -> bool {
    if data.len() < 8 {
        return true;
    }
    let shard_count = (u64::from_le_bytes(data[0..8].try_into().unwrap()) % 16) as usize + 1;

    if let Ok(mut set) = ShardSet::new_default(shard_count) {
        for _ in 0..50 {
            let _ = set.send_all();
            let _ = set.recv_all();
        }
        for shard in set.shards_mut() {
            for _ in 0..20 {
                let _ = shard.next_target();
            }
        }
    }
    true
}

fn fuzz_ipv4(data: &[u8]) -> bool {
    if data.len() < 8 {
        return true;
    }
    let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let seed = if seed == 0 { 1 } else { seed.min(4294967310) };

    if let Ok(mut perm) = CyclicPermutation::new_default(seed) {
        for _ in 0..100 {
            if perm.is_complete() {
                break;
            }
            if let Some(ip) = perm.next_ipv4() {
                assert!(ip as u64 <= MAX_IPV4);
                assert_ne!(ip, 0);
                let octets = ipv4_to_octets(ip);
                let reconstructed = u32::from_be_bytes(octets);
                assert_eq!(ip, reconstructed);
            }
        }
    }
    true
}
