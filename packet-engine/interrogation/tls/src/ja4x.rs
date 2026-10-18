//! JA4X — X.509 certificate structure fingerprint (§6.1).
//!
//! JA4X format: `{oid_sequence_hash}_{san_count}_{ext_count}`
//!
//! Fingerprints X.509 certificates based on OID (Object Identifier) sequences,
//! SAN (Subject Alternative Name) count, and extension count.
//! This enables clustering certificates by issuing CA and configuration patterns.

use sha2::{Digest, Sha256};

/// JA4X certificate structure fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4xFingerprint {
    pub full: String,
    pub oid_hash: String,
    pub san_count: u32,
    pub ext_count: u32,
}

/// A minimal X.509 certificate structure for JA4X computation.
#[derive(Clone, Debug)]
pub struct X509Cert<'a> {
    /// DER-encoded certificate bytes
    pub der: &'a [u8],
    /// OID sequences extracted from the certificate (e.g., key algorithm, signature algorithm)
    pub oids: Vec<Vec<u32>>,
    /// Number of Subject Alternative Names
    pub san_count: u32,
    /// Number of X.509 extensions
    pub ext_count: u32,
}

impl Ja4xFingerprint {
    /// Compute JA4X fingerprint from a parsed X.509 certificate structure.
    pub fn compute(cert: &X509Cert<'_>) -> Self {
        // OID sequence hash: flatten all OIDs into a string, SHA-256, truncate
        let oid_str: String = cert
            .oids
            .iter()
            .map(|oid| {
                oid.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .collect::<Vec<_>>()
            .join(",");

        let oid_hash = if oid_str.is_empty() {
            "000000000000".to_string()
        } else {
            let hash = Sha256::digest(oid_str.as_bytes());
            hex::encode(&hash[..6])
        };

        let full = format!("{}_{}_{:02x}", oid_hash, cert.san_count, cert.ext_count);

        Ja4xFingerprint {
            full,
            oid_hash,
            san_count: cert.san_count,
            ext_count: cert.ext_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4x_format() {
        let cert = X509Cert {
            der: &[],
            oids: vec![
                vec![1, 2, 840, 113549, 1, 1, 11], // SHA256WithRSAEncryption
            ],
            san_count: 3,
            ext_count: 5,
        };
        let ja4x = Ja4xFingerprint::compute(&cert);
        assert!(ja4x.full.contains("_3_05"), "JA4X format: {}", ja4x.full);
        assert_eq!(ja4x.san_count, 3);
        assert_eq!(ja4x.ext_count, 5);
    }
}
