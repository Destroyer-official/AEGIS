//! Kani proof harnesses for aegis-sequence SipHash-2-4
//!
//! Properties verified against the actual aegis_sequence crate:
//! 1. Determinism: same (key, data) → same hash
//! 2. Reference vectors: known test vectors are satisfied
//! 3. Key sensitivity: different keys produce different hashes (fixed keys)
//! 4. Encode/validate round-trip
//! 5. Output range: no panics, valid u32/u64 output
//!
//! Run: cargo kani --harness proofs/siphash_proofs.rs

use aegis_sequence::{SeqKey, siphash24, siphash24_bytes};
use core::net::Ipv4Addr;

// ───────── Proof 1: Determinism ─────────

/// For any symbolic key and data (bounded length), calling hash twice
/// with the same inputs always yields the same output.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_determinism() {
    let key_bytes: [u8; 16] = kani::any();
    let key = SeqKey::from_bytes(&key_bytes);

    let len: usize = kani::any();
    kani::assume(len <= 64);
    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    let h1 = key.hash(&data);
    let h2 = key.hash(&data);
    assert_eq!(h1, h2, "SipHash must be deterministic");
}

// ───────── Proof 2: Reference vectors ─────────

/// Verify known test vectors from SipHash spec (key 00..0f).
/// These are hard-coded constants — if they fail, the implementation is wrong.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_reference_vectors() {
    let key = SeqKey::from_bytes(&[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ]);

    // Empty message
    assert_eq!(key.hash(b""), 0x726fdb47dd0e0e31);
    // Message = [0x00]
    assert_eq!(key.hash(&[0x00]), 0x74f839c593dc67fd);
    // Message = [0x00, 0x01]
    assert_eq!(key.hash(&[0x00, 0x01]), 0x0d6c8009d9a94f5a);
    // Message = [0x00..0x07] (8 bytes)
    assert_eq!(
        key.hash(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        0x93f5f5799a932462
    );
}

// ───────── Proof 3: Key sensitivity (fixed keys) ─────────

/// For a fixed known key, verify that changing the key produces a different hash.
/// Uses FIXED keys (not symbolic) because "for ALL key pairs, no collisions"
/// is mathematically false for hash functions.
#[kani::proof]
#[kani::unwind(32)]
fn proof_siphash_key_sensitivity() {
    let key_a = SeqKey::from_bytes(&[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ]);
    let key_b = SeqKey::from_bytes(&[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0xff, // last byte changed
    ]);

    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= 32);
    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    let h_a = key_a.hash(&data);
    let h_b = key_b.hash(&data);

    assert_ne!(h_a, h_b, "Key sensitivity violated for known keys");
}

// ───────── Proof 4: Encode/validate round-trip ─────────

/// For any symbolic key, IP, port, and timestamp, encode then validate must succeed.
#[kani::proof]
#[kani::unwind(32)]
fn proof_encode_validate_roundtrip() {
    let key_bytes: [u8; 16] = kani::any();
    let key = SeqKey::from_bytes(&key_bytes);

    let ip_bytes: [u8; 4] = kani::any();
    let ip = Ipv4Addr::from(ip_bytes);
    let port: u16 = kani::any();
    let ts: u32 = kani::any();

    let seq = key.encode_ipv4(ip, port, ts);
    assert!(key.validate_ipv4(ip, port, ts, seq), "encode/validate round-trip failed");
}

// ───────── Proof 5: Output range ─────────

/// encode() always returns a valid u32 without panicking.
#[kani::proof]
#[kani::unwind(32)]
fn proof_encode_output_range() {
    let key_bytes: [u8; 16] = kani::any();
    let key = SeqKey::from_bytes(&key_bytes);

    let len: usize = kani::any();
    kani::assume(len <= 64);
    let mut data = Vec::new();
    for _ in 0..len {
        data.push(kani::any::<u8>());
    }

    // Must not panic and must return a u32
    let _seq = key.encode(&data);
}

// ───────── Proof 6: validate_ipv4 rejects wrong inputs ─────────

/// For a fixed encode, validate_ipv4 must reject wrong timestamp.
#[kani::proof]
#[kani::unwind(32)]
fn proof_validate_rejects_wrong_timestamp() {
    let key_bytes: [u8; 16] = kani::any();
    let key = SeqKey::from_bytes(&key_bytes);

    let ip_bytes: [u8; 4] = kani::any();
    let ip = Ipv4Addr::from(ip_bytes);
    let port: u16 = kani::any();
    let ts: u32 = kani::any();

    let seq = key.encode_ipv4(ip, port, ts);

    // Wrong timestamp must not validate
    kani::assume(ts != u32::MAX);
    assert!(
        !key.validate_ipv4(ip, port, ts.wrapping_add(1), seq),
        "validate_ipv4 accepted wrong timestamp"
    );
}

// ───────── Proof 7: as_bytes roundtrip ─────────

/// SeqKey::from_bytes then as_bytes must preserve the key.
#[kani::proof]
fn proof_key_bytes_roundtrip() {
    let key_bytes: [u8; 16] = kani::any();
    let key = SeqKey::from_bytes(&key_bytes);
    let recovered = key.as_bytes();
    assert_eq!(key_bytes, recovered, "key bytes roundtrip failed");
}
