//! JA4D — DHCPv4/v6 per-packet fingerprinting (§6.1).
//!
//! JA4D format: `{proto}{msg_type}_{option_hash}_{vendor_hash}`
//!
//! Fingerprints DHCP clients based on:
//! - Message type (Discover, Offer, Request, etc.)
//! - DHCP option ordering and values
//! - Vendor class identifier

use sha2::{Digest, Sha256};

/// DHCP protocol type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DhcpProto {
    V4,
    V6,
}

impl DhcpProto {
    pub fn ja4_char(&self) -> char {
        match self {
            Self::V4 => '4',
            Self::V6 => '6',
        }
    }
}

/// DHCPv4 message type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DhcpV4MessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
    Inform = 8,
}

/// DHCPv6 message type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DhcpV6MessageType {
    Solicit = 1,
    Advertise = 2,
    Request = 3,
    Confirm = 4,
    Renew = 5,
    Rebind = 6,
    Reply = 7,
    Release = 8,
    Decline = 9,
    Reconfigure = 10,
    InformationRequest = 11,
}

/// JA4D DHCP fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4dFingerprint {
    pub full: String,
    pub proto: char,
    pub msg_type: u8,
    pub option_hash: String,
    pub vendor_hash: String,
}

/// DHCP option for fingerprinting.
#[derive(Clone, Debug)]
pub struct DhcpOption<'a> {
    pub code: u8,
    pub data: &'a [u8],
}

/// DHCP packet data for JA4D computation.
#[derive(Clone, Debug)]
pub struct DhcpPacket<'a> {
    pub proto: DhcpProto,
    pub message_type: u8,
    pub options: Vec<DhcpOption<'a>>,
    pub vendor_class: Option<&'a [u8]>,
}

impl Ja4dFingerprint {
    /// Compute JA4D fingerprint from a parsed DHCP packet.
    pub fn compute(pkt: &DhcpPacket<'_>) -> Self {
        // Option ordering hash
        let option_codes: Vec<String> = pkt
            .options
            .iter()
            .map(|o| format!("{:02x}", o.code))
            .collect();
        let option_str = option_codes.join(",");
        let option_hash = if option_str.is_empty() {
            "000000000000".to_string()
        } else {
            let hash = Sha256::digest(option_str.as_bytes());
            hex::encode(&hash[..6])
        };

        // Vendor class hash
        let vendor_hash = if let Some(vc) = pkt.vendor_class {
            let hash = Sha256::digest(vc);
            hex::encode(&hash[..6])
        } else {
            "000000000000".to_string()
        };

        let full = format!(
            "{}{:02x}_{}_{}",
            pkt.proto.ja4_char(),
            pkt.message_type,
            option_hash,
            vendor_hash,
        );

        Ja4dFingerprint {
            full,
            proto: pkt.proto.ja4_char(),
            msg_type: pkt.message_type,
            option_hash,
            vendor_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4d_format() {
        let pkt = DhcpPacket {
            proto: DhcpProto::V4,
            message_type: 1, // DISCOVER
            options: vec![
                DhcpOption {
                    code: 53,
                    data: &[1],
                }, // Message Type
                DhcpOption {
                    code: 61,
                    data: &[1, 0xaa, 0xbb],
                }, // Client ID
                DhcpOption {
                    code: 55,
                    data: &[1, 3, 6],
                }, // Parameter Request List
            ],
            vendor_class: Some(b"MSFT 10.0"),
        };
        let ja4d = Ja4dFingerprint::compute(&pkt);
        // proto='4', message_type=1 → "01" in hex
        assert!(
            ja4d.full.starts_with("401_"),
            "JA4D should start with 401_, got: {}",
            ja4d.full
        );
        assert_ne!(ja4d.vendor_hash, "000000000000");
    }

    #[test]
    fn test_ja4d_v6() {
        let pkt = DhcpPacket {
            proto: DhcpProto::V6,
            message_type: 1, // SOLICIT
            options: vec![],
            vendor_class: None,
        };
        let ja4d = Ja4dFingerprint::compute(&pkt);
        // proto='6', message_type=1 → "01" in hex
        assert!(
            ja4d.full.starts_with("601_"),
            "JA4Dv6 should start with 601_, got: {}",
            ja4d.full
        );
    }
}
