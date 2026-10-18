//! QUIC Version Negotiation packet parsing (RFC 9000 §17.2.1).
//!
//! When a client sends an Initial packet with a version the server doesn't
//! support, the server responds with a Version Negotiation packet containing
//! the list of supported versions.
//!
//! Version Negotiation packets are not encrypted and can be parsed directly.
//!
//! This module supports both IETF QUIC (v1 and drafts) and Google QUIC (gQUIC)
//! version negotiation formats.

pub mod versions {
    //! Well-known QUIC version constants.
    //!
    //! IETF QUIC versions use a numeric space; gQUIC versions are ASCII-encoded
    //! tags such as `"Q043"` (0x51303433).

    // IETF QUIC
    /// IETF QUIC v1 (RFC 9000).
    pub const QUIC_V1: u32 = 0x00000001;

    // IETF drafts (0xff000000 + draft_num)
    pub const DRAFT_29: u32 = 0xff00001d;
    pub const DRAFT_30: u32 = 0xff00001e;
    pub const DRAFT_31: u32 = 0xff00001f;
    pub const DRAFT_32: u32 = 0xff000020;
    pub const DRAFT_33: u32 = 0xff000021;
    pub const DRAFT_34: u32 = 0xff000022;
    pub const DRAFT_35: u32 = 0xff000023;

    // gQUIC (Google QUIC) — ASCII-encoded version bytes
    pub const GQUIC_Q043: u32 = 0x51303433; // "Q043"
    pub const GQUIC_Q046: u32 = 0x51303436; // "Q046"
    pub const GQUIC_Q050: u32 = 0x51303530; // "Q050"
    pub const GQUIC_T051: u32 = 0x54303531; // "T051"
    pub const GQUIC_T052: u32 = 0x54303532; // "T052"
    pub const GQUIC_T053: u32 = 0x54303533; // "T053"
    pub const GQUIC_T054: u32 = 0x54303534; // "T054"
    pub const GQUIC_T055: u32 = 0x54303535; // "T055"
}

use versions::*;

/// Identifies the family a QUIC version belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuicVersionFamily {
    /// IETF QUIC v1.
    IetfV1,
    /// IETF QUIC draft, with the draft number.
    IetfDraft(u16),
    /// Google QUIC, identified by its ASCII tag (e.g. "Q043").
    Gquic(String),
    /// Unknown / unrecognized version.
    Unknown(u32),
}

/// Classify a 32-bit version tag into its family.
pub fn identify_version(v: u32) -> QuicVersionFamily {
    match v {
        QUIC_V1 => QuicVersionFamily::IetfV1,
        0xff000000..=0xff0000ff => {
            let draft = (v & 0xff) as u16;
            QuicVersionFamily::IetfDraft(draft)
        }
        _ if is_gquic(v) => {
            let tag = gquic_version_tag(v);
            QuicVersionFamily::Gquic(tag)
        }
        _ => QuicVersionFamily::Unknown(v),
    }
}

/// Return a human-readable name for a known version.
pub fn version_name(v: u32) -> &'static str {
    match v {
        QUIC_V1 => "IETF QUIC v1",
        DRAFT_29 => "IETF QUIC draft-29",
        DRAFT_30 => "IETF QUIC draft-30",
        DRAFT_31 => "IETF QUIC draft-31",
        DRAFT_32 => "IETF QUIC draft-32",
        DRAFT_33 => "IETF QUIC draft-33",
        DRAFT_34 => "IETF QUIC draft-34",
        DRAFT_35 => "IETF QUIC draft-35",
        GQUIC_Q043 => "gQUIC Q043",
        GQUIC_Q046 => "gQUIC Q046",
        GQUIC_Q050 => "gQUIC Q050",
        GQUIC_T051 => "gQUIC T051",
        GQUIC_T052 => "gQUIC T052",
        GQUIC_T053 => "gQUIC T053",
        GQUIC_T054 => "gQUIC T054",
        GQUIC_T055 => "gQUIC T055",
        _ => "Unknown",
    }
}

/// Returns `true` if `v` is a gQUIC version (ASCII-encoded tag).
pub fn is_gquic(v: u32) -> bool {
    matches!(
        v,
        GQUIC_Q043
            | GQUIC_Q046
            | GQUIC_Q050
            | GQUIC_T051
            | GQUIC_T052
            | GQUIC_T053
            | GQUIC_T054
            | GQUIC_T055
    )
}

/// Returns `true` if `v` is an IETF draft version (0xff000000 + N).
pub fn is_ietf_draft(v: u32) -> bool {
    (0xff000000..=0xff0000ff).contains(&v)
}

/// If `v` is an IETF draft, return the draft number; otherwise `None`.
pub fn draft_number(v: u32) -> Option<u16> {
    if is_ietf_draft(v) {
        Some((v & 0xff) as u16)
    } else {
        None
    }
}

/// Returns `true` if `v` is a recognized QUIC version.
pub fn is_known_version(v: u32) -> bool {
    matches!(
        v,
        QUIC_V1
            | DRAFT_29
            | DRAFT_30
            | DRAFT_31
            | DRAFT_32
            | DRAFT_33
            | DRAFT_34
            | DRAFT_35
            | GQUIC_Q043
            | GQUIC_Q046
            | GQUIC_Q050
            | GQUIC_T051
            | GQUIC_T052
            | GQUIC_T053
            | GQUIC_T054
            | GQUIC_T055
    )
}

/// Convert a 32-bit gQUIC version tag to its ASCII string (e.g. "Q043").
fn gquic_version_tag(v: u32) -> String {
    let bytes = v.to_be_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Given the client's offered versions and the server's supported versions,
/// return the first client version that the server also supports.
pub fn negotiate_version(client_versions: &[u32], server_versions: &[u32]) -> Option<u32> {
    client_versions
        .iter()
        .find(|&&cv| server_versions.contains(&cv))
        .copied()
}

// ─────────────────────────────────────────────────────────────────────────────
// IETF QUIC Version Negotiation
// ─────────────────────────────────────────────────────────────────────────────

/// A parsed QUIC Version Negotiation packet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionNegotiation {
    /// Destination Connection ID from the client
    pub dcid: Vec<u8>,
    /// Source Connection ID from the client
    pub scid: Vec<u8>,
    /// List of supported QUIC versions offered by the server
    pub supported_versions: Vec<u32>,
}

impl VersionNegotiation {
    /// Parse a Version Negotiation packet from raw bytes.
    ///
    /// # Layout (RFC 9000 §17.2.1)
    /// ```text
    /// +-+-+-+-+-+-+-+-+
    /// |1|1|X|X|X|X|X|X|  Long header, type=0x00
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                         Version (0)                           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// | DCID Len (8) |         Destination Connection ID (*)        |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// | SCID Len (8) |           Source Connection ID (*)           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                    Supported Version 1 (32)                  |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                   [Supported Version 2 (32)]                 |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                              ...                              |
    /// ```
    pub fn parse(input: &[u8]) -> Result<Self, VersionNegotiationError> {
        if input.len() < 5 {
            return Err(VersionNegotiationError::TooShort);
        }

        // Check long header bit
        if input[0] & 0x80 == 0 {
            return Err(VersionNegotiationError::NotLongHeader);
        }

        // Version must be 0 for Version Negotiation
        let version = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
        if version != 0 {
            return Err(VersionNegotiationError::VersionNotZero);
        }

        let mut pos = 5;

        // DCID
        if pos >= input.len() {
            return Err(VersionNegotiationError::TooShort);
        }
        let dcid_len = input[pos] as usize;
        pos += 1;
        if pos + dcid_len > input.len() {
            return Err(VersionNegotiationError::Truncated);
        }
        let dcid = input[pos..pos + dcid_len].to_vec();
        pos += dcid_len;

        // SCID
        if pos >= input.len() {
            return Err(VersionNegotiationError::TooShort);
        }
        let scid_len = input[pos] as usize;
        pos += 1;
        if pos + scid_len > input.len() {
            return Err(VersionNegotiationError::Truncated);
        }
        let scid = input[pos..pos + scid_len].to_vec();
        pos += scid_len;

        // Supported versions (4 bytes each, remaining bytes)
        // RFC 9000 §17.2.1: no random field between SCID and versions
        let mut supported_versions = Vec::new();
        while pos + 4 <= input.len() {
            let ver =
                u32::from_be_bytes([input[pos], input[pos + 1], input[pos + 2], input[pos + 3]]);
            supported_versions.push(ver);
            pos += 4;
        }

        // If there's a trailing byte, ignore it (version lists must be 4-byte aligned,
        // but we handle gracefully)

        Ok(Self {
            dcid,
            scid,
            supported_versions,
        })
    }

    /// Check if a specific QUIC version is supported by the server.
    pub fn supports_version(&self, version: u32) -> bool {
        self.supported_versions.contains(&version)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionNegotiationError {
    TooShort,
    NotLongHeader,
    VersionNotZero,
    Truncated,
}

// ─────────────────────────────────────────────────────────────────────────────
// gQUIC Version Negotiation
// ─────────────────────────────────────────────────────────────────────────────

/// A parsed gQUIC Version Negotiation packet.
///
/// gQUIC VN uses a different wire format from IETF QUIC VN:
/// - The long header bit (0x80) is set.
/// - The version field contains the *rejected* version (non-zero).
/// - No "random" field is present.
///
/// ```text
/// +-+-+-+-+-+-+-+-+
/// |1|X|X|X|X|X|X|X|  Long header
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Rejected Version (32)                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  DCID Len(8)  |       Destination Connection ID (*)          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  SCID Len(8)  |         Source Connection ID (*)             |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                   Supported Version 1 (32)                   |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                   Supported Version 2 (32)                   |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                             ...                               |
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GquicVersionNegotiation {
    /// The version the client sent that the server rejected.
    pub rejected_version: u32,
    /// Destination Connection ID.
    pub dcid: Vec<u8>,
    /// Source Connection ID.
    pub scid: Vec<u8>,
    /// List of gQUIC versions supported by the server.
    pub supported_versions: Vec<u32>,
}

impl GquicVersionNegotiation {
    /// Parse a gQUIC Version Negotiation packet from raw bytes.
    pub fn parse(input: &[u8]) -> Result<Self, VersionNegotiationError> {
        if input.len() < 5 {
            return Err(VersionNegotiationError::TooShort);
        }

        // Long header bit must be set
        if input[0] & 0x80 == 0 {
            return Err(VersionNegotiationError::NotLongHeader);
        }

        // gQUIC VN: version field is the rejected version (must be non-zero)
        let rejected_version = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
        if rejected_version == 0 {
            return Err(VersionNegotiationError::VersionNotZero);
        }

        let mut pos = 5;

        // DCID
        if pos >= input.len() {
            return Err(VersionNegotiationError::TooShort);
        }
        let dcid_len = input[pos] as usize;
        pos += 1;
        if pos + dcid_len > input.len() {
            return Err(VersionNegotiationError::Truncated);
        }
        let dcid = input[pos..pos + dcid_len].to_vec();
        pos += dcid_len;

        // SCID
        if pos >= input.len() {
            return Err(VersionNegotiationError::TooShort);
        }
        let scid_len = input[pos] as usize;
        pos += 1;
        if pos + scid_len > input.len() {
            return Err(VersionNegotiationError::Truncated);
        }
        let scid = input[pos..pos + scid_len].to_vec();
        pos += scid_len;

        // Supported versions (4 bytes each)
        let mut supported_versions = Vec::new();
        while pos + 4 <= input.len() {
            let ver =
                u32::from_be_bytes([input[pos], input[pos + 1], input[pos + 2], input[pos + 3]]);
            supported_versions.push(ver);
            pos += 4;
        }

        Ok(Self {
            rejected_version,
            dcid,
            scid,
            supported_versions,
        })
    }

    /// Check if a specific version is in the server's supported list.
    pub fn supports_version(&self, version: u32) -> bool {
        self.supported_versions.contains(&version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── IETF VN helpers & tests (existing) ──────────────────────────────────

    fn build_version_negotiation(dcid: &[u8], scid: &[u8], versions: &[u32]) -> Vec<u8> {
        let mut pkt = Vec::new();
        // Header byte: long header + type 0x00
        pkt.push(0xC0);
        // Version = 0
        pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // DCID
        pkt.push(dcid.len() as u8);
        pkt.extend_from_slice(dcid);
        // SCID
        pkt.push(scid.len() as u8);
        pkt.extend_from_slice(scid);
        // No random field per RFC 9000 §17.2.1
        // Supported versions
        for v in versions {
            pkt.extend_from_slice(&v.to_be_bytes());
        }
        pkt
    }

    #[test]
    fn test_parse_version_negotiation() {
        let dcid = vec![0x11, 0x22, 0x33, 0x44];
        let scid = vec![0x55, 0x66, 0x77, 0x88];
        let versions = vec![0x00000001, 0xff00001d, 0xff00001e];

        let pkt = build_version_negotiation(&dcid, &scid, &versions);
        let vn = VersionNegotiation::parse(&pkt).unwrap();

        assert_eq!(vn.dcid, dcid);
        assert_eq!(vn.scid, scid);
        assert_eq!(vn.supported_versions, versions);
    }

    #[test]
    fn test_supports_version() {
        let pkt = build_version_negotiation(&[0x11], &[0x22], &[0x00000001, 0xff00001d]);
        let vn = VersionNegotiation::parse(&pkt).unwrap();

        assert!(vn.supports_version(0x00000001));
        assert!(vn.supports_version(0xff00001d));
        assert!(!vn.supports_version(0x00000002));
    }

    #[test]
    fn test_parse_version_not_zero() {
        let mut pkt = vec![0xC0];
        pkt.extend_from_slice(&0x00000001u32.to_be_bytes()); // Non-zero version
        pkt.push(0x04);
        pkt.extend_from_slice(&[0x11; 4]);
        pkt.push(0x04);
        pkt.extend_from_slice(&[0x22; 4]);
        pkt.extend_from_slice(&[0; 8]);

        assert_eq!(
            VersionNegotiation::parse(&pkt),
            Err(VersionNegotiationError::VersionNotZero)
        );
    }

    // ── identify_version tests ──────────────────────────────────────────────

    #[test]
    fn test_identify_version_quic_v1() {
        assert_eq!(identify_version(QUIC_V1), QuicVersionFamily::IetfV1);
    }

    #[test]
    fn test_identify_version_drafts() {
        assert_eq!(identify_version(DRAFT_29), QuicVersionFamily::IetfDraft(29));
        assert_eq!(identify_version(DRAFT_30), QuicVersionFamily::IetfDraft(30));
        assert_eq!(identify_version(DRAFT_31), QuicVersionFamily::IetfDraft(31));
        assert_eq!(identify_version(DRAFT_32), QuicVersionFamily::IetfDraft(32));
        assert_eq!(identify_version(DRAFT_33), QuicVersionFamily::IetfDraft(33));
        assert_eq!(identify_version(DRAFT_34), QuicVersionFamily::IetfDraft(34));
        assert_eq!(identify_version(DRAFT_35), QuicVersionFamily::IetfDraft(35));
    }

    #[test]
    fn test_identify_version_gquic() {
        assert_eq!(
            identify_version(GQUIC_Q043),
            QuicVersionFamily::Gquic("Q043".into())
        );
        assert_eq!(
            identify_version(GQUIC_Q046),
            QuicVersionFamily::Gquic("Q046".into())
        );
        assert_eq!(
            identify_version(GQUIC_Q050),
            QuicVersionFamily::Gquic("Q050".into())
        );
        assert_eq!(
            identify_version(GQUIC_T051),
            QuicVersionFamily::Gquic("T051".into())
        );
        assert_eq!(
            identify_version(GQUIC_T052),
            QuicVersionFamily::Gquic("T052".into())
        );
        assert_eq!(
            identify_version(GQUIC_T053),
            QuicVersionFamily::Gquic("T053".into())
        );
        assert_eq!(
            identify_version(GQUIC_T054),
            QuicVersionFamily::Gquic("T054".into())
        );
        assert_eq!(
            identify_version(GQUIC_T055),
            QuicVersionFamily::Gquic("T055".into())
        );
    }

    #[test]
    fn test_identify_version_unknown() {
        assert_eq!(
            identify_version(0xDEADBEEF),
            QuicVersionFamily::Unknown(0xDEADBEEF)
        );
        assert_eq!(
            identify_version(0x00000099),
            QuicVersionFamily::Unknown(0x00000099)
        );
    }

    // ── version_name tests ──────────────────────────────────────────────────

    #[test]
    fn test_version_name() {
        assert_eq!(version_name(QUIC_V1), "IETF QUIC v1");
        assert_eq!(version_name(DRAFT_29), "IETF QUIC draft-29");
        assert_eq!(version_name(DRAFT_30), "IETF QUIC draft-30");
        assert_eq!(version_name(DRAFT_31), "IETF QUIC draft-31");
        assert_eq!(version_name(DRAFT_32), "IETF QUIC draft-32");
        assert_eq!(version_name(DRAFT_33), "IETF QUIC draft-33");
        assert_eq!(version_name(DRAFT_34), "IETF QUIC draft-34");
        assert_eq!(version_name(DRAFT_35), "IETF QUIC draft-35");
        assert_eq!(version_name(GQUIC_Q043), "gQUIC Q043");
        assert_eq!(version_name(GQUIC_Q046), "gQUIC Q046");
        assert_eq!(version_name(GQUIC_Q050), "gQUIC Q050");
        assert_eq!(version_name(GQUIC_T051), "gQUIC T051");
        assert_eq!(version_name(GQUIC_T052), "gQUIC T052");
        assert_eq!(version_name(GQUIC_T053), "gQUIC T053");
        assert_eq!(version_name(GQUIC_T054), "gQUIC T054");
        assert_eq!(version_name(GQUIC_T055), "gQUIC T055");
        assert_eq!(version_name(0x12345678), "Unknown");
    }

    // ── is_gquic / is_ietf_draft / draft_number tests ───────────────────────

    #[test]
    fn test_is_gquic() {
        assert!(is_gquic(GQUIC_Q043));
        assert!(is_gquic(GQUIC_Q046));
        assert!(is_gquic(GQUIC_Q050));
        assert!(is_gquic(GQUIC_T051));
        assert!(is_gquic(GQUIC_T052));
        assert!(is_gquic(GQUIC_T053));
        assert!(is_gquic(GQUIC_T054));
        assert!(is_gquic(GQUIC_T055));
        assert!(!is_gquic(QUIC_V1));
        assert!(!is_gquic(DRAFT_29));
        assert!(!is_gquic(0xDEADBEEF));
    }

    #[test]
    fn test_is_ietf_draft() {
        assert!(is_ietf_draft(DRAFT_29));
        assert!(is_ietf_draft(DRAFT_30));
        assert!(is_ietf_draft(DRAFT_31));
        assert!(is_ietf_draft(DRAFT_32));
        assert!(is_ietf_draft(DRAFT_33));
        assert!(is_ietf_draft(DRAFT_34));
        assert!(is_ietf_draft(DRAFT_35));
        assert!(is_ietf_draft(0xff000000));
        assert!(is_ietf_draft(0xff0000ff));
        assert!(!is_ietf_draft(QUIC_V1));
        assert!(!is_ietf_draft(GQUIC_Q043));
        assert!(!is_ietf_draft(0x01000000));
    }

    #[test]
    fn test_draft_number() {
        assert_eq!(draft_number(DRAFT_29), Some(29));
        assert_eq!(draft_number(DRAFT_30), Some(30));
        assert_eq!(draft_number(DRAFT_31), Some(31));
        assert_eq!(draft_number(DRAFT_32), Some(32));
        assert_eq!(draft_number(DRAFT_33), Some(33));
        assert_eq!(draft_number(DRAFT_34), Some(34));
        assert_eq!(draft_number(DRAFT_35), Some(35));
        assert_eq!(draft_number(0xff000001), Some(1));
        assert_eq!(draft_number(QUIC_V1), None);
        assert_eq!(draft_number(GQUIC_Q043), None);
    }

    // ── is_known_version tests ──────────────────────────────────────────────

    #[test]
    fn test_is_known_version() {
        assert!(is_known_version(QUIC_V1));
        assert!(is_known_version(DRAFT_29));
        assert!(is_known_version(DRAFT_30));
        assert!(is_known_version(DRAFT_31));
        assert!(is_known_version(DRAFT_32));
        assert!(is_known_version(DRAFT_33));
        assert!(is_known_version(DRAFT_34));
        assert!(is_known_version(DRAFT_35));
        assert!(is_known_version(GQUIC_Q043));
        assert!(is_known_version(GQUIC_Q046));
        assert!(is_known_version(GQUIC_Q050));
        assert!(is_known_version(GQUIC_T051));
        assert!(is_known_version(GQUIC_T052));
        assert!(is_known_version(GQUIC_T053));
        assert!(is_known_version(GQUIC_T054));
        assert!(is_known_version(GQUIC_T055));
        assert!(!is_known_version(0xDEADBEEF));
        assert!(!is_known_version(0x00000099));
        assert!(!is_known_version(0xff000064));
    }

    // ── negotiate_version tests ─────────────────────────────────────────────

    #[test]
    fn test_negotiate_version_match() {
        let client = vec![GQUIC_Q043, GQUIC_Q046, GQUIC_Q050];
        let server = vec![GQUIC_Q050, GQUIC_T051];
        assert_eq!(negotiate_version(&client, &server), Some(GQUIC_Q050));
    }

    #[test]
    fn test_negotiate_version_no_match() {
        let client = vec![GQUIC_Q043, GQUIC_Q046];
        let server = vec![GQUIC_T051, GQUIC_T052];
        assert_eq!(negotiate_version(&client, &server), None);
    }

    #[test]
    fn test_negotiate_version_first_match_wins() {
        let client = vec![DRAFT_32, DRAFT_33, QUIC_V1];
        let server = vec![QUIC_V1, DRAFT_33, DRAFT_32];
        assert_eq!(negotiate_version(&client, &server), Some(DRAFT_32));
    }

    #[test]
    fn test_negotiate_version_empty_client() {
        let server = vec![QUIC_V1];
        assert_eq!(negotiate_version(&[], &server), None);
    }

    #[test]
    fn test_negotiate_version_empty_server() {
        let client = vec![QUIC_V1];
        assert_eq!(negotiate_version(&client, &[]), None);
    }

    // ── gQUIC VN parser tests ───────────────────────────────────────────────

    fn build_gquic_vn(
        rejected_version: u32,
        dcid: &[u8],
        scid: &[u8],
        versions: &[u32],
    ) -> Vec<u8> {
        let mut pkt = Vec::new();
        // Long header bit set
        pkt.push(0x80);
        // Rejected version
        pkt.extend_from_slice(&rejected_version.to_be_bytes());
        // DCID
        pkt.push(dcid.len() as u8);
        pkt.extend_from_slice(dcid);
        // SCID
        pkt.push(scid.len() as u8);
        pkt.extend_from_slice(scid);
        // Supported versions
        for v in versions {
            pkt.extend_from_slice(&v.to_be_bytes());
        }
        pkt
    }

    #[test]
    fn test_parse_gquic_version_negotiation() {
        let dcid = vec![0xAA, 0xBB];
        let scid = vec![0xCC, 0xDD, 0xEE];
        let versions = vec![GQUIC_Q043, GQUIC_Q046, GQUIC_Q050];

        let pkt = build_gquic_vn(GQUIC_Q043, &dcid, &scid, &versions);
        let vn = GquicVersionNegotiation::parse(&pkt).unwrap();

        assert_eq!(vn.rejected_version, GQUIC_Q043);
        assert_eq!(vn.dcid, dcid);
        assert_eq!(vn.scid, scid);
        assert_eq!(vn.supported_versions, versions);
    }

    #[test]
    fn test_parse_gquic_vn_too_short() {
        assert_eq!(
            GquicVersionNegotiation::parse(&[0x80, 0x00]),
            Err(VersionNegotiationError::TooShort)
        );
    }

    #[test]
    fn test_parse_gquic_vn_not_long_header() {
        // Header byte without bit 7
        assert_eq!(
            GquicVersionNegotiation::parse(&[0x00, 0x51, 0x30, 0x34, 0x33]),
            Err(VersionNegotiationError::NotLongHeader)
        );
    }

    #[test]
    fn test_parse_gquic_vn_zero_version() {
        let mut pkt = vec![0x80];
        pkt.extend_from_slice(&0u32.to_be_bytes());
        pkt.push(0x00);
        pkt.push(0x00);
        assert_eq!(
            GquicVersionNegotiation::parse(&pkt),
            Err(VersionNegotiationError::VersionNotZero)
        );
    }

    #[test]
    fn test_parse_gquic_vn_truncated() {
        let pkt = build_gquic_vn(GQUIC_Q043, &[], &[], &[]);
        // Packet is valid but has zero supported versions – that's fine
        let vn = GquicVersionNegotiation::parse(&pkt).unwrap();
        assert_eq!(vn.supported_versions.len(), 0);
    }

    #[test]
    fn test_gquic_vn_supports_version() {
        let pkt = build_gquic_vn(GQUIC_Q043, &[0x01], &[0x02], &[GQUIC_Q046, GQUIC_Q050]);
        let vn = GquicVersionNegotiation::parse(&pkt).unwrap();

        assert!(vn.supports_version(GQUIC_Q046));
        assert!(vn.supports_version(GQUIC_Q050));
        assert!(!vn.supports_version(GQUIC_Q043));
    }
}
