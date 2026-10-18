//! Fuzz target for IPv4 permutation traversal.
//! Tests: next_ipv4, collect_all_ipv4, ipv4_to_octets.
//! Expected: no panics, no OOB, values always <= MAX_IPV4.

#![no_main]
use libfuzzer_sys::fuzz_target;
use aegis_traversal::{CyclicPermutation, ipv4_to_octets, MAX_IPV4};

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let seed = if seed == 0 { 1 } else { seed.min(4294967310) };

    // Test next_ipv4
    if let Ok(mut perm) = CyclicPermutation::new_default(seed) {
        for _ in 0..200 {
            if perm.is_complete() {
                break;
            }
            if let Some(ip) = perm.next_ipv4() {
                assert!(ip as u64 <= MAX_IPV4, "ipv4 exceeds MAX_IPV4: {}", ip);
                assert_ne!(ip, 0, "ipv4 should never be 0");

                // Test ipv4_to_octets roundtrip
                let octets = ipv4_to_octets(ip);
                let reconstructed = u32::from_be_bytes(octets);
                assert_eq!(ip, reconstructed, "octets roundtrip failed");
            }
        }
    }

    // Test with small prime for collect_all_ipv4
    if data.len() >= 16 {
        let small_seed = (u64::from_le_bytes(data[8..16].try_into().unwrap()) % 16) + 1;
        if small_seed < 17 {
            if let Ok(perm) = CyclicPermutation::new(17, 3, small_seed) {
                let all = perm.collect_all_ipv4();
                for ip in &all {
                    assert!(*ip != 0);
                    assert!((*ip as u64) <= MAX_IPV4);
                }
            }
        }
    }
});
