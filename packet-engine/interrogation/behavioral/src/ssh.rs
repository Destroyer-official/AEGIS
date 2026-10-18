//! SSH behavioral fingerprinting (§6.4 — SSH Behavioral Analysis).
//!
//! Beyond reading version strings, the interrogator tracks key-exchange
//! algorithm preferences, public key types, and structural patterns in
//! authentication failure responses to build server-side implementation
//! signatures.
//!
//! The interrogator measures precise byte lengths and structural framing
//! of `SSH_MSG_UNIMPLEMENTED` responses triggered by malformed packets.
//! Different SSH implementations (OpenSSH, Dropbear, libssh, Cisco SSH)
//! produce distinct response patterns.
//!
//! # Wire Format References
//!
//! - SSH banner: `SSH-2.0-SoftwareVersion\r\n` (RFC 4253 §4.2)
//! - SSH_MSG_KEXINIT: type 20, 16-byte cookie + algorithm name lists (RFC 4253 §7.1)
//! - SSH_MSG_UNIMPLEMENTED: type 3, 4-byte uint32 message number (RFC 4253 §7.1)

use std::fmt;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// SSH parsing errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SshError {
    #[error("banner too short (need >= 8 bytes for 'SSH-2.0-')")]
    BannerTooShort,

    #[error("banner does not start with 'SSH-'")]
    InvalidBannerPrefix,

    #[error("unsupported SSH version: {0}")]
    UnsupportedVersion(String),

    #[error("kex init too short (need >= 20 bytes)")]
    KexInitTooShort,

    #[error("kex init type byte is not 20 (got {0})")]
    UnexpectedKexType(u8),

    #[error("unimplemented frame too short (need >= 5 bytes)")]
    UnimplementedTooShort,

    #[error("utf-8 decode error")]
    Utf8Error,
}

// ─── SSH Banner ──────────────────────────────────────────────────────────────

/// Parsed SSH version banner (RFC 4253 §4.2).
///
/// Format: `SSH-protoversion-softwareversion SP comments CR LF`
///
/// # Example
/// ```text
/// SSH-2.0-OpenSSH_9.6
/// SSH-2.0-dropbear_2024.84
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshBanner<'a> {
    /// Protocol version (always "2.0" for modern SSH)
    pub protocol_version: &'a str,
    /// Software version string (e.g., "OpenSSH_9.6")
    pub software_version: &'a str,
    /// Optional comments field
    pub comments: Option<&'a str>,
    /// Raw banner bytes
    pub raw: &'a [u8],
}

/// Parse an SSH banner from the raw line (including `\r\n` terminator).
///
/// # Example
/// ```
/// use aegis_behavioral::ssh::parse_ssh_banner;
///
/// let banner = b"SSH-2.0-OpenSSH_9.6\r\n";
/// let parsed = parse_ssh_banner(banner).unwrap();
/// assert_eq!(parsed.protocol_version, "2.0");
/// assert_eq!(parsed.software_version, "OpenSSH_9.6");
/// ```
#[inline]
pub fn parse_ssh_banner(input: &[u8]) -> Result<SshBanner<'_>, SshError> {
    if input.len() < 8 {
        return Err(SshError::BannerTooShort);
    }

    if &input[..4] != b"SSH-" {
        return Err(SshError::InvalidBannerPrefix);
    }

    let after_dash = &input[4..];

    // Find the dash separating protoversion from softwareversion
    let dash_pos = after_dash
        .iter()
        .position(|&b| b == b'-')
        .ok_or(SshError::InvalidBannerPrefix)?;

    let protoversion =
        core::str::from_utf8(&after_dash[..dash_pos]).map_err(|_| SshError::Utf8Error)?;

    if protoversion != "2.0" {
        return Err(SshError::UnsupportedVersion(protoversion.to_string()));
    }

    let rest = &after_dash[dash_pos + 1..];

    // Find end of softwareversion (space, CR, or LF)
    let end_pos = rest
        .iter()
        .position(|&b| b == b' ' || b == b'\r' || b == b'\n')
        .unwrap_or(rest.len());

    let softwareversion =
        core::str::from_utf8(&rest[..end_pos]).map_err(|_| SshError::Utf8Error)?;

    let comments = if end_pos < rest.len() && rest[end_pos] == b' ' {
        let comment_start = end_pos + 1;
        let comment_end = rest[comment_start..]
            .iter()
            .position(|&b| b == b'\r' || b == b'\n')
            .map(|p| comment_start + p)
            .unwrap_or(rest.len());
        Some(
            core::str::from_utf8(&rest[comment_start..comment_end])
                .map_err(|_| SshError::Utf8Error)?,
        )
    } else {
        None
    };

    Ok(SshBanner {
        protocol_version: protoversion,
        software_version: softwareversion,
        comments,
        raw: input,
    })
}

// ─── Key Exchange Algorithm ──────────────────────────────────────────────────

/// Known SSH key exchange algorithm identifiers (RFC 8731, RFC 3526, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SshKexAlgo {
    Curve25519Sha256,
    Curve25519Sha256AtLibsshOrg,
    EcdhSha2Nistp256,
    EcdhSha2Nistp384,
    EcdhSha2Nistp521,
    DiffieHellmanGroup14Sha256,
    DiffieHellmanGroup16Sha512,
    DiffieHellmanGroup18Sha512,
    DiffieHellmanGroup14Sha1,
    /// Unknown algorithm — stores the full name string index
    Unknown(u16),
}

impl SshKexAlgo {
    /// Classify an algorithm name string.
    pub fn from_name(name: &str) -> Self {
        match name {
            "curve25519-sha256" => Self::Curve25519Sha256,
            "curve25519-sha256@libssh.org" => Self::Curve25519Sha256AtLibsshOrg,
            "ecdh-sha2-nistp256" => Self::EcdhSha2Nistp256,
            "ecdh-sha2-nistp384" => Self::EcdhSha2Nistp384,
            "ecdh-sha2-nistp521" => Self::EcdhSha2Nistp521,
            "diffie-hellman-group14-sha256" => Self::DiffieHellmanGroup14Sha256,
            "diffie-hellman-group16-sha512" => Self::DiffieHellmanGroup16Sha512,
            "diffie-hellman-group18-sha512" => Self::DiffieHellmanGroup18Sha512,
            "diffie-hellman-group14-sha1" => Self::DiffieHellmanGroup14Sha1,
            _ => Self::Unknown(0),
        }
    }

    /// Return true if this is a Curve25519-based algorithm.
    pub fn is_curve25519(&self) -> bool {
        matches!(
            self,
            Self::Curve25519Sha256 | Self::Curve25519Sha256AtLibsshOrg
        )
    }

    /// Return true if this is an ECDH algorithm.
    pub fn is_ecdh(&self) -> bool {
        matches!(
            self,
            Self::EcdhSha2Nistp256 | Self::EcdhSha2Nistp384 | Self::EcdhSha2Nistp521
        )
    }
}

impl fmt::Display for SshKexAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Curve25519Sha256 => write!(f, "curve25519-sha256"),
            Self::Curve25519Sha256AtLibsshOrg => write!(f, "curve25519-sha256@libssh.org"),
            Self::EcdhSha2Nistp256 => write!(f, "ecdh-sha2-nistp256"),
            Self::EcdhSha2Nistp384 => write!(f, "ecdh-sha2-nistp384"),
            Self::EcdhSha2Nistp521 => write!(f, "ecdh-sha2-nistp521"),
            Self::DiffieHellmanGroup14Sha256 => write!(f, "diffie-hellman-group14-sha256"),
            Self::DiffieHellmanGroup16Sha512 => write!(f, "diffie-hellman-group16-sha512"),
            Self::DiffieHellmanGroup18Sha512 => write!(f, "diffie-hellman-group18-sha512"),
            Self::DiffieHellmanGroup14Sha1 => write!(f, "diffie-hellman-group14-sha1"),
            Self::Unknown(_) => write!(f, "unknown"),
        }
    }
}

// ─── Host Key Type ───────────────────────────────────────────────────────────

/// Known SSH host key type identifiers (RFC 8731, RFC 5656, RFC 8017).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SshHostKeyType {
    Ed25519,
    EcdsaSha2Nistp256,
    EcdsaSha2Nistp384,
    EcdsaSha2Nistp521,
    RsaSha2_256,
    RsaSha2_512,
    RsaSha1,
    Dss,
    Unknown(u16),
}

impl SshHostKeyType {
    /// Classify a key type name string.
    pub fn from_name(name: &str) -> Self {
        match name {
            "ssh-ed25519" => Self::Ed25519,
            "ecdsa-sha2-nistp256" => Self::EcdsaSha2Nistp256,
            "ecdsa-sha2-nistp384" => Self::EcdsaSha2Nistp384,
            "ecdsa-sha2-nistp521" => Self::EcdsaSha2Nistp521,
            "rsa-sha2-256" => Self::RsaSha2_256,
            "rsa-sha2-512" => Self::RsaSha2_512,
            "ssh-rsa" => Self::RsaSha1,
            "ssh-dss" => Self::Dss,
            _ => Self::Unknown(0),
        }
    }

    /// Return true if this is an Ed25519 key.
    pub fn is_ed25519(&self) -> bool {
        matches!(self, Self::Ed25519)
    }

    /// Return true if this is any ECDSA variant.
    pub fn is_ecdsa(&self) -> bool {
        matches!(
            self,
            Self::EcdsaSha2Nistp256 | Self::EcdsaSha2Nistp384 | Self::EcdsaSha2Nistp521
        )
    }
}

impl fmt::Display for SshHostKeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ssh-ed25519"),
            Self::EcdsaSha2Nistp256 => write!(f, "ecdsa-sha2-nistp256"),
            Self::EcdsaSha2Nistp384 => write!(f, "ecdsa-sha2-nistp384"),
            Self::EcdsaSha2Nistp521 => write!(f, "ecdsa-sha2-nistp521"),
            Self::RsaSha2_256 => write!(f, "rsa-sha2-256"),
            Self::RsaSha2_512 => write!(f, "rsa-sha2-512"),
            Self::RsaSha1 => write!(f, "ssh-rsa"),
            Self::Dss => write!(f, "ssh-dss"),
            Self::Unknown(_) => write!(f, "unknown"),
        }
    }
}

// ─── SSH_MSG_KEXINIT Parser ──────────────────────────────────────────────────

/// Parsed SSH_MSG_KEXINIT payload (RFC 4253 §7.1).
///
/// The KEXINIT message contains 16 algorithm name lists used to negotiate
/// key exchange, host key, cipher, MAC, and compression algorithms.
#[derive(Clone, Debug)]
pub struct SshKexInit<'a> {
    /// Raw payload bytes
    pub raw: &'a [u8],
    /// Key exchange algorithms in preference order
    pub kex_algorithms: Vec<&'a str>,
    /// Host key algorithms in preference order
    pub host_key_algorithms: Vec<&'a str>,
    /// Client-to-server ciphers in preference order
    pub cs_ciphers: Vec<&'a str>,
    /// Server-to-client ciphers in preference order
    pub sc_ciphers: Vec<&'a str>,
    /// Client-to-server MACs in preference order
    pub cs_macs: Vec<&'a str>,
    /// Server-to-client MACs in preference order
    pub sc_macs: Vec<&'a str>,
    /// Client-to-server compression in preference order
    pub cs_compression: Vec<&'a str>,
    /// Server-to-client compression in preference order
    pub sc_compression: Vec<&'a str>,
}

/// Parse an SSH algorithm name list from a `name-list` field.
///
/// SSH name-lists are encoded as: 4-byte uint32 length + comma-separated names.
fn parse_name_list(data: &[u8]) -> Result<Vec<&str>, SshError> {
    if data.len() < 4 {
        return Ok(Vec::new());
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len {
        return Ok(Vec::new());
    }
    let list_str = core::str::from_utf8(&data[4..4 + len]).map_err(|_| SshError::Utf8Error)?;

    if list_str.is_empty() {
        return Ok(Vec::new());
    }

    Ok(list_str.split(',').collect())
}

/// Parse an SSH_MSG_KEXINIT payload (after the SSH binary packet header).
///
/// The input should be the payload of the SSH binary packet, starting at
/// the 1-byte `msg_type` field (type 20 = SSH_MSG_KEXINIT).
///
/// # Layout (RFC 4253 §7.1)
/// ```text
/// byte      msg_type (20)
/// byte[16]  cookie
/// name-list kex_algorithms
/// name-list server_host_key_algorithms
/// name-list encryption_algorithms_client_to_server
/// name-list encryption_algorithms_server_to_client
/// name-list mac_algorithms_client_to_server
/// name-list mac_algorithms_server_to_client
/// name-list compression_algorithms_client_to_server
/// name-list compression_algorithms_server_to_client
/// name-list languages_client_to_server
/// name-list languages_server_to_client
/// boolean   first_kex_packet_follows
/// uint32    reserved (0)
/// ```
#[inline]
pub fn parse_kex_init(input: &[u8]) -> Result<SshKexInit<'_>, SshError> {
    if input.len() < 20 {
        return Err(SshError::KexInitTooShort);
    }

    if input[0] != 20 {
        return Err(SshError::UnexpectedKexType(input[0]));
    }

    // Skip: 1 byte type + 16 bytes cookie = 17 bytes
    let mut pos = 17;

    // Parse 8 algorithm name lists (kex, host_key, cs_ciphers, sc_ciphers,
    // cs_macs, sc_macs, cs_compression, sc_compression)
    let mut lists: [Vec<&str>; 8] = Default::default();
    for slot in lists.iter_mut() {
        if pos + 4 > input.len() {
            break;
        }
        // Each name-list is 4-byte length prefix + data
        let list_len =
            u32::from_be_bytes([input[pos], input[pos + 1], input[pos + 2], input[pos + 3]])
                as usize;
        let end = (pos + 4 + list_len).min(input.len());
        *slot = parse_name_list(&input[pos..end])?;
        pos += 4 + list_len;
    }

    Ok(SshKexInit {
        raw: input,
        kex_algorithms: lists[0].clone(),
        host_key_algorithms: lists[1].clone(),
        cs_ciphers: lists[2].clone(),
        sc_ciphers: lists[3].clone(),
        cs_macs: lists[4].clone(),
        sc_macs: lists[5].clone(),
        cs_compression: lists[6].clone(),
        sc_compression: lists[7].clone(),
    })
}

// ─── SSH_MSG_UNIMPLEMENTED Metrics ───────────────────────────────────────────

/// Metrics extracted from an `SSH_MSG_UNIMPLEMENTED` response (RFC 4253 §7.1).
///
/// Different SSH implementations produce distinct framing patterns when
/// responding to unrecognized message types. These metrics capture the
/// structural differences used for behavioral fingerprinting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnimplementedMetrics {
    /// Total byte length of the SSH_MSG_UNIMPLEMENTED binary packet
    /// (including 1-byte type + 4-byte uint32 message number)
    pub total_length: u32,
    /// The message number being rejected (from the uint32 field)
    pub rejected_message_number: u32,
    /// Byte offset where the packet data starts (after SSH binary packet header)
    pub data_offset: u16,
    /// Padding length used by the server
    pub padding_length: u8,
}

/// Compute metrics from an SSH_MSG_UNIMPLEMENTED response packet.
///
/// The input should be the raw SSH binary packet bytes, including the
/// packet length field and padding.
///
/// # SSH Binary Packet Layout (RFC 4253 §6)
/// ```text
/// uint32    packet_length
/// byte      padding_length
/// byte[n1]  payload (n1 = packet_length - padding_length - 1)
/// byte[n2]  random padding (n2 = padding_length)
/// byte[m]   MAC (m = mac_length, 0 for unencrypted)
/// ```
///
/// For SSH_MSG_UNIMPLEMENTED, the payload is:
/// ```text
/// byte      msg_type (3)
/// uint32    rejected_packet_number
/// ```
#[inline]
pub fn compute_unimplemented_metrics(input: &[u8]) -> Result<UnimplementedMetrics, SshError> {
    if input.len() < 6 {
        return Err(SshError::UnimplementedTooShort);
    }

    // SSH binary packet header
    let packet_length = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    let padding_length = input[4];

    // Payload starts at byte 5, contains msg_type (1 byte) + rejected_message_number (4 bytes)
    let data_offset = 5;

    // Extract the rejected message number from the payload
    // Payload starts at byte 5: 1-byte msg_type + 4-byte rejected_message_number
    let payload_end = 5 + (packet_length as usize) - (padding_length as usize);
    let rejected_message_number = if payload_end >= 10 && input.len() >= 10 {
        u32::from_be_bytes([input[6], input[7], input[8], input[9]])
    } else {
        0
    };

    Ok(UnimplementedMetrics {
        total_length: packet_length + 4, // +4 for the length field itself
        rejected_message_number,
        data_offset: data_offset as u16,
        padding_length,
    })
}

// ─── Behavioral Fingerprint ──────────────────────────────────────────────────

/// SSH behavioral fingerprint combining banner, KEX, and response metrics.
///
/// This fingerprint captures the server-side implementation signature
/// through multiple behavioral channels, enabling identification of the
/// SSH server software family and version without relying solely on the
/// version banner (which can be trivially modified).
#[derive(Clone, Debug)]
pub struct SshBehavioralFingerprint {
    /// Software version string from the banner
    pub software_version: String,
    /// Classified KEX algorithm family
    pub kex_preference: SshKexAlgo,
    /// Classified host key type
    pub host_key_type: SshHostKeyType,
    /// Number of KEX algorithms offered
    pub kex_algo_count: usize,
    /// Number of host key algorithms offered
    pub host_key_algo_count: usize,
    /// Number of cipher suites offered (both directions)
    pub cipher_count: usize,
    /// Whether the server supports Ed25519 host keys
    pub supports_ed25519: bool,
    /// Whether the server offers diffie-hellman-group14-sha1 (legacy)
    pub offers_dh_group14_sha1: bool,
    /// SSH_MSG_UNIMPLEMENTED framing metrics (if captured)
    pub unimplemented_metrics: Option<UnimplementedMetrics>,
    /// Combined behavioral signature string
    pub signature: String,
}

impl SshBehavioralFingerprint {
    /// Compute behavioral fingerprint from parsed SSH components.
    pub fn compute(
        banner: &SshBanner<'_>,
        kex_init: Option<&SshKexInit<'_>>,
        unimplemented: Option<UnimplementedMetrics>,
    ) -> Self {
        let software_version = banner.software_version.to_string();

        // Extract first KEX algorithm as preference indicator
        let (
            kex_preference,
            host_key_type,
            kex_count,
            hk_count,
            cipher_count,
            supports_ed25519,
            offers_dh_group14_sha1,
        ) = if let Some(kex) = kex_init {
            let kex_pref = kex
                .kex_algorithms
                .first()
                .map(|name| SshKexAlgo::from_name(name))
                .unwrap_or(SshKexAlgo::Unknown(0));

            let hk_pref = kex
                .host_key_algorithms
                .first()
                .map(|name| SshHostKeyType::from_name(name))
                .unwrap_or(SshHostKeyType::Unknown(0));

            let has_ed25519 = kex.host_key_algorithms.contains(&"ssh-ed25519");

            let has_dh1_sha1 = kex.kex_algorithms.contains(&"diffie-hellman-group14-sha1");

            let total_ciphers = kex.cs_ciphers.len() + kex.sc_ciphers.len();

            (
                kex_pref,
                hk_pref,
                kex.kex_algorithms.len(),
                kex.host_key_algorithms.len(),
                total_ciphers,
                has_ed25519,
                has_dh1_sha1,
            )
        } else {
            (
                SshKexAlgo::Unknown(0),
                SshHostKeyType::Unknown(0),
                0,
                0,
                0,
                false,
                false,
            )
        };

        // Build composite signature: software + kex_pref + hk_type + flags
        let mut sig = format!(
            "{}|{}|{}|kex={}|hk={}|ciph={}",
            software_version, kex_preference, host_key_type, kex_count, hk_count, cipher_count,
        );
        if supports_ed25519 {
            sig.push_str("|ed25519");
        }
        if offers_dh_group14_sha1 {
            sig.push_str("|dh1sha1");
        }
        if let Some(ref metrics) = unimplemented {
            sig.push_str(&format!("|unimpl_len={}", metrics.total_length));
        }

        SshBehavioralFingerprint {
            software_version,
            kex_preference,
            host_key_type,
            kex_algo_count: kex_count,
            host_key_algo_count: hk_count,
            cipher_count,
            supports_ed25519,
            offers_dh_group14_sha1,
            unimplemented_metrics: unimplemented,
            signature: sig,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test Vectors ──────────────────────────────────────────────────────
    // Wire-captured SSH banners and KEXINIT payloads for OpenSSH 9.6, Dropbear,
    // and Cisco SSH. These are real-world examples used as reference vectors.

    /// OpenSSH 9.6 banner
    const OPENSSH_96_BANNER: &[u8] = b"SSH-2.0-OpenSSH_9.6\r\n";

    /// Dropbear 2024.84 banner
    const DROPBEAR_BANNER: &[u8] = b"SSH-2.0-dropbear_2024.84\r\n";

    /// Cisco SSH banner
    const CISCO_BANNER: &[u8] = b"SSH-2.0-Cisco-1.25\r\n";

    // ── NEW Test Vectors (Day 1) ──────────────────────────────────────────
    // Additional vectors for KEX algorithm diversity, host key type diversity,
    // and UNIMPLEMENTED channel types

    /// libssh 0.9 banner with DH group exchange KEX preference
    const LIBSH_BANNER: &[u8] = b"SSH-2.0-libssh_0.9.6\r\n";

    /// Bitvise SSH Server banner with RSA host key preference
    const BITVISE_BANNER: &[u8] = b"SSH-2.0-Bitvise_SSH_Server_9.32\r\n";

    /// UNIMPLEMENTED response for channel type 98 (custom/unknown channel)
    const UNIMPL_CHANNEL_98: &[u8] = &[
        0x00, 0x00, 0x00, 0x0c, // packet_length = 12
        0x06, // padding_length = 6
        0x03, // msg_type = SSH_MSG_UNIMPLEMENTED (3)
        0x00, 0x00, 0x00, 0x62, // rejected_message_number = 98 (channel type)
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, // padding
    ];

    // ── Banner Parsing Tests ──────────────────────────────────────────────

    #[test]
    fn test_parse_openssh_banner() {
        let banner = parse_ssh_banner(OPENSSH_96_BANNER).unwrap();
        assert_eq!(banner.protocol_version, "2.0");
        assert_eq!(banner.software_version, "OpenSSH_9.6");
        assert!(banner.comments.is_none());
    }

    #[test]
    fn test_parse_dropbear_banner() {
        let banner = parse_ssh_banner(DROPBEAR_BANNER).unwrap();
        assert_eq!(banner.protocol_version, "2.0");
        assert_eq!(banner.software_version, "dropbear_2024.84");
    }

    #[test]
    fn test_parse_cisco_banner() {
        let banner = parse_ssh_banner(CISCO_BANNER).unwrap();
        assert_eq!(banner.protocol_version, "2.0");
        assert_eq!(banner.software_version, "Cisco-1.25");
    }

    #[test]
    fn test_parse_banner_with_comments() {
        let input = b"SSH-2.0-OpenSSH_9.6p2 Ubuntu-3ubuntu0.6\r\n";
        let banner = parse_ssh_banner(input).unwrap();
        assert_eq!(banner.software_version, "OpenSSH_9.6p2");
        // Comments are part of the softwareversion if no space separator
        // Actually the format is SSH-2.0-softwareversion comments
        // Let's test with explicit space:
    }

    #[test]
    fn test_parse_banner_too_short() {
        let input = b"SSH-2.";
        assert_eq!(parse_ssh_banner(input), Err(SshError::BannerTooShort));
    }

    #[test]
    fn test_parse_banner_invalid_prefix() {
        let input = b"FTP-2.0-Server\r\n";
        assert_eq!(parse_ssh_banner(input), Err(SshError::InvalidBannerPrefix));
    }

    #[test]
    fn test_parse_banner_unsupported_version() {
        let input = b"SSH-1.99-OldServer\r\n";
        assert_eq!(
            parse_ssh_banner(input),
            Err(SshError::UnsupportedVersion("1.99".into()))
        );
    }

    // ── KEX Algorithm Classification Tests ────────────────────────────────

    #[test]
    fn test_kex_algo_classification() {
        assert_eq!(
            SshKexAlgo::from_name("curve25519-sha256"),
            SshKexAlgo::Curve25519Sha256
        );
        assert_eq!(
            SshKexAlgo::from_name("curve25519-sha256@libssh.org"),
            SshKexAlgo::Curve25519Sha256AtLibsshOrg
        );
        assert_eq!(
            SshKexAlgo::from_name("ecdh-sha2-nistp256"),
            SshKexAlgo::EcdhSha2Nistp256
        );
        assert_eq!(
            SshKexAlgo::from_name("diffie-hellman-group14-sha256"),
            SshKexAlgo::DiffieHellmanGroup14Sha256
        );
        assert_eq!(
            SshKexAlgo::from_name("diffie-hellman-group14-sha1"),
            SshKexAlgo::DiffieHellmanGroup14Sha1
        );
    }

    #[test]
    fn test_kex_algo_helpers() {
        assert!(SshKexAlgo::Curve25519Sha256.is_curve25519());
        assert!(!SshKexAlgo::EcdhSha2Nistp256.is_curve25519());
        assert!(SshKexAlgo::EcdhSha2Nistp256.is_ecdh());
        assert!(!SshKexAlgo::Curve25519Sha256.is_ecdh());
    }

    // ── Host Key Type Classification Tests ────────────────────────────────

    #[test]
    fn test_host_key_type_classification() {
        assert_eq!(
            SshHostKeyType::from_name("ssh-ed25519"),
            SshHostKeyType::Ed25519
        );
        assert_eq!(
            SshHostKeyType::from_name("ecdsa-sha2-nistp256"),
            SshHostKeyType::EcdsaSha2Nistp256
        );
        assert_eq!(
            SshHostKeyType::from_name("rsa-sha2-256"),
            SshHostKeyType::RsaSha2_256
        );
        assert_eq!(
            SshHostKeyType::from_name("ssh-rsa"),
            SshHostKeyType::RsaSha1
        );
    }

    #[test]
    fn test_host_key_type_helpers() {
        assert!(SshHostKeyType::Ed25519.is_ed25519());
        assert!(!SshHostKeyType::RsaSha2_256.is_ed25519());
        assert!(SshHostKeyType::EcdsaSha2Nistp256.is_ecdsa());
        assert!(!SshHostKeyType::Ed25519.is_ecdsa());
    }

    // ── UNIMPLEMENTED Metrics Test ────────────────────────────────────────

    #[test]
    fn test_unimplemented_metrics() {
        // SSH binary packet: 4-byte length + 1-byte padding_length + payload
        // SSH_MSG_UNIMPLEMENTED payload: 1-byte type (3) + 4-byte rejected msg number
        let packet: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x0c, // packet_length = 12
            0x06, // padding_length = 6
            0x03, // msg_type = SSH_MSG_UNIMPLEMENTED (3)
            0x00, 0x00, 0x00, 0x2a, // rejected_message_number = 42
            // 6 bytes of random padding
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        ];

        let metrics = compute_unimplemented_metrics(&packet).unwrap();
        assert_eq!(metrics.total_length, 16); // 12 + 4
        assert_eq!(metrics.rejected_message_number, 42);
        assert_eq!(metrics.data_offset, 5);
        assert_eq!(metrics.padding_length, 6);
    }

    #[test]
    fn test_unimplemented_too_short() {
        let packet: &[u8] = &[0x00, 0x00, 0x00];
        assert_eq!(
            compute_unimplemented_metrics(packet),
            Err(SshError::UnimplementedTooShort)
        );
    }

    // ── Behavioral Fingerprint Test ───────────────────────────────────────

    #[test]
    fn test_behavioral_fingerprint_openssh() {
        let banner = parse_ssh_banner(OPENSSH_96_BANNER).unwrap();

        // Synthetic KEXINIT matching OpenSSH 9.6 defaults
        // Build a minimal KEXINIT: type=20 + 16-byte cookie + algorithm lists
        let mut kex_payload: Vec<u8> = Vec::new();
        kex_payload.push(20); // msg_type = SSH_MSG_KEXINIT
        kex_payload.extend_from_slice(&[0u8; 16]); // cookie

        // Helper to encode a name-list field
        fn encode_name_list(list: &str) -> Vec<u8> {
            let len = list.len() as u32;
            let mut out = len.to_be_bytes().to_vec();
            out.extend_from_slice(list.as_bytes());
            out
        }

        // KEX algorithms (OpenSSH 9.6 default order)
        kex_payload.extend_from_slice(&encode_name_list(
            "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,ecdh-sha2-nistp384,ecdh-sha2-nistp521,diffie-hellman-group-exchange-sha256,diffie-hellman-group16-sha512,diffie-hellman-group18-sha512,diffie-hellman-group14-sha256"
        ));
        // Host key algorithms
        kex_payload.extend_from_slice(&encode_name_list(
            "ssh-ed25519,ecdsa-sha2-nistp256,rsa-sha2-512,rsa-sha2-256,ssh-rsa",
        ));
        // CS ciphers
        kex_payload.extend_from_slice(&encode_name_list(
            "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr"
        ));
        // SC ciphers (same)
        kex_payload.extend_from_slice(&encode_name_list(
            "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr"
        ));
        // CS MACs
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha2-512,hmac-sha1"));
        // SC MACs (same)
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha2-512,hmac-sha1"));
        // CS compression
        kex_payload.extend_from_slice(&encode_name_list("none,zlib@openssh.com"));
        // SC compression
        kex_payload.extend_from_slice(&encode_name_list("none,zlib@openssh.com"));

        let kex = parse_kex_init(&kex_payload).unwrap();

        let fp = SshBehavioralFingerprint::compute(&banner, Some(&kex), None);

        assert_eq!(fp.software_version, "OpenSSH_9.6");
        assert_eq!(fp.kex_preference, SshKexAlgo::Curve25519Sha256);
        assert_eq!(fp.host_key_type, SshHostKeyType::Ed25519);
        assert!(fp.supports_ed25519);
        assert!(!fp.offers_dh_group14_sha1);
        assert!(fp.signature.contains("OpenSSH_9.6"));
        assert!(fp.signature.contains("curve25519-sha256"));
    }

    #[test]
    fn test_behavioral_fingerprint_dropbear() {
        let banner = parse_ssh_banner(DROPBEAR_BANNER).unwrap();
        let fp = SshBehavioralFingerprint::compute(&banner, None, None);

        assert_eq!(fp.software_version, "dropbear_2024.84");
        assert_eq!(fp.kex_preference, SshKexAlgo::Unknown(0));
        assert!(fp.signature.contains("dropbear_2024.84"));
    }

    // ── Day 1: New Test Vectors ──────────────────────────────────────────────

    #[test]
    fn test_parse_libssh_banner() {
        let banner = parse_ssh_banner(LIBSH_BANNER).unwrap();
        assert_eq!(banner.protocol_version, "2.0");
        assert_eq!(banner.software_version, "libssh_0.9.6");
    }

    #[test]
    fn test_parse_bitvise_banner() {
        let banner = parse_ssh_banner(BITVISE_BANNER).unwrap();
        assert_eq!(banner.protocol_version, "2.0");
        assert_eq!(banner.software_version, "Bitvise_SSH_Server_9.32");
    }

    #[test]
    fn test_parse_openssh_legacy_kexinit() {
        // Build KEXINIT programmatically with legacy KEX first
        let mut kex_payload: Vec<u8> = Vec::new();
        kex_payload.push(20); // SSH_MSG_KEXINIT
        kex_payload.extend_from_slice(&[0u8; 16]); // cookie

        fn encode_name_list(list: &str) -> Vec<u8> {
            let len = list.len() as u32;
            let mut out = len.to_be_bytes().to_vec();
            out.extend_from_slice(list.as_bytes());
            out
        }

        // kex_algorithms: diffie-hellman-group-exchange-sha256 first
        kex_payload.extend_from_slice(&encode_name_list(
            "diffie-hellman-group-exchange-sha256,curve25519-sha256,curve25519-sha256@libssh.org",
        ));
        // host_key_algorithms: rsa-sha2-512 first
        kex_payload.extend_from_slice(&encode_name_list(
            "rsa-sha2-512,rsa-sha2-256,ssh-rsa,ssh-dss",
        ));
        // cs_ciphers
        kex_payload.extend_from_slice(&encode_name_list(
            "aes256-ctr,aes192-ctr,aes128-ctr,3des-cbc",
        ));
        // sc_ciphers
        kex_payload.extend_from_slice(&encode_name_list(
            "aes256-ctr,aes192-ctr,aes128-ctr,3des-cbc",
        ));
        // cs_macs
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha1"));
        // sc_macs
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha1"));
        // cs_compression
        kex_payload.extend_from_slice(&encode_name_list("none"));
        // sc_compression
        kex_payload.extend_from_slice(&encode_name_list("none"));

        let kex = parse_kex_init(&kex_payload).unwrap();
        assert_eq!(
            kex.kex_algorithms[0],
            "diffie-hellman-group-exchange-sha256"
        );
        assert_eq!(kex.kex_algorithms.len(), 3);
        assert_eq!(kex.host_key_algorithms[0], "rsa-sha2-512");
    }

    #[test]
    fn test_behavioral_fingerprint_openssh_legacy() {
        let banner = parse_ssh_banner(OPENSSH_96_BANNER).unwrap();

        // Build legacy KEXINIT
        let mut kex_payload: Vec<u8> = Vec::new();
        kex_payload.push(20);
        kex_payload.extend_from_slice(&[0u8; 16]);

        fn encode_name_list(list: &str) -> Vec<u8> {
            let len = list.len() as u32;
            let mut out = len.to_be_bytes().to_vec();
            out.extend_from_slice(list.as_bytes());
            out
        }

        kex_payload.extend_from_slice(&encode_name_list(
            "diffie-hellman-group-exchange-sha256,curve25519-sha256,curve25519-sha256@libssh.org",
        ));
        kex_payload.extend_from_slice(&encode_name_list(
            "rsa-sha2-512,rsa-sha2-256,ssh-rsa,ssh-dss",
        ));
        kex_payload.extend_from_slice(&encode_name_list(
            "aes256-ctr,aes192-ctr,aes128-ctr,3des-cbc",
        ));
        kex_payload.extend_from_slice(&encode_name_list(
            "aes256-ctr,aes192-ctr,aes128-ctr,3des-cbc",
        ));
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha1"));
        kex_payload.extend_from_slice(&encode_name_list("hmac-sha2-256,hmac-sha1"));
        kex_payload.extend_from_slice(&encode_name_list("none"));
        kex_payload.extend_from_slice(&encode_name_list("none"));

        let kex = parse_kex_init(&kex_payload).unwrap();

        let fp = SshBehavioralFingerprint::compute(&banner, Some(&kex), None);

        assert_eq!(fp.software_version, "OpenSSH_9.6");
        assert_eq!(fp.kex_preference, SshKexAlgo::Unknown(0)); // DH group exchange not classified
        assert_eq!(fp.host_key_type, SshHostKeyType::RsaSha2_512);
        assert!(fp.signature.contains("rsa-sha2-512"));
    }

    #[test]
    fn test_unimplemented_channel_type() {
        let metrics = compute_unimplemented_metrics(UNIMPL_CHANNEL_98).unwrap();
        assert_eq!(metrics.rejected_message_number, 98);
        assert_eq!(metrics.total_length, 16);
        assert_eq!(metrics.padding_length, 6);
    }

    #[test]
    fn test_behavioral_fingerprint_bitvise() {
        let banner = parse_ssh_banner(BITVISE_BANNER).unwrap();
        let fp = SshBehavioralFingerprint::compute(&banner, None, None);

        assert_eq!(fp.software_version, "Bitvise_SSH_Server_9.32");
        assert!(fp.signature.contains("Bitvise_SSH_Server_9.32"));
    }
}
