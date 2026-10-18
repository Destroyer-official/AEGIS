//! Fuzz target for CyclicPermutation.
//! Tests: prime values, generators, seeds, next/next_ipv4 calls.
//! Expected: no panics, no OOB, proper error handling.

#![no_main]
use libfuzzer_sys::fuzz_target;
use aegis_traversal::CyclicPermutation;

fuzz_target!(|data: &[u8]| {
    if data.len() < 24 {
        return;
    }

    // Extract parameters
    let p = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let g = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let seed = u64::from_le_bytes(data[16..24].try_into().unwrap());

    // Try to create permutation (may fail for invalid params — that's fine)
    if let Ok(mut perm) = CyclicPermutation::new(p, g, seed) {
        // Run bounded iterations
        for _ in 0..500 {
            if perm.is_complete() {
                break;
            }
            let _ = perm.next();
        }

        // Test reset
        perm.reset();
        assert_eq!(perm.steps(), 0);

        // Test next_ipv4 (may skip values > MAX_IPV4)
        if let Ok(mut perm2) = CyclicPermutation::new(p, g, seed) {
            for _ in 0..100 {
                if perm2.is_complete() {
                    break;
                }
                let _ = perm2.next_ipv4();
            }
        }
    }
});
