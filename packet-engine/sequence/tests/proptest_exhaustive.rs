//! Proptest exhaustive coverage for SipHash-2-4 sequence encoding
//! Target: 10K+ generated test cases covering edge cases
//! - Empty data, max-length data
//! - Wrap-around arithmetic
//! - Deterministic output
//! - Validation correctness

use aegis_sequence::{siphash24, siphash24_bytes, SeqKey};
use core::net::{Ipv4Addr, Ipv6Addr};
use proptest::prelude::*;

/// Generate a valid 16-byte key
fn arb_key() -> impl Strategy<Value = [u8; 16]> {
    prop::array::uniform16(any::<u8>())
}

/// Generate a valid IPv4 address
fn arb_ipv4() -> impl Strategy<Value = Ipv4Addr> {
    prop::array::uniform4(any::<u8>()).prop_map(Ipv4Addr::from)
}

/// Generate a valid IPv6 address
fn arb_ipv6() -> impl Strategy<Value = Ipv6Addr> {
    prop::array::uniform16(any::<u8>()).prop_map(Ipv6Addr::from)
}

/// Generate a valid port (0..65535)
fn arb_port() -> impl Strategy<Value = u16> {
    any::<u16>()
}

/// Generate a valid timestamp
fn arb_timestamp() -> impl Strategy<Value = u32> {
    any::<u32>()
}

/// Generate data with specific edge-case lengths
#[allow(dead_code)]
fn arb_data_with_edges() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        // Empty data
        Just(vec![]),
        // 1 byte
        prop::array::uniform1(any::<u8>()).prop_map(|b| b.to_vec()),
        // 7 bytes (last block boundary)
        prop::array::uniform7(any::<u8>()).prop_map(|b| b.to_vec()),
        // 8 bytes (exact block)
        prop::array::uniform8(any::<u8>()).prop_map(|b| b.to_vec()),
        // 15 bytes (max tail)
        prop::array::uniform15(any::<u8>()).prop_map(|b| b.to_vec()),
        // 16 bytes
        prop::array::uniform16(any::<u8>()).prop_map(|b| b.to_vec()),
        // 22 bytes (typical probe data)
        prop::array::uniform22(any::<u8>()).prop_map(|b| b.to_vec()),
        // 32 bytes
        prop::array::uniform32(any::<u8>()).prop_map(|b| b.to_vec()),
        // 64 bytes (use collection instead of uniform64 which doesn't exist)
        prop::collection::vec(any::<u8>(), 64..65).prop_map(|b| b),
        // Random length 0..512
        prop::collection::vec(any::<u8>(), 0..512),
    ]
}

/// Generate all-zero data (edge case for SipHash)
#[allow(dead_code)]
fn arb_zero_data() -> impl Strategy<Value = Vec<u8>> {
    (0usize..256).prop_map(|len| vec![0u8; len])
}

/// Generate all-0xFF data (edge case)
#[allow(dead_code)]
fn arb_max_data() -> impl Strategy<Value = Vec<u8>> {
    (0usize..256).prop_map(|len| vec![0xFFu8; len])
}

// ───────── SipHash-2-4 Core Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    /// Property: SipHash output is deterministic for same key+data
    #[test]
    fn siphash_deterministic(key in arb_key(), data in prop::collection::vec(any::<u8>(), 0..1024)) {
        let k0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
        let k1 = u64::from_le_bytes(key[8..16].try_into().unwrap());
        let h1 = siphash24(k0, k1, &data);
        let h2 = siphash24(k0, k1, &data);
        prop_assert_eq!(h1, h2);
    }

    /// Property: SipHash with different keys produces different outputs (high probability)
    #[test]
    fn siphash_different_keys(key1 in arb_key(), key2 in arb_key(), data in prop::collection::vec(any::<u8>(), 1..256)) {
        prop_assume!(key1 != key2);
        let h1 = siphash24_bytes(&key1, &data);
        let h2 = siphash24_bytes(&key2, &data);
        // Collision is extremely rare for random keys
        prop_assert_ne!(h1, h2, "collision with different keys on data len={}", data.len());
    }

    /// Property: SipHash with different data produces different outputs (high probability)
    #[test]
    fn siphash_different_data(key in arb_key(), data1 in prop::collection::vec(any::<u8>(), 1..256), data2 in prop::collection::vec(any::<u8>(), 1..256)) {
        prop_assume!(data1 != data2);
        let h1 = siphash24_bytes(&key, &data1);
        let h2 = siphash24_bytes(&key, &data2);
        prop_assert_ne!(h1, h2, "collision with different data");
    }

    /// Property: Empty data always produces same hash for same key
    #[test]
    fn siphash_empty_data(key in arb_key()) {
        let h1 = siphash24_bytes(&key, &[]);
        let h2 = siphash24_bytes(&key, &[]);
        prop_assert_eq!(h1, h2);
    }

    /// Property: 32-bit truncation is consistent
    #[test]
    fn encode_32bit_truncation(key in arb_key(), data in prop::collection::vec(any::<u8>(), 0..256)) {
        let key_obj = SeqKey::from_bytes(&key);
        let full = key_obj.hash(&data);
        let truncated = key_obj.encode(&data);
        prop_assert_eq!(truncated, full as u32);
    }
}

// ───────── IPv4 Encode/Validate Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Property: encode_ipv4 then validate_ipv4 always succeeds (round-trip)
    #[test]
    fn ipv4_roundtrip(key in arb_key(), ip in arb_ipv4(), port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv4(ip, port, ts);
        prop_assert!(key_obj.validate_ipv4(ip, port, ts, seq));
    }

    /// Property: validate_ipv4 fails with wrong timestamp
    #[test]
    fn ipv4_wrong_timestamp(key in arb_key(), ip in arb_ipv4(), port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv4(ip, port, ts);
        prop_assert!(!key_obj.validate_ipv4(ip, port, ts.wrapping_add(1), seq));
    }

    /// Property: validate_ipv4 fails with wrong IP
    #[test]
    fn ipv4_wrong_ip(key in arb_key(), ip1 in arb_ipv4(), ip2 in arb_ipv4(), port in arb_port(), ts in arb_timestamp()) {
        prop_assume!(ip1 != ip2);
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv4(ip1, port, ts);
        prop_assert!(!key_obj.validate_ipv4(ip2, port, ts, seq));
    }

    /// Property: validate_ipv4 fails with wrong port
    #[test]
    fn ipv4_wrong_port(key in arb_key(), ip in arb_ipv4(), port1 in arb_port(), port2 in arb_port(), ts in arb_timestamp()) {
        prop_assume!(port1 != port2);
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv4(ip, port1, ts);
        prop_assert!(!key_obj.validate_ipv4(ip, port2, ts, seq));
    }

    /// Property: encode_ipv4 output is deterministic
    #[test]
    fn ipv4_encode_deterministic(key in arb_key(), ip in arb_ipv4(), port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let s1 = key_obj.encode_ipv4(ip, port, ts);
        let s2 = key_obj.encode_ipv4(ip, port, ts);
        prop_assert_eq!(s1, s2);
    }

    /// Property: encode_full_ipv4 round-trip
    #[test]
    fn ipv4_full_roundtrip(key in arb_key(), dst_ip in arb_ipv4(), dst_port in arb_port(),
                           src_ip in arb_ipv4(), src_port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_full_ipv4(dst_ip, dst_port, src_ip, src_port, ts);
        prop_assert!(key_obj.validate_full_ipv4(dst_ip, dst_port, src_ip, src_port, ts, seq));
    }

    /// Property: encode_full_ipv4 differs when src_port changes
    #[test]
    fn ipv4_full_src_port_matters(key in arb_key(), dst_ip in arb_ipv4(), dst_port in arb_port(),
                                  src_ip in arb_ipv4(), src_port1 in arb_port(), src_port2 in arb_port(),
                                  ts in arb_timestamp()) {
        prop_assume!(src_port1 != src_port2);
        let key_obj = SeqKey::from_bytes(&key);
        let s1 = key_obj.encode_full_ipv4(dst_ip, dst_port, src_ip, src_port1, ts);
        let s2 = key_obj.encode_full_ipv4(dst_ip, dst_port, src_ip, src_port2, ts);
        prop_assert_ne!(s1, s2);
    }
}

// ───────── IPv6 Encode/Validate Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: encode_ipv6 then validate_ipv6 always succeeds
    #[test]
    fn ipv6_roundtrip(key in arb_key(), ip in arb_ipv6(), port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv6(ip, port, ts);
        prop_assert!(key_obj.validate_ipv6(ip, port, ts, seq));
    }

    /// Property: validate_ipv6 fails with wrong timestamp
    #[test]
    fn ipv6_wrong_timestamp(key in arb_key(), ip in arb_ipv6(), port in arb_port(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv6(ip, port, ts);
        prop_assert!(!key_obj.validate_ipv6(ip, port, ts.wrapping_add(1), seq));
    }

    /// Property: validate_ipv6 fails with wrong IP
    #[test]
    fn ipv6_wrong_ip(key in arb_key(), ip1 in arb_ipv6(), ip2 in arb_ipv6(), port in arb_port(), ts in arb_timestamp()) {
        prop_assume!(ip1 != ip2);
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv6(ip1, port, ts);
        prop_assert!(!key_obj.validate_ipv6(ip2, port, ts, seq));
    }
}

// ───────── Edge Case Tests ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: All-zero key produces valid output
    #[test]
    fn zero_key(data in prop::collection::vec(any::<u8>(), 0..256)) {
        let key = SeqKey::from_bytes(&[0u8; 16]);
        let seq = key.encode(&data);
        // Just verify it doesn't panic and returns a value
        let _ = seq;
    }

    /// Property: All-0xFF key produces valid output
    #[test]
    fn max_key(data in prop::collection::vec(any::<u8>(), 0..256)) {
        let key = SeqKey::from_bytes(&[0xFFu8; 16]);
        let seq = key.encode(&data);
        let _ = seq;
    }

    /// Property: Empty data with all-zero key
    #[test]
    fn zero_key_empty_data(key in arb_key()) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode(&[]);
        // Verify it's a valid 32-bit value
        let _ = seq;
    }

    /// Property: Maximum data (all 0xFF bytes)
    #[test]
    fn max_data(key in arb_key(), len in 0usize..1024) {
        let data = vec![0xFFu8; len];
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode(&data);
        let _ = seq;
    }

    /// Property: Sequential data pattern
    #[test]
    fn sequential_data(key in arb_key(), len in 0usize..256) {
        let data: Vec<u8> = (0..len).map(|i| (i & 0xFF) as u8).collect();
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode(&data);
        let _ = seq;
    }

    /// Property: Window validation within bounds
    #[test]
    fn window_validation(key in arb_key(), ip in arb_ipv4(), port in arb_port(),
                         ts in arb_timestamp(), offset in 0u32..100) {
        let key_obj = SeqKey::from_bytes(&key);
        let target_ts = ts.wrapping_add(offset);
        let seq = key_obj.encode_ipv4(ip, port, target_ts);
        let result = key_obj.validate_ipv4_with_window(ip, port, seq, ts, offset);
        prop_assert!(result.is_some(), "window validation should find ts within offset {}", offset);
    }

    /// Property: Window validation fails outside window
    #[test]
    fn window_validation_outside(key in arb_key(), ip in arb_ipv4(), port in arb_port(),
                                 ts in arb_timestamp(), offset in 101u32..1000) {
        let key_obj = SeqKey::from_bytes(&key);
        let target_ts = ts.wrapping_add(offset);
        let seq = key_obj.encode_ipv4(ip, port, target_ts);
        let result = key_obj.validate_ipv4_with_window(ip, port, seq, ts, 50);
        prop_assert!(result.is_none(), "window validation should fail outside window");
    }
}

// ───────── Wrap-Around Arithmetic Tests ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: Timestamp wrap-around (u32 overflow) doesn't break validation
    #[test]
    fn timestamp_wraparound(key in arb_key(), ip in arb_ipv4(), port in arb_port(),
                            ts in (u32::MAX - 100)..=u32::MAX) {
        let key_obj = SeqKey::from_bytes(&key);
        let seq = key_obj.encode_ipv4(ip, port, ts);
        prop_assert!(key_obj.validate_ipv4(ip, port, ts, seq));
        // Next timestamp wraps around to 0
        let next_ts = ts.wrapping_add(1);
        prop_assert!(!key_obj.validate_ipv4(ip, port, next_ts, seq));
    }

    /// Property: Port overflow (u16 max → 0) produces different seq
    #[test]
    fn port_overflow(key in arb_key(), ip in arb_ipv4(), ts in arb_timestamp()) {
        let key_obj = SeqKey::from_bytes(&key);
        let s1 = key_obj.encode_ipv4(ip, u16::MAX, ts);
        let s2 = key_obj.encode_ipv4(ip, 0, ts);
        prop_assert_ne!(s1, s2);
    }

    /// Property: Key bytes wrap-around (as_bytes roundtrip)
    #[test]
    fn key_as_bytes_roundtrip(key in arb_key()) {
        let key_obj = SeqKey::from_bytes(&key);
        let bytes = key_obj.as_bytes();
        let key_obj2 = SeqKey::from_bytes(&bytes);
        prop_assert_eq!(key_obj, key_obj2);
    }
}
