//! HTTP/2 Connection Preface (RFC 9113 §3.4).
//!
//! After a TLS handshake completes for gRPC, the client must send the
//! HTTP/2 connection preface followed by a SETTINGS frame before any
//! other HTTP/2 frames.
//!
//! The connection preface is a fixed 24-byte sequence:
//! ```text
//! PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n
//! ```

/// HTTP/2 connection preface constants.
pub struct Http2Preface;

impl Http2Preface {
    /// The 24-byte HTTP/2 connection preface.
    pub const PREFACE: [u8; 24] = *b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    /// Get the complete preface bytes.
    pub fn bytes() -> &'static [u8; 24] {
        &Self::PREFACE
    }

    /// Validate that the received bytes match the HTTP/2 connection preface.
    pub fn validate(data: &[u8]) -> bool {
        data.starts_with(&Self::PREFACE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preface_length() {
        assert_eq!(Http2Preface::PREFACE.len(), 24);
    }

    #[test]
    fn test_preface_content() {
        assert_eq!(
            &Http2Preface::PREFACE[..],
            b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
        );
    }

    #[test]
    fn test_validate_success() {
        let mut data = Vec::new();
        data.extend_from_slice(&Http2Preface::PREFACE);
        data.extend_from_slice(&[0x00, 0x00, 0x0c, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(Http2Preface::validate(&data));
    }

    #[test]
    fn test_validate_failure() {
        let data = b"PRI * HTTP/1.1\r\n\r\nSM\r\n\r\n";
        assert!(!Http2Preface::validate(data));
    }
}
