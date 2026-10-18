//! TLS record layer parsing — zero-copy extraction of TLS record headers.
//!
//! Parses the outer TLS record layer (RFC 8446 §5.1) to extract content type,
//! protocol version, and payload length from raw bytes.

/// TLS content type values (RFC 8446 §4.2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
    // 24–255: reserved for future use
}

impl ContentType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            20 => Some(Self::ChangeCipherSpec),
            21 => Some(Self::Alert),
            22 => Some(Self::Handshake),
            23 => Some(Self::ApplicationData),
            _ => None,
        }
    }
}

/// TLS protocol version (RFC 8446 §4.2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ProtocolVersion {
    Tls10 = 0x0301,
    Tls11 = 0x0302,
    Tls12 = 0x0303,
    Tls13 = 0x0304,
}

impl ProtocolVersion {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x0301 => Some(Self::Tls10),
            0x0302 => Some(Self::Tls11),
            0x0303 => Some(Self::Tls12),
            0x0304 => Some(Self::Tls13),
            _ => None,
        }
    }

    /// Return 2-character JA4 version string (e.g., "13" for TLS 1.3).
    pub fn ja4_version(&self) -> &'static str {
        match self {
            Self::Tls10 => "10",
            Self::Tls11 => "11",
            Self::Tls12 => "12",
            Self::Tls13 => "13",
        }
    }
}

/// Parsed TLS record header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TlsRecord<'a> {
    pub content_type: ContentType,
    pub version: ProtocolVersion,
    pub payload: &'a [u8],
}

/// Parse a TLS record from raw bytes.
///
/// Input should start at the TLS record header (content_type byte).
/// Returns the parsed record header and a slice pointing to the payload.
///
/// # Record Layout (RFC 8446 §5.1)
/// ```text
/// +--------+--------+--------+--------+--------+
/// | type   | version(2)      | length(2)        |
/// +--------+--------+--------+--------+--------+
/// | fragment (length bytes)                     |
/// +--------+--------+--------+--------+--------+
/// ```
#[inline]
pub fn parse_tls_record(input: &[u8]) -> Result<TlsRecord<'_>, TlsRecordError> {
    if input.len() < 5 {
        return Err(TlsRecordError::TooShort);
    }

    let content_type = ContentType::from_u8(input[0]).ok_or(TlsRecordError::InvalidContentType)?;
    let version = ProtocolVersion::from_u16(u16::from_be_bytes([input[1], input[2]]))
        .ok_or(TlsRecordError::InvalidVersion)?;
    let length = u16::from_be_bytes([input[3], input[4]]) as usize;

    if input.len() < 5 + length {
        return Err(TlsRecordError::TruncatedPayload { expected: length });
    }

    Ok(TlsRecord {
        content_type,
        version,
        payload: &input[5..5 + length],
    })
}

/// Error type for TLS record parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TlsRecordError {
    TooShort,
    InvalidContentType,
    InvalidVersion,
    TruncatedPayload { expected: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tls13_client_hello_record() {
        // Minimal TLS 1.3 Client Hello record header:
        // content_type=22 (Handshake), version=0x0301, length=0x0000
        // Note: TLS 1.3 uses legacy_version=0x0301 in the record layer
        let record = [0x16, 0x03, 0x01, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
        let parsed = parse_tls_record(&record).unwrap();
        assert_eq!(parsed.content_type, ContentType::Handshake);
        assert_eq!(parsed.version, ProtocolVersion::Tls10); // Legacy version
        assert_eq!(parsed.payload.len(), 5);
    }

    #[test]
    fn test_parse_too_short() {
        let record = [0x16, 0x03, 0x01];
        assert_eq!(parse_tls_record(&record), Err(TlsRecordError::TooShort));
    }

    #[test]
    fn test_parse_invalid_content_type() {
        let record = [0x19, 0x03, 0x01, 0x00, 0x00];
        assert_eq!(
            parse_tls_record(&record),
            Err(TlsRecordError::InvalidContentType)
        );
    }
}
