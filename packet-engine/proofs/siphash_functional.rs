//! Kani proof: SipHash-2-4 determinism and functional correctness
//!
//! Properties verified:
//!
//! 1. **Determinism**: Same (key, data) always produces the same hash.
//! 2. **Bit-exact reference vectors**: Known test vectors are satisfied.
//! 3. **Key sensitivity (fixed key)**: Changing one byte of the key changes the hash.
//! 4. **Fixed output width**: Output is always a valid u64 (no UB in computation).

#![cfg_attr(kani, no_std)]
extern crate alloc;

use alloc::vec::Vec;

// ───────── SipHash-2-4 (replicated for standalone proof) ─────────

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

// ───────── Proof 1: Determinism ─────────

/// For any symbolic key (k0, k1) and symbolic data of bounded length,
/// calling siphash24 twice with the same inputs always yields the same output.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_determinism() {
    let k0: u64 = kani::any();
    let k1: u64 = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= 64);

    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    let h1 = siphash24(k0, k1, &data);
    let h2 = siphash24(k0, k1, &data);
    assert_eq!(h1, h2, "SipHash-2-4 must be deterministic");
}

// ───────── Proof 2: Reference vectors (key 00..0f) ─────────

/// Verify the reference test vectors from the SipHash paper / spec.
/// These are hard-coded to prove the implementation matches the spec.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_reference_vectors() {
    // Key = 0x00 0x01 ... 0x0f
    let key: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let k0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
    let k1 = u64::from_le_bytes(key[8..16].try_into().unwrap());

    // Empty message
    assert_eq!(siphash24(k0, k1, b""), 0x726fdb47dd0e0e31);
    // Message = [0x00]
    assert_eq!(siphash24(k0, k1, &[0x00]), 0x74f839c593dc67fd);
    // Message = [0x00, 0x01]
    assert_eq!(siphash24(k0, k1, &[0x00, 0x01]), 0x0d6c8009d9a94f5a);
    // Message = [0x00..0x07] (8 bytes)
    assert_eq!(
        siphash24(k0, k1, &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        0x93f5f5799a932462
    );
}

// ───────── Proof 3: Key sensitivity (fixed key comparison) ─────────

/// For a fixed known key, verify that flipping one byte of the key
/// produces a different hash. This tests that the key is actually mixed
/// into the hash state (not ignored).
///
/// Uses FIXED keys (not symbolic) to avoid the false-positive issue where
/// kani would need to verify "for ALL key pairs, hashes differ" — which
/// is mathematically false for any hash function (collisions exist).
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_key_sensitivity() {
    // Fixed key A
    let k0_a: u64 = 0x0706050403020100;
    let k1_a: u64 = 0x0f0e0d0c0b0a0908;

    // Key B differs in first byte of k0
    let k0_b: u64 = 0x07060504030201FF;
    let k1_b: u64 = 0x0f0e0d0c0b0a0908;

    // Symbolic data (non-empty)
    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= 32);
    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    let h_a = siphash24(k0_a, k1_a, &data);
    let h_b = siphash24(k0_b, k1_b, &data);

    // For these specific known-good keys, the hashes must differ.
    // This verifies the key material is mixed into the state.
    assert_ne!(h_a, h_b, "Key sensitivity violated for known keys");
}

// ───────── Proof 4: Output range ─────────

/// SipHash-2-4 output is always a valid u64 (wrapping arithmetic only).
/// No panics, no UB.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_output_range() {
    let k0: u64 = kani::any();
    let k1: u64 = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= 64);

    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    // Must not panic and must return a u64
    let _hash = siphash24(k0, k1, &data);
}
