//! Fuzz target: SipHash-2-4 with fuzzed keys
//! Covers edge cases: empty key, max key, zero key, random keys

#![no_main]

use libfuzzer_sys::fuzz_target;
use aegis_sequence::{siphash24, siphash24_bytes, SeqKey};

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return; // Need at least 16 bytes for key
    }
    
    // Split into key (16 bytes) and message (rest)
    let key_bytes: [u8; 16] = data[0..16].try_into().unwrap();
    let msg = &data[16..];
    
    // Test via SeqKey
    let key = SeqKey::from_bytes(&key_bytes);
    let _h1 = key.hash(msg);
    let _s1 = key.encode(msg);
    
    // Test via raw functions
    let _h2 = siphash24_bytes(&key_bytes, msg);
    let _s2 = siphash24_u32(&key_bytes, msg);
    
    // Also test with key as message
    let _h3 = key.hash(&key_bytes);
    
    // Test with message as key (if long enough)
    if msg.len() >= 16 {
        let msg_key: [u8; 16] = msg[0..16].try_into().unwrap();
        let _h4 = siphash24_bytes(&msg_key, &key_bytes);
    }
});

// Helper function for 32-bit truncated hash
fn siphash24_u32(key: &[u8; 16], data: &[u8]) -> u32 {
    let k0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
    let k1 = u64::from_le_bytes(key[8..16].try_into().unwrap());
    siphash24(k0, k1, data) as u32
}