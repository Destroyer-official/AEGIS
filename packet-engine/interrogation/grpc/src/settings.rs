//! HTTP/2 SETTINGS frame construction (RFC 9113 §6.5).
//!
//! After the connection preface, the client sends a SETTINGS frame
//! advertising its transport parameters. For gRPC, this includes
//! appropriate initial window size and maximum concurrent streams.

/// HTTP/2 frame types.
pub const FRAME_TYPE_SETTINGS: u8 = 0x04;

/// HTTP/2 SETTINGS identifiers (RFC 9113 §6.5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum SettingId {
    /// SETTINGS_HEADER_TABLE_SIZE (0x1)
    HeaderTableSize = 0x0001,
    /// SETTINGS_ENABLE_PUSH (0x2)
    EnablePush = 0x0002,
    /// SETTINGS_MAX_CONCURRENT_STREAMS (0x3)
    MaxConcurrentStreams = 0x0003,
    /// SETTINGS_INITIAL_WINDOW_SIZE (0x4)
    InitialWindowSize = 0x0004,
    /// SETTINGS_MAX_FRAME_SIZE (0x5)
    MaxFrameSize = 0x0005,
    /// SETTINGS_MAX_HEADER_LIST_SIZE (0x6)
    MaxHeaderListSize = 0x0006,
}

impl SettingId {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x0001 => Some(Self::HeaderTableSize),
            0x0002 => Some(Self::EnablePush),
            0x0003 => Some(Self::MaxConcurrentStreams),
            0x0004 => Some(Self::InitialWindowSize),
            0x0005 => Some(Self::MaxFrameSize),
            0x0006 => Some(Self::MaxHeaderListSize),
            _ => None,
        }
    }
}

/// A single SETTINGS parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setting {
    pub id: u16,
    pub value: u32,
}

/// HTTP/2 SETTINGS frame builder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsFrame {
    pub settings: Vec<Setting>,
}

impl SettingsFrame {
    /// Create a default SETTINGS frame suitable for gRPC connections.
    ///
    /// Default values:
    /// - ENABLE_PUSH: 0 (no server push for gRPC)
    /// - MAX_CONCURRENT_STREAMS: 100
    /// - INITIAL_WINDOW_SIZE: 1048576 (1 MB)
    /// - MAX_FRAME_SIZE: 16384 (16 KB, default)
    /// - MAX_HEADER_LIST_SIZE: 65536 (64 KB)
    pub fn grpc_default() -> Self {
        Self {
            settings: vec![
                Setting {
                    id: 0x0002,
                    value: 0,
                }, // ENABLE_PUSH = 0
                Setting {
                    id: 0x0003,
                    value: 100,
                }, // MAX_CONCURRENT_STREAMS
                Setting {
                    id: 0x0004,
                    value: 1048576,
                }, // INITIAL_WINDOW_SIZE = 1MB
                Setting {
                    id: 0x0005,
                    value: 16384,
                }, // MAX_FRAME_SIZE
                Setting {
                    id: 0x0006,
                    value: 65536,
                }, // MAX_HEADER_LIST_SIZE
            ],
        }
    }

    /// Create an empty SETTINGS frame (acknowledgment).
    pub fn ack() -> Self {
        Self { settings: vec![] }
    }

    /// Serialize the SETTINGS frame into bytes.
    ///
    /// Frame layout (RFC 9113 §4.1):
    /// ```text
    /// +-----------------------------------------------+
    /// |                 Length (24)                   |
    /// +---------------+---------------+---------------+
    /// |   Type (8)    |   Flags (8)   |
    /// +-+-------------+---------------+------...-----+
    /// |R|                 Stream Identifier (31)     |
    /// +-+--------------------------------------------+
    /// |                   Frame Body                 |
    /// +-----------------------------------------------+
    /// ```
    pub fn serialize(&self) -> Vec<u8> {
        let body_len = self.settings.len() * 6; // 2-byte id + 4-byte value each
        let mut frame = Vec::with_capacity(9 + body_len);

        // Length (24 bits)
        frame.push((body_len >> 16) as u8);
        frame.push((body_len >> 8) as u8);
        frame.push(body_len as u8);

        // Type (SETTINGS = 0x04)
        frame.push(FRAME_TYPE_SETTINGS);

        // Flags: ACK (0x01) if settings list is empty, else 0x00
        // RFC 9113 §6.5: A SETTINGS frame with empty body is an ACK
        if self.settings.is_empty() {
            frame.push(0x01); // ACK flag
        } else {
            frame.push(0x00);
        }

        // Stream Identifier (0 for connection-level)
        frame.push(0x00);
        frame.push(0x00);
        frame.push(0x00);
        frame.push(0x00);

        // Settings payload
        for setting in &self.settings {
            frame.push((setting.id >> 8) as u8);
            frame.push(setting.id as u8);
            frame.push((setting.value >> 24) as u8);
            frame.push((setting.value >> 16) as u8);
            frame.push((setting.value >> 8) as u8);
            frame.push(setting.value as u8);
        }

        frame
    }

    /// Parse a SETTINGS frame from the frame body (after the 9-byte header).
    pub fn parse_body(body: &[u8]) -> Result<Self, SettingsParseError> {
        if !body.len().is_multiple_of(6) {
            return Err(SettingsParseError::InvalidBodyLength);
        }

        let mut settings = Vec::new();
        for chunk in body.as_chunks::<6>().0 {
            let id = u16::from_be_bytes([chunk[0], chunk[1]]);
            let value = u32::from_be_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
            settings.push(Setting { id, value });
        }

        Ok(Self { settings })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsParseError {
    InvalidBodyLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_default_settings() {
        let sf = SettingsFrame::grpc_default();
        let frame = sf.serialize();
        // 9-byte header + 5 settings * 6 bytes = 39 bytes
        assert_eq!(frame.len(), 39);
        // Type = 0x04
        assert_eq!(frame[3], FRAME_TYPE_SETTINGS);
        // Stream ID = 0
        assert_eq!(&frame[5..9], &[0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_settings_roundtrip() {
        let sf = SettingsFrame::grpc_default();
        let frame = sf.serialize();
        let body = &frame[9..];
        let parsed = SettingsFrame::parse_body(body).unwrap();
        assert_eq!(parsed.settings.len(), 5);
        assert_eq!(parsed.settings[0].id, 0x0002);
        assert_eq!(parsed.settings[0].value, 0);
        assert_eq!(parsed.settings[2].id, 0x0004);
        assert_eq!(parsed.settings[2].value, 1048576);
    }

    #[test]
    fn test_settings_ack() {
        let sf = SettingsFrame::ack();
        let frame = sf.serialize();
        // 9-byte header only (empty body)
        assert_eq!(frame.len(), 9);
    }

    #[test]
    fn test_parse_empty_body() {
        let parsed = SettingsFrame::parse_body(&[]).unwrap();
        assert!(parsed.settings.is_empty());
    }

    #[test]
    fn test_parse_invalid_body_length() {
        let body = [0x00, 0x01, 0x00, 0x00, 0x00]; // 5 bytes (not multiple of 6)
        assert_eq!(
            SettingsFrame::parse_body(&body),
            Err(SettingsParseError::InvalidBodyLength)
        );
    }
}
