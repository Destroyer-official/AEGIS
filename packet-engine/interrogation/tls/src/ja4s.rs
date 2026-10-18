//! JA4S — TLS server fingerprint from ServerHello messages (§6.1).
//!
//! JA4S format: `s{ver}{cipher}_{extensions}`
//!
//! - `ver`: 2-char TLS version from the Server Hello
//! - `cipher`: Selected cipher suite hex code
//! - `extensions`: Hashed extension types

use crate::grease::is_grease;
use crate::server_hello::ServerHello;
use sha2::{Digest, Sha256};

/// JA4S server fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4sFingerprint {
    pub full: String,
    pub version: String,
    pub cipher: String,
    pub ext_hash: String,
}

impl Ja4sFingerprint {
    /// Compute JA4S fingerprint from a parsed Server Hello.
    pub fn compute(sh: &ServerHello<'_>) -> Self {
        // Version: prefer supported_versions extension (0x002b) for TLS 1.3 detection.
        // Real TLS 1.3 ServerHello always has legacy_version = 0x0303 on the wire;
        // the actual negotiated version is in the supported_versions extension.
        let version = if let Some(sv) = sh.supported_version {
            match sv {
                0x0304 => "13".to_string(),
                0x0303 => "12".to_string(),
                0x0302 => "11".to_string(),
                0x0301 => "10".to_string(),
                _ => format!("{:02x}", sv & 0xff),
            }
        } else {
            match sh.legacy_version {
                0x0304 => "13".to_string(),
                0x0303 => "12".to_string(),
                0x0302 => "11".to_string(),
                0x0301 => "10".to_string(),
                _ => format!("{:02x}", sh.legacy_version & 0xff),
            }
        };

        // Cipher suite
        let cipher = format!("{:04x}", sh.cipher_suite);

        // Extension hash: SHA-256 of sorted non-GREASE extension types
        let ext_types: Vec<String> = sh
            .extensions
            .iter()
            .map(|e| e.extension_type)
            .filter(|&t| !is_grease(t))
            .map(|t| format!("{:04x}", t))
            .collect();

        let mut sorted = ext_types;
        sorted.sort();
        let combined = sorted.join(",");
        let hash = Sha256::digest(combined.as_bytes());
        let ext_hash = hex::encode(&hash[..6]); // lowercase per FoxIO spec

        let full = format!("s{}_{}_{}", version, cipher, ext_hash);

        Ja4sFingerprint {
            full,
            version,
            cipher,
            ext_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server_hello::parse_server_hello;

    #[test]
    fn test_ja4s_format() {
        let sh = parse_server_hello(&[
            0x03, 0x03, // legacy_version = TLS 1.2
            // random (32 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, // session_id_echo (empty)
            0x00, // cipher_suite: TLS_RSA_WITH_AES_128_GCM_SHA256
            0x00, 0x9c, // compression_method: null
            0x00, // extensions length: 0
            0x00, 0x00,
        ])
        .unwrap();

        let ja4s = sh.ja4s();
        assert!(
            ja4s.full.starts_with("s12_009c_"),
            "JA4S should start with s12_009c_, got: {}",
            ja4s.full
        );
    }
}
