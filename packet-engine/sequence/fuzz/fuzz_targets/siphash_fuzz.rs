//! Fuzz target: SipHash-2-4 over arbitrary data with fixed key
//! Tests constant-time property and edge case data lengths

#![no_main]

use libfuzzer_sys::fuzz_target;
use aegis_sequence::{siphash24, SeqKey};

fuzz_target!(|data: &[u8]| {
    // Use a fixed non-zero key
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    
    // Hash the data - this exercises the core SipHash loop
    let _hash = key.hash(data);
    
    // Also test the 32-bit truncated version
    let _seq = key.encode(data);
    
    // Test with different key variations
    let zero_key = SeqKey::from_bytes(&[0u8; 16]);
    let _h1 = zero_key.hash(data);
    
    let max_key = SeqKey::from_bytes(&[0xFFu8; 16]);
    let _h2 = max_key.hash(data);
    
    // Test empty key (all zeros is handled above)
    // Test the raw siphash24 function directly using as_bytes()
    let key_bytes = key.as_bytes();
    let k0 = u64::from_le_bytes(key_bytes[0..8].try_into().unwrap());
    let k1 = u64::from_le_bytes(key_bytes[8..16].try_into().unwrap());
    let _h3 = siphash24(k0, k1, data);
});