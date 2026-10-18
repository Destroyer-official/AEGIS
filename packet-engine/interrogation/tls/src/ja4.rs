//! JA4 TLS client fingerprint computation (§6.1).
//!
//! Format: `{proto}{ver}{sni}{ciphers}{exts}{alpn}_{cipher_hash}_{ext_hash}`
//!
//! - `proto`: 't' (TCP), 'q' (QUIC), 'd' (DTLS)
//! - `ver`: 2-char TLS version (e.g., "13")
//! - `sni`: 'd' (domain) or 'i' (IP)
//! - `ciphers`: 2-char count of cipher suites (excluding GREASE)
//! - `exts`: 2-char count of extensions (excluding GREASE)
//! - `alpn`: First and last char of first ALPN protocol
//! - `cipher_hash`: Truncated SHA-256 of sorted cipher hex codes (12 chars)
//! - `ext_hash`: Truncated SHA-256 of sorted extension hex codes + sig algos (12 chars)

use crate::client_hello::{ClientHello, Snitype, Transport};
use crate::grease::is_grease;
use sha2::{Digest, Sha256};

/// JA4 TLS client fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4Fingerprint {
    /// Full JA4 string (e.g., "t13d1516h2_8daaf6152771_e5627efa2ab1")
    pub full: String,
    /// Protocol transport character
    pub proto: char,
    /// TLS version (2-char)
    pub version: String,
    /// SNI type character
    pub sni: char,
    /// Number of cipher suites (excluding GREASE)
    pub cipher_count: u32,
    /// Number of extensions (excluding GREASE and SNI)
    pub ext_count: u32,
    /// First and last character of first ALPN value
    pub alpn: String,
    /// Truncated SHA-256 of sorted cipher suites
    pub cipher_hash: String,
    /// Truncated SHA-256 of sorted extensions + sig algos
    pub ext_hash: String,
}

impl Ja4Fingerprint {
    /// Compute JA4 fingerprint from a parsed Client Hello.
    pub fn compute(ch: &ClientHello<'_>, transport: Transport) -> Self {
        // 1. Protocol
        let proto = transport.ja4_char();

        // 2. Version — use supported_versions extension if present (TLS 1.3),
        //    otherwise fall back to legacy_version
        let version = if let Some(sv) = ch.supported_versions {
            // supported_versions: 1-byte list length + 2-byte versions
            if sv.len() >= 3 {
                let first_ver = u16::from_be_bytes([sv[1], sv[2]]);
                // Map wire version to human-readable: 0x0304 → "13", 0x0303 → "12"
                match first_ver {
                    0x0304 => "13".to_string(),
                    0x0303 => "12".to_string(),
                    0x0302 => "11".to_string(),
                    0x0301 => "10".to_string(),
                    _ => format!("{:02x}", first_ver & 0xff),
                }
            } else {
                extract_version_from_legacy(ch.legacy_version)
            }
        } else {
            extract_version_from_legacy(ch.legacy_version)
        };

        // 3. SNI
        let sni = if let Some(sni_val) = ch.sni {
            // Check if SNI looks like an IP address vs domain
            // Simple heuristic: if it parses as IP, it's 'i', otherwise 'd'
            if is_ip_address(sni_val) {
                Snitype::Ip.ja4_char()
            } else {
                Snitype::Domain.ja4_char()
            }
        } else {
            Snitype::Ip.ja4_char()
        };

        // 4. Cipher count (excluding GREASE)
        let non_grease_ciphers: Vec<u16> = ch
            .cipher_suites
            .iter()
            .copied()
            .filter(|&c| !is_grease(c))
            .collect();
        let cipher_count = non_grease_ciphers.len();
        let cipher_count_str = format!("{:02}", cipher_count);

        // 5. Extension count (excluding GREASE only, per FoxIO JA4 spec: "Include SNI and ALPN")
        let non_grease_exts: Vec<u16> = ch
            .extensions
            .iter()
            .map(|e| e.extension_type)
            .filter(|&t| !is_grease(t))
            .collect();
        let ext_count = non_grease_exts.len();
        let ext_count_str = format!("{:02}", ext_count);

        // 6. ALPN — first and last character of first ALPN protocol value
        let alpn_str = if let Some(alpn_data) = ch.alpn {
            // ALPN data: 2-byte list length, then per-protocol: 1-byte len + value
            if alpn_data.len() >= 4 {
                let first_proto_len = alpn_data[2] as usize;
                if alpn_data.len() >= 3 + first_proto_len && first_proto_len > 0 {
                    let first_proto = &alpn_data[3..3 + first_proto_len];
                    let first_char = first_proto[0] as char;
                    let last_char = first_proto[first_proto.len() - 1] as char;
                    format!("{}{}", first_char, last_char)
                } else {
                    "00".to_string()
                }
            } else {
                "00".to_string()
            }
        } else {
            "00".to_string()
        };

        // 7. Cipher hash — SHA-256 of sorted cipher suite hex codes, truncated to 12 chars
        let cipher_hash = compute_cipher_hash(&non_grease_ciphers);

        // 8. Ext hash — SHA-256 of sorted extension hex codes (excluding SNI 0x0000 and ALPN 0x0010)
        //    + signature algorithms in original order, truncated to 12 chars
        let ext_hash = compute_ext_hash(ch);

        let full = format!(
            "{}{}{}{}{}{}_{}_{}",
            proto, version, sni, cipher_count_str, ext_count_str, alpn_str, cipher_hash, ext_hash
        );

        Ja4Fingerprint {
            full,
            proto,
            version,
            sni,
            cipher_count: cipher_count as u32,
            ext_count: ext_count as u32,
            alpn: alpn_str,
            cipher_hash,
            ext_hash,
        }
    }
}

/// Compute the cipher suite hash: SHA-256 of sorted hex codes, truncated to 12 chars.
fn compute_cipher_hash(ciphers: &[u16]) -> String {
    let mut sorted: Vec<String> = ciphers.iter().map(|c| format!("{:04x}", c)).collect();
    sorted.sort();
    let combined = sorted.join(",");
    let hash = Sha256::digest(combined.as_bytes());
    hex::encode(&hash[..6]) // 12 hex chars, lowercase per FoxIO spec
}

/// Compute the extension hash:
/// 1. Collect non-GREASE extension types (excluding SNI 0x0000 and ALPN 0x0010), sort hex
/// 2. Append signature algorithms (0x000d) data in original order if present
/// 3. SHA-256, truncate to 12 chars
fn compute_ext_hash(ch: &ClientHello<'_>) -> String {
    // Collect extension types (excluding SNI 0x0000, ALPN 0x0010, Padding 0x0015, GREASE)
    let ext_types: Vec<String> = ch
        .extensions
        .iter()
        .map(|e| e.extension_type)
        .filter(|&t| !is_grease(t) && t != 0x0000 && t != 0x0010 && t != 0x0015)
        .map(|t| format!("{:04x}", t))
        .collect();

    let mut combined = ext_types.join(",");

    // Append signature algorithms in original order (as per JA4 spec)
    if let Some(sig_alg) = ch.signature_algorithms {
        // sig_alg is the data portion of extension 0x000d: 2-byte list length + values
        if !combined.is_empty() {
            combined.push('|');
        }
        // Encode the raw sig alg bytes as hex
        for chunk in sig_alg.chunks(2) {
            if chunk.len() == 2 {
                combined.push_str(&format!("{:02x}{:02x}", chunk[0], chunk[1]));
            }
        }
    }

    let hash = Sha256::digest(combined.as_bytes());
    hex::encode(&hash[..6]) // 12 hex chars, lowercase per FoxIO spec
}

/// Extract 2-char version from legacy_version field.
fn extract_version_from_legacy(legacy: u16) -> String {
    match legacy {
        0x0301 => "10".to_string(),
        0x0302 => "11".to_string(),
        0x0303 => "12".to_string(),
        0x0304 => "13".to_string(),
        _ => format!("{:02x}", legacy & 0xff),
    }
}

/// Check if a byte slice looks like an IP address (IPv4 or IPv6 string).
fn is_ip_address(val: &[u8]) -> bool {
    let s = match core::str::from_utf8(val) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Quick check: contains only digits, dots, colons, and hex chars
    // IPv4: "192.168.1.1" or IPv6: "2001:db8::1"
    s.parse::<std::net::Ipv4Addr>().is_ok() || s.parse::<std::net::Ipv6Addr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_hello::parse_client_hello;

    /// TLS 1.3 Client Hello with 8 cipher suites, SNI "example.com", ALPN h2+http/1.1
    const TEST_CLIENT_HELLO: &[u8] = &[
        0x03, 0x01, // legacy_version
        // random
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
        0x17, 0x18, // session_id (empty)
        0x00, // cipher_suites (16 bytes = 8 suites)
        0x00, 0x10, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03, 0xc0, 0x2b, 0xc0, 0x2f, 0xc0, 0x2c, 0xc0,
        0x30, 0xcc, 0xa9, // compression_methods
        0x01, 0x00, // extensions total length
        0x00, 0x39, // 57 bytes
        // server_name
        0x00, 0x00, 0x00, 0x10, // type, data length (16)
        0x00, 0x0e, // list length (14)
        0x00, 0x00, 0x0b, // name type, name length (11)
        b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm',
        // signature_algorithms
        0x00, 0x0d, 0x00, 0x06, 0x00, 0x04, 0x04, 0x03, 0x05, 0x01, // ALPN
        0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, b'h', b'2', 0x08, b'h', b't', b't', b'p', b'/',
        b'1', b'.', b'1', // supported_versions
        0x00, 0x2b, 0x00, 0x05, 0x04, 0x03, 0x04, // TLS 1.3
        0x03, 0x03, // TLS 1.2
    ];

    #[test]
    fn test_ja4_format() {
        let ch = parse_client_hello(TEST_CLIENT_HELLO).unwrap();
        let ja4 = ch.ja4(Transport::Tcp);
        // Should start with "t13d" (TCP, TLS 1.3, domain SNI)
        assert!(
            ja4.full.starts_with("t13d"),
            "JA4 should start with t13d, got: {}",
            ja4.full
        );
        // Should have 8 cipher suites
        assert_eq!(ja4.cipher_count, 8);
        // Should contain underscore separators
        assert_eq!(
            ja4.full.matches('_').count(),
            2,
            "JA4 should have 2 underscores"
        );
        // 8 cipher suites → "08", 4 ext types (SNI + sig_algos + ALPN + supported_versions) → "04"
        assert!(
            ja4.full.starts_with("t13d0804h2"),
            "JA4 format: {}",
            ja4.full
        );
    }

    #[test]
    fn test_ja4_no_sni() {
        // Client Hello without SNI
        let ch = parse_client_hello(TEST_CLIENT_HELLO).unwrap();
        let mut ch_no_sni = ch.clone();
        ch_no_sni.sni = None;
        let ja4 = Ja4Fingerprint::compute(&ch_no_sni, Transport::Tcp);
        assert!(
            ja4.full.starts_with("t13i"),
            "JA4 without SNI should have 'i', got: {}",
            ja4.full
        );
    }

    #[test]
    fn test_ja4_quic_transport() {
        let ch = parse_client_hello(TEST_CLIENT_HELLO).unwrap();
        let ja4 = ch.ja4(Transport::Quic);
        assert!(
            ja4.full.starts_with("q13d"),
            "JA4 QUIC should start with q13d, got: {}",
            ja4.full
        );
    }

    #[test]
    fn test_ja4_alpn_extraction() {
        let ch = parse_client_hello(TEST_CLIENT_HELLO).unwrap();
        let ja4 = ch.ja4(Transport::Tcp);
        // First ALPN is "h2", first='h', last='2'
        assert_eq!(ja4.alpn, "h2");
    }
}
