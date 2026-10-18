//! JA4T — TCP client OS fingerprint from SYN packets (§6.1).
//!
//! JA4T format: `{mss}_{wscale}_{sack}_{tcp_options}_{ttl}`
//!
//! Fingerprints TCP stack implementation from initial SYN packet fields:
//! window size, MSS, window scale, SACK, TCP option ordering, TTL.

use sha2::{Digest, Sha256};

/// JA4T TCP client fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4tFingerprint {
    pub full: String,
    pub mss: u16,
    pub wscale: u8,
    pub sack: bool,
    pub tcp_options: String,
    pub ttl: u8,
}

/// TCP options parsed from a SYN packet for JA4T.
#[derive(Clone, Debug, Default)]
pub struct TcpOptions {
    pub mss: Option<u16>,
    pub window_scale: Option<u8>,
    pub sack_permitted: bool,
    pub timestamp: Option<u64>,
    /// Option type bytes in original order (for hashing)
    pub option_order: Vec<u8>,
}

/// Parse TCP options from raw bytes (after the fixed 20-byte TCP header).
pub fn parse_tcp_options(data: &[u8]) -> TcpOptions {
    let mut opts = TcpOptions::default();
    let mut pos = 0;

    while pos < data.len() {
        let opt_type = data[pos];
        match opt_type {
            0 => break, // EOL
            1 => {
                pos += 1;
                continue;
            } // NOP
            2 => {
                // MSS
                if pos + 4 <= data.len() {
                    opts.mss = Some(u16::from_be_bytes([data[pos + 2], data[pos + 3]]));
                    opts.option_order.push(2);
                }
                pos += 4;
            }
            3 => {
                // Window Scale
                if pos + 3 <= data.len() {
                    opts.window_scale = Some(data[pos + 2]);
                    opts.option_order.push(3);
                }
                pos += 3;
            }
            4 => {
                // SACK Permitted
                opts.sack_permitted = true;
                opts.option_order.push(4);
                pos += 2;
            }
            8 => {
                // Timestamp
                if pos + 10 <= data.len() {
                    let ts = u64::from_be_bytes([
                        0,
                        0,
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                    ]);
                    opts.timestamp = Some(ts);
                    opts.option_order.push(8);
                }
                pos += 10;
            }
            _ => {
                // Unknown option — skip based on length
                if pos + 1 < data.len() {
                    let len = data[pos + 1] as usize;
                    if len >= 2 {
                        pos += len;
                    } else {
                        pos += 1;
                    }
                } else {
                    pos += 1;
                }
            }
        }
    }

    opts
}

impl Ja4tFingerprint {
    /// Compute JA4T fingerprint from TCP SYN fields.
    pub fn compute(mss: u16, wscale: u8, sack: bool, options: &TcpOptions, ttl: u8) -> Self {
        let opt_order_hex: Vec<String> = options
            .option_order
            .iter()
            .map(|o| format!("{:02x}", o))
            .collect();
        let opt_str = opt_order_hex.join(",");

        let full = format!(
            "{:04x}_{:02x}_{}_{:02x}_{}",
            mss,
            wscale,
            if sack { "1" } else { "0" },
            ttl,
            Sha256::digest(opt_str.as_bytes())[..6]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );

        Ja4tFingerprint {
            full,
            mss,
            wscale,
            sack,
            tcp_options: opt_str,
            ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4t_format() {
        let opts = TcpOptions {
            mss: Some(1460),
            window_scale: Some(7),
            sack_permitted: true,
            timestamp: None,
            option_order: vec![2, 3, 4],
        };
        let ja4t = Ja4tFingerprint::compute(1460, 7, true, &opts, 64);
        assert!(
            ja4t.full.starts_with("05b4_07_1_40_"),
            "JA4T format: {}",
            ja4t.full
        );
        assert_eq!(ja4t.mss, 1460);
        assert_eq!(ja4t.wscale, 7);
        assert!(ja4t.sack);
    }

    #[test]
    fn test_parse_tcp_options() {
        // MSS=1460, WS=7, SACK permitted
        let raw = [
            0x02, 0x04, 0x05, 0xB4, // MSS
            0x01, // NOP
            0x03, 0x03, 0x07, // WS
            0x01, // NOP
            0x04, 0x02, // SACK
        ];
        let opts = parse_tcp_options(&raw);
        assert_eq!(opts.mss, Some(1460));
        assert_eq!(opts.window_scale, Some(7));
        assert!(opts.sack_permitted);
        assert_eq!(opts.option_order, vec![2, 3, 4]);
    }
}
