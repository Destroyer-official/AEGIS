//! GREASE (Generate Random Extensions And Sustain Extensibility) value detection.
//!
//! RFC 8701 defines GREASE values used to prevent TLS ossification.
//! These values must be excluded from JA4 fingerprint computation.

/// GREASE values per RFC 8701 — the 16 possible 2-byte values.
/// Pattern: 0x?a?a where the nibbles are equal and the low bit of the high byte is 1.
const GREASE_VALUES: [u16; 16] = [
    0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a, 0xaaaa, 0xbaba,
    0xcaca, 0xdada, 0xeaea, 0xfafa,
];

/// Check if a u16 value is a GREASE value.
#[inline]
pub fn is_grease(val: u16) -> bool {
    GREASE_VALUES.contains(&val)
}

/// Check if a single byte is a GREASE high byte (lower nibble is 0xa).
#[inline]
pub fn is_grease_byte(b: u8) -> bool {
    (b & 0x0f) == 0x0a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_grease_values() {
        for v in GREASE_VALUES {
            assert!(is_grease(v), "0x{:04x} should be GREASE", v);
        }
    }

    #[test]
    fn test_non_grease_values() {
        assert!(!is_grease(0x0000));
        assert!(!is_grease(0x1301)); // TLS_AES_128_GCM_SHA256
        assert!(!is_grease(0x00ff));
        assert!(!is_grease(0x1302));
    }

    #[test]
    fn test_grease_byte() {
        assert!(is_grease_byte(0x0a));
        assert!(is_grease_byte(0x1a));
        assert!(is_grease_byte(0xfa));
        assert!(!is_grease_byte(0x00));
        assert!(!is_grease_byte(0x13));
    }
}
