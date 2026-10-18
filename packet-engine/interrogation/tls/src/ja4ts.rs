//! JA4TS — TCP server fingerprint from SYN-ACK packets (§6.1).
//!
//! JA4TS format: `{wscale}_{mss}_{sack}_{tcp_options}_{ttl}`
//!
//! Fingerprints TCP server stack from SYN-ACK response fields.

use crate::ja4t::TcpOptions;
use sha2::{Digest, Sha256};

/// JA4TS TCP server fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4tsFingerprint {
    pub full: String,
    pub wscale: u8,
    pub mss: u16,
    pub sack: bool,
    pub tcp_options_hash: String,
    pub ttl: u8,
}

impl Ja4tsFingerprint {
    /// Compute JA4TS fingerprint from TCP SYN-ACK fields.
    pub fn compute(mss: u16, wscale: u8, sack: bool, options: &TcpOptions, ttl: u8) -> Self {
        let opt_order_hex: Vec<String> = options
            .option_order
            .iter()
            .map(|o| format!("{:02x}", o))
            .collect();
        let opt_str = opt_order_hex.join(",");
        let opt_hash = hex::encode(&Sha256::digest(opt_str.as_bytes())[..6]);

        let full = format!(
            "{:02x}_{:04x}_{}_{:02x}_{}",
            wscale,
            mss,
            if sack { "1" } else { "0" },
            ttl,
            opt_hash
        );

        Ja4tsFingerprint {
            full,
            wscale,
            mss,
            sack,
            tcp_options_hash: opt_hash,
            ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4ts_format() {
        let opts = TcpOptions {
            mss: Some(1440),
            window_scale: Some(8),
            sack_permitted: true,
            timestamp: None,
            option_order: vec![2, 3, 4],
        };
        let ja4ts = Ja4tsFingerprint::compute(1440, 8, true, &opts, 128);
        assert!(
            ja4ts.full.starts_with("08_05a0_1_80_"),
            "JA4TS format: {}",
            ja4ts.full
        );
    }
}
