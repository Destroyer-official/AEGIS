//! Fuzz target: SipHash-2-4 IPv4 probe encoding
//! Tests encode_ipv4, encode_full_ipv4, validate paths with fuzzed inputs

#![no_main]

use libfuzzer_sys::fuzz_target;
use aegis_sequence::{SeqKey, siphash24_bytes};
use std::net::Ipv4Addr;

fuzz_target!(|data: &[u8]| {
    if data.len() < 20 {
        return; // Need key(16) + ip(4) + port(2) + ts(4) = 26 bytes minimum, but we can handle shorter
    }
    
    let key_bytes: [u8; 16] = data[0..16].try_into().unwrap();
    let key = SeqKey::from_bytes(&key_bytes);
    
    // Parse IPv4 (4 bytes)
    if data.len() >= 20 {
        let ip_bytes: [u8; 4] = data[16..20].try_into().unwrap();
        let ip = Ipv4Addr::from(ip_bytes);
        
        // Parse port (2 bytes, little-endian)
        let port = if data.len() >= 22 {
            u16::from_le_bytes([data[20], data[21]])
        } else {
            80
        };
        
        // Parse timestamp (4 bytes, little-endian)
        let timestamp = if data.len() >= 26 {
            u32::from_le_bytes([data[22], data[23], data[24], data[25]])
        } else {
            0
        };
        
        // Test encode_ipv4
        let seq = key.encode_ipv4(ip, port, timestamp);
        
        // Test validate_ipv4 (should pass for correct seq)
        assert!(key.validate_ipv4(ip, port, timestamp, seq));
        
        // Test validate_ipv4 with wrong timestamp (should fail)
        if timestamp != 0 {
            assert!(!key.validate_ipv4(ip, port, timestamp.wrapping_sub(1), seq));
        }
        
        // Test encode_full_ipv4 with src_ip and src_port
        let src_ip = Ipv4Addr::from([192, 168, 1, 1]);
        let src_port = 12345u16;
        let full_seq = key.encode_full_ipv4(ip, port, src_ip, src_port, timestamp);
        
        // Validate round-trip (must use validate_full_ipv4, not validate_ipv4)
        assert!(key.validate_full_ipv4(ip, port, src_ip, src_port, timestamp, full_seq));
        
        // Test window validation
        let _found = key.validate_ipv4_with_window(ip, port, seq, timestamp, 10);
        
        // Test with different timestamp within window
        if timestamp > 5 {
            let _found2 = key.validate_ipv4_with_window(ip, port, seq, timestamp - 3, 10);
        }
    }
    
    // Test IPv6 encoding if enough data
    if data.len() >= 38 {
        // IPv6 = 16 bytes IP + 2 port + 4 timestamp = 22 bytes after key
        let ip_bytes: [u8; 16] = data[16..32].try_into().unwrap();
        let ip = std::net::Ipv6Addr::from(ip_bytes);
        let port = u16::from_le_bytes([data[32], data[33]]);
        let timestamp = u32::from_le_bytes([data[34], data[35], data[36], data[37]]);
        
        let seq6 = key.encode_ipv6(ip, port, timestamp);
        assert!(key.validate_ipv6(ip, port, timestamp, seq6));
    }
    
    // Test raw siphash24 with structured data
    // Build a buffer matching the IPv4 layout
    if data.len() >= 26 {
        let mut buf = [0u8; 26];
        buf[0..16].copy_from_slice(&key_bytes);
        buf[16..20].copy_from_slice(&data[16..20]);
        if data.len() >= 22 { buf[20..22].copy_from_slice(&data[20..22]); }
        if data.len() >= 26 { buf[22..26].copy_from_slice(&data[22..26]); }
        let _raw = siphash24_bytes(&key_bytes, &buf[16..]);
    }
});