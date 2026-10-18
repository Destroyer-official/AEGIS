//! QUIC Initial packet construction (RFC 9000 §17.2.2).
//!
//! Interrogators craft QUIC Initial packets with:
//! - Version field set to the supported QUIC version
//! - Short connection IDs (1–4 bytes)
//! - Frame types: CRYPTO (for TLS handshake), PADDING
//!
//! The Initial packet is the first packet sent by a client to establish
//! a QUIC connection. It contains the TLS Client Hello in CRYPTO frames.

/// QUIC version numbers (RFC 9000 §15).
pub mod quic_version {
    /// QUIC version 1 (RFC 9000)
    pub const QUIC_V1: u32 = 0x00000001;

    /// Check if a version number is a supported QUIC version.
    pub fn is_supported(version: u32) -> bool {
        version == QUIC_V1
    }

    /// Check if a version is a reserved version for version negotiation testing.
    pub fn is_reserved(version: u32) -> bool {
        (version & 0x0a0a0a0a) == 0x0a0a0a0a
    }
}

/// QUIC packet type identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuicPacketType {
    Initial,
    ZeroRtt,
    Handshake,
    Retry,
    VersionNegotiation, // Special: long header with version=0
}

impl QuicPacketType {
    /// First byte for long header packets (RFC 9000 §17.2).
    /// Layout: 1|1|PacketType(2)|TypeSpecificBits(4)
    pub fn header_byte(&self) -> u8 {
        match self {
            Self::Initial => 0xC0,   // type bits = 00
            Self::ZeroRtt => 0xD0,   // type bits = 01
            Self::Handshake => 0xE0, // type bits = 10
            Self::Retry => 0xF0,     // type bits = 11
            Self::VersionNegotiation => 0x80,
        }
    }
}

/// QUIC frame types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QuicFrameType {
    Padding = 0x00,
    Ping = 0x01,
    Crypto = 0x06,
    NewToken = 0x07,
    Stream = 0x08, // + variant bits
}

/// A constructed QUIC Initial packet.
#[derive(Clone, Debug)]
pub struct QuicInitialPacket {
    /// QUIC version (u32 big-endian)
    pub version: u32,
    /// Destination Connection ID (variable length, 0–20 bytes)
    pub dcid: Vec<u8>,
    /// Source Connection ID (variable length, 0–20 bytes)
    pub scid: Vec<u8>,
    /// Token (variable length, 0–20 bytes for Initial packets)
    pub token: Vec<u8>,
    /// CRYPTO frame payload (the TLS Client Hello bytes)
    pub crypto_payload: Vec<u8>,
}

impl QuicInitialPacket {
    /// Create a new QUIC Initial packet with default connection ID lengths.
    pub fn new(dcid_len: usize, scid_len: usize) -> Self {
        Self {
            version: quic_version::QUIC_V1,
            dcid: vec![0u8; dcid_len],
            scid: vec![0u8; scid_len],
            token: Vec::new(),
            crypto_payload: Vec::new(),
        }
    }

    /// Set the QUIC version.
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Set the token.
    pub fn with_token(mut self, token: Vec<u8>) -> Self {
        self.token = token;
        self
    }

    /// Set the CRYPTO frame payload (TLS Client Hello).
    pub fn with_crypto_payload(mut self, payload: Vec<u8>) -> Self {
        self.crypto_payload = payload;
        self
    }

    /// Set random connection IDs.
    pub fn with_random_ids(mut self, dcid: Vec<u8>, scid: Vec<u8>) -> Self {
        self.dcid = dcid;
        self.scid = scid;
        self
    }

    /// Serialize the Initial packet into bytes.
    ///
    /// Initial packet layout (RFC 9000 §17.2.2):
    /// ```text
    /// +-+-+-+-+-+-+-+-+
    /// |1|1|1|1|0|0|0|0|  Long header, type=Initial (0xC0)
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                         Version (32)                         |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// | DCID Len (8) |         Destination Connection ID (*)        |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// | SCID Len (8) |           Source Connection ID (*)           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                       Token Length (vint)                    |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                      Token (*) {#token}                      |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                      Length (vint)                           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                     Packet Payload (*)                       |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    pub fn serialize(&self) -> Vec<u8> {
        // Build payload: CRYPTO frame + PADDING
        let mut payload = Vec::new();

        // CRYPTO frame: type(1) + offset(varint) + length(varint) + data
        let crypto_len = self.crypto_payload.len();
        payload.push(QuicFrameType::Crypto as u8);

        // CRYPTO frame offset = 0 (single-byte varint: 0x00)
        payload.push(0x00);

        // CRYPTO frame length (varint encoding)
        if crypto_len < 64 {
            payload.push(crypto_len as u8);
        } else if crypto_len < 16384 {
            payload.push(0x40 | ((crypto_len >> 8) as u8));
            payload.push(crypto_len as u8);
        } else {
            // For larger payloads, use 4-byte varint
            payload.push(0x80 | ((crypto_len >> 24) as u8));
            payload.push((crypto_len >> 16) as u8);
            payload.push((crypto_len >> 8) as u8);
            payload.push(crypto_len as u8);
        }

        // CRYPTO frame data
        payload.extend_from_slice(&self.crypto_payload);

        // Add PADDING to meet minimum Initial packet size (1200 bytes recommended)
        let min_size: usize = 1200;
        // Header size (without packet length varint):
        // 1 (header byte) + 4 (version) + 1 + dcid_len + 1 + scid_len + 1 + token_len
        let base_header = 8 + self.dcid.len() + self.scid.len() + self.token.len();

        // First pass: assume 2-byte varint for length (covers payloads < 16384)
        // Total = base_header + 2 (varint) + payload_len >= 1200
        let target_payload = min_size.saturating_sub(base_header + 2);
        if payload.len() < target_payload {
            let pad_needed = target_payload - payload.len();
            payload.extend(std::iter::repeat_n(0x00, pad_needed));
        }

        // Calculate packet length (varint)
        let packet_length = payload.len();

        let mut packet = Vec::new();

        // Header byte: 0xC0 (Initial packet)
        packet.push(0xC0);

        // Version (4 bytes, big-endian)
        packet.extend_from_slice(&self.version.to_be_bytes());

        // DCID
        packet.push(self.dcid.len() as u8);
        packet.extend_from_slice(&self.dcid);

        // SCID
        packet.push(self.scid.len() as u8);
        packet.extend_from_slice(&self.scid);

        // Token length + token
        packet.push(self.token.len() as u8);
        packet.extend_from_slice(&self.token);

        // Packet length (varint)
        if packet_length < 64 {
            packet.push(packet_length as u8);
        } else if packet_length < 16384 {
            packet.push(0x40 | ((packet_length >> 8) as u8));
            packet.push(packet_length as u8);
        } else {
            packet.push(0x80 | ((packet_length >> 24) as u8));
            packet.push((packet_length >> 16) as u8);
            packet.push((packet_length >> 8) as u8);
            packet.push(packet_length as u8);
        }

        // Payload
        packet.extend_from_slice(&payload);

        packet
    }

    /// Parse a QUIC Initial packet from raw bytes.
    pub fn parse(input: &[u8]) -> Result<Self, QuicParseError> {
        if input.is_empty() {
            return Err(QuicParseError::TooShort);
        }

        // Check long header bit
        if input[0] & 0x80 == 0 {
            return Err(QuicParseError::NotLongHeader);
        }

        // Check packet type (bits 6-7 should be 0b11 for Initial)
        if input[0] & 0xC0 != 0xC0 {
            return Err(QuicParseError::NotInitial);
        }

        if input.len() < 5 {
            return Err(QuicParseError::TooShort);
        }

        let version = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);

        let mut pos = 5;

        // DCID
        if pos >= input.len() {
            return Err(QuicParseError::TooShort);
        }
        let dcid_len = input[pos] as usize;
        pos += 1;
        if pos + dcid_len > input.len() {
            return Err(QuicParseError::Truncated);
        }
        let dcid = input[pos..pos + dcid_len].to_vec();
        pos += dcid_len;

        // SCID
        if pos >= input.len() {
            return Err(QuicParseError::TooShort);
        }
        let scid_len = input[pos] as usize;
        pos += 1;
        if pos + scid_len > input.len() {
            return Err(QuicParseError::Truncated);
        }
        let scid = input[pos..pos + scid_len].to_vec();
        pos += scid_len;

        // Token length (varint)
        if pos >= input.len() {
            return Err(QuicParseError::TooShort);
        }
        let token_len = input[pos] as usize;
        pos += 1;
        if pos + token_len > input.len() {
            return Err(QuicParseError::Truncated);
        }
        let token = input[pos..pos + token_len].to_vec();

        // Packet length (varint) — skip for now
        // We just note the remainder is the payload

        Ok(Self {
            version,
            dcid,
            scid,
            token,
            crypto_payload: Vec::new(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuicParseError {
    TooShort,
    NotLongHeader,
    NotInitial,
    Truncated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_packet_structure() {
        let pkt = QuicInitialPacket::new(8, 8)
            .with_version(quic_version::QUIC_V1)
            .with_random_ids(vec![0xAA; 8], vec![0xBB; 8])
            .with_crypto_payload(vec![0x01, 0x02, 0x03, 0x04]);

        let wire = pkt.serialize();

        // Verify long header bit
        assert!(wire[0] & 0x80 != 0);
        // Verify Initial type
        assert_eq!(wire[0] & 0xC0, 0xC0);
        // Verify version
        assert_eq!(&wire[1..5], &quic_version::QUIC_V1.to_be_bytes());
        // Verify DCID length
        assert_eq!(wire[5], 8);
        // Verify DCID
        assert_eq!(&wire[6..14], &[0xAA; 8]);
        // Verify SCID length
        assert_eq!(wire[14], 8);
        // Verify SCID
        assert_eq!(&wire[15..23], &[0xBB; 8]);
    }

    #[test]
    fn test_initial_packet_minimum_size() {
        let pkt = QuicInitialPacket::new(8, 8).with_crypto_payload(vec![0x01; 100]);

        let wire = pkt.serialize();
        // Initial packets should be padded to at least 1200 bytes
        assert!(
            wire.len() >= 1200,
            "Initial packet too small: {}",
            wire.len()
        );
    }

    #[test]
    fn test_parse_roundtrip() {
        let pkt = QuicInitialPacket::new(4, 4)
            .with_random_ids(vec![0x11; 4], vec![0x22; 4])
            .with_crypto_payload(vec![0x01, 0x02, 0x03]);

        let wire = pkt.serialize();
        let parsed = QuicInitialPacket::parse(&wire).unwrap();
        assert_eq!(parsed.version, quic_version::QUIC_V1);
        assert_eq!(parsed.dcid, vec![0x11; 4]);
        assert_eq!(parsed.scid, vec![0x22; 4]);
    }
}
