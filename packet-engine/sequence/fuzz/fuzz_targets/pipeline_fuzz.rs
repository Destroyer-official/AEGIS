//! Fuzz target for the zero-copy pipeline module.
//! Tests: key bytes, IP addresses, ports, timestamps.
//! Expected: no panics, no OOB, no undefined behavior.

#![no_main]
use libfuzzer_sys::fuzz_target;
use aegis_sequence::SeqKey;
use aegis_sequence::pipeline::ProbePipeline;
use aegis_traversal::CyclicPermutation;

fuzz_target!(|data: &[u8]| {
    // Extract key (first 16 bytes)
    if data.len() < 16 {
        return;
    }
    let mut key_bytes = [0u8; 16];
    key_bytes.copy_from_slice(&data[..16]);
    let key = SeqKey::from_bytes(&key_bytes);

    // Extract remaining data for parameters
    let rest = &data[16..];

    // Extract port (2 bytes)
    let port = if rest.len() >= 2 {
        u16::from_le_bytes([rest[0], rest[1]])
    } else {
        443
    };

    // Extract timestamp (4 bytes)
    let ts = if rest.len() >= 6 {
        u32::from_le_bytes([rest[2], rest[3], rest[4], rest[5]])
    } else {
        1234567890
    };

    // Extract seed for permutation (first 8 bytes of remaining after port+ts)
    let seed = if rest.len() >= 14 {
        u64::from_le_bytes(rest[6..14].try_into().unwrap())
    } else {
        1
    };
    let seed = if seed == 0 { 1 } else { seed.min(4294967310) };

    // Create permutation with extracted seed
    if let Ok(perm) = CyclicPermutation::new_default(seed) {
        let mut pipeline = ProbePipeline::new(key, perm, ts);

        // Run pipeline for a bounded number of steps
        for _ in 0..100 {
            if let Some(target) = pipeline.next(port) {
                // Validate the target we just created
                let _ = pipeline.validate(&target, target.seq.wrapping_add(1));
                let _ = pipeline.validate(&target, target.seq);
                let _ = pipeline.validate_window(&target, target.seq.wrapping_add(1), 5);
            } else {
                break;
            }
        }
    }
});
