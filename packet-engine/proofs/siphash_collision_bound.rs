//! Kani proof: SipHash-2-4 collision resistance structural checks
//!
//! These are structural checks on the implementation using FIXED keys
//! (not symbolic), verifying:
//!
//! - **No single-byte collisions** for a known key (all 256 1-byte messages)
//! - **Avalanche property** for a known key (bit flip changes output)
//!
//! For a correct SipHash-2-4 with a fixed key, 256 messages in 64-bit space
//! have collision probability ~256^2/2^65 ≈ 2^-48, making exhaustive check feasible.
//!
//! NOTE: We do NOT prove "for all keys, no collisions exist" — this is
//! mathematically false for any hash function (birthday paradox). Formal
//! verification of collision resistance requires information-theoretic
//! analysis, not bounded model checking.

#![cfg_attr(kani, no_std)]
extern crate alloc;

use alloc::vec::Vec;

// ───────── SipHash-2-4 (replicated) ─────────

#[inline(always)]
fn rotl(x: u64, b: u32) -> u64 {
    x.rotate_left(b)
}

#[inline(always)]
fn sip_round(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
    *v0 = v0.wrapping_add(*v1);
    *v1 = rotl(*v1, 13);
    *v1 ^= *v0;
    *v0 = rotl(*v0, 32);
    *v2 = v2.wrapping_add(*v3);
    *v3 = rotl(*v3, 16);
    *v3 ^= *v2;
    *v0 = v0.wrapping_add(*v3);
    *v3 = rotl(*v3, 21);
    *v3 ^= *v0;
    *v2 = v2.wrapping_add(*v1);
    *v1 = rotl(*v1, 17);
    *v1 ^= *v2;
    *v2 = rotl(*v2, 32);
}

#[inline]
fn siphash24(k0: u64, k1: u64, data: &[u8]) -> u64 {
    let mut v0 = k0 ^ 0x736f6d6570736575u64;
    let mut v1 = k1 ^ 0x646f72616e646f6du64;
    let mut v2 = k0 ^ 0x6c7967656e657261u64;
    let mut v3 = k1 ^ 0x7465646279746573u64;

    let len = data.len();
    let blocks = len / 8;

    for i in 0..blocks {
        let offset = i * 8;
        let m = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]);
        v3 ^= m;
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= m;
    }

    let tail_start = blocks * 8;
    let tail_len = len - tail_start;
    let mut b: u64 = (len as u64) << 56;
    for i in 0..tail_len {
        b |= (data[tail_start + i] as u64) << (8 * i);
    }

    v3 ^= b;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= b;

    v2 ^= 0xff;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);

    v0 ^ v1 ^ v2 ^ v3
}

// ───────── Proof 1: No trivial collisions for single-byte messages (fixed key) ─────────

/// For a FIXED known key, enumerate all 256 single-byte messages and verify
/// no two produce the same 64-bit hash. This exhaustively checks the
/// smallest non-trivial collision space for a specific key.
///
/// Uses a fixed key (not symbolic) because:
/// - For symbolic keys, the solver would need to verify "for ALL keys, no collisions",
///   which is false (birthday paradox guarantees collisions exist for some keys).
/// - For a fixed key, we verify that the specific implementation doesn't degenerate.
#[kani::proof]
#[kani::unwind(256)]
fn proof_no_trivial_single_byte_collisions() {
    // Fixed key from SipHash reference
    let k0: u64 = 0x0706050403020100;
    let k1: u64 = 0x0f0e0d0c0b0a0908;

    let mut seen = Vec::new();

    for byte in 0u16..=255 {
        let data = [byte as u8];
        let h = siphash24(k0, k1, &data);

        for &prev in &seen {
            assert_ne!(prev, h, "Single-byte collision found at byte={}", byte);
        }
        seen.push(h);
    }

    assert_eq!(seen.len(), 256, "Should have 256 distinct hashes");
}

// ───────── Proof 2: Avalanche property (fixed key) ─────────

/// For a FIXED key and symbolic message, flipping one bit of the input
/// changes at least 1 bit of the output. We verify with a fixed key
/// to avoid false failures (for some keys/messages, bit flip may change
/// fewer than 4 bits — this is statistically unlikely but possible).
#[kani::proof]
#[kani::unwind(32)]
fn proof_avalanche_property() {
    // Fixed key
    let k0: u64 = 0x0706050403020100;
    let k1: u64 = 0x0f0e0d0c0b0a0908;

    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= 16);
    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    let h_original = siphash24(k0, k1, &data);

    // Flip bit 0 of first byte
    data[0] ^= 1;
    let h_flipped = siphash24(k0, k1, &data);

    // At least 1 bit must change in the output
    assert_ne!(
        h_original, h_flipped,
        "Avalanche violation: flipping 1 input bit produced 0 output changes"
    );
}

// ───────── Proof 3: No zero-hash for non-empty data (fixed key) ─────────

/// For a fixed key, verify that non-empty messages don't hash to 0.
/// While 0 is a valid hash output, it's statistically unlikely and
/// its absence for short messages indicates good diffusion.
#[kani::proof]
#[kani::unwind(32)]
fn proof_no_zero_hash_short_messages() {
    let k0: u64 = 0x0706050403020100;
    let k1: u64 = 0x0f0e0d0c0b0a0908;

    // Check all 1-byte and 2-byte messages
    for byte in 0u16..=255 {
        let data = [byte as u8];
        let h = siphash24(k0, k1, &data);
        assert_ne!(h, 0, "1-byte message hashed to 0");
    }

    for b1 in 0u16..=255 {
        for b2 in 0u16..=255 {
            let data = [b1 as u8, b2 as u8];
            let h = siphash24(k0, k1, &data);
            assert_ne!(h, 0, "2-byte message hashed to 0");
        }
    }
}
