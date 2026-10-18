//! QUIC Transport Parameters parsing (RFC 9000 §18).
//!
//! Transport parameters are encoded as a sequence of parameter ID-value pairs
//! in the TLS handshake. They are sent unencrypted in Initial packets and
//! contain connection limits, flow control windows, and other configuration.
//!
//! Interrogators extract transport parameters from unencrypted Initial packets
//! to fingerprint QUIC server implementations.

/// QUIC transport parameter IDs (RFC 9000 §18.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum TransportParamId {
    /// 0x00: original_destination_connection_id (server-only)
    OriginalDestinationConnectionId = 0x00,
    /// 0x01: max_idle_timeout
    MaxIdleTimeout = 0x01,
    /// 0x02: stateless_reset_token (server-only)
    StatelessResetToken = 0x02,
    /// 0x03: max_udp_payload_size
    MaxUdpPayloadSize = 0x03,
    /// 0x04: initial_max_data
    InitialMaxData = 0x04,
    /// 0x05: initial_max_stream_data_bidi_local
    InitialMaxStreamDataBidiLocal = 0x05,
    /// 0x06: initial_max_stream_data_bidi_remote
    InitialMaxStreamDataBidiRemote = 0x06,
    /// 0x07: initial_max_stream_data_uni
    InitialMaxStreamDataUni = 0x07,
    /// 0x08: initial_max_streams_bidi
    InitialMaxStreamsBidi = 0x08,
    /// 0x09: initial_max_streams_uni
    InitialMaxStreamsUni = 0x09,
    /// 0x0a: ack_delay_exponent
    AckDelayExponent = 0x0a,
    /// 0x0b: max_ack_delay
    MaxAckDelay = 0x0b,
    /// 0x0c: disable_active_migration
    DisableActiveMigration = 0x0c,
    /// 0x0d: preferred_address (server-only)
    PreferredAddress = 0x0d,
    /// 0x0e: active_connection_id_limit
    ActiveConnectionIdLimit = 0x0e,
    /// 0x0f: initial_source_connection_id
    InitialSourceConnectionId = 0x0f,
    /// 0x10: retry_source_connection_id (server-only)
    RetrySourceConnectionId = 0x10,
    /// 0x20: max_datagram_frame_size (RFC 9221)
    MaxDatagramFrameSize = 0x20,
}

impl TransportParamId {
    pub fn from_u64(val: u64) -> Option<Self> {
        match val {
            0x00 => Some(Self::OriginalDestinationConnectionId),
            0x01 => Some(Self::MaxIdleTimeout),
            0x02 => Some(Self::StatelessResetToken),
            0x03 => Some(Self::MaxUdpPayloadSize),
            0x04 => Some(Self::InitialMaxData),
            0x05 => Some(Self::InitialMaxStreamDataBidiLocal),
            0x06 => Some(Self::InitialMaxStreamDataBidiRemote),
            0x07 => Some(Self::InitialMaxStreamDataUni),
            0x08 => Some(Self::InitialMaxStreamsBidi),
            0x09 => Some(Self::InitialMaxStreamsUni),
            0x0a => Some(Self::AckDelayExponent),
            0x0b => Some(Self::MaxAckDelay),
            0x0c => Some(Self::DisableActiveMigration),
            0x0d => Some(Self::PreferredAddress),
            0x0e => Some(Self::ActiveConnectionIdLimit),
            0x0f => Some(Self::InitialSourceConnectionId),
            0x10 => Some(Self::RetrySourceConnectionId),
            0x20 => Some(Self::MaxDatagramFrameSize),
            _ => None,
        }
    }
}

/// A single transport parameter.
#[derive(Clone, Debug)]
pub struct TransportParam {
    pub id: u64,
    pub value: Vec<u8>,
}

/// Parsed QUIC transport parameters.
#[derive(Clone, Debug)]
pub struct TransportParams {
    pub params: Vec<TransportParam>,
}

impl TransportParams {
    /// Parse transport parameters from a binary-encoded parameter block.
    ///
    /// Each parameter is: varint ID + varint length + value bytes
    pub fn parse(input: &[u8]) -> Result<Self, TransportParamError> {
        let mut params = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            // Read parameter ID (varint)
            let (id, new_pos) = decode_varint(input, pos)?;
            pos = new_pos;

            // Read parameter length (varint)
            let (len, new_pos) = decode_varint(input, pos)?;
            pos = new_pos;

            // Read parameter value
            if pos + len as usize > input.len() {
                return Err(TransportParamError::Truncated);
            }
            let value = input[pos..pos + len as usize].to_vec();
            pos += len as usize;

            params.push(TransportParam { id, value });
        }

        Ok(Self { params })
    }

    /// Get a transport parameter by ID.
    pub fn get(&self, id: TransportParamId) -> Option<&TransportParam> {
        self.params.iter().find(|p| p.id == id as u64)
    }

    /// Get initial_max_data (flow control).
    pub fn initial_max_data(&self) -> Option<u64> {
        self.get(TransportParamId::InitialMaxData)
            .and_then(|p| decode_varint_value(&p.value))
    }

    /// Get initial_max_streams_bidi.
    pub fn initial_max_streams_bidi(&self) -> Option<u64> {
        self.get(TransportParamId::InitialMaxStreamsBidi)
            .and_then(|p| decode_varint_value(&p.value))
    }

    /// Get initial_max_streams_uni.
    pub fn initial_max_streams_uni(&self) -> Option<u64> {
        self.get(TransportParamId::InitialMaxStreamsUni)
            .and_then(|p| decode_varint_value(&p.value))
    }

    /// Get max_idle_timeout.
    pub fn max_idle_timeout(&self) -> Option<u64> {
        self.get(TransportParamId::MaxIdleTimeout)
            .and_then(|p| decode_varint_value(&p.value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportParamError {
    Truncated,
    InvalidVarint,
}

/// Decode a QUIC variable-length integer (RFC 9000 §16).
///
/// Layout:
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                 Variable-Length Integer Encoding                |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
fn decode_varint(input: &[u8], pos: usize) -> Result<(u64, usize), TransportParamError> {
    if pos >= input.len() {
        return Err(TransportParamError::InvalidVarint);
    }

    let prefix = input[pos] >> 6;
    match prefix {
        0 => Ok((input[pos] as u64 & 0x3F, pos + 1)),
        1 => {
            if pos + 2 > input.len() {
                return Err(TransportParamError::InvalidVarint);
            }
            let val = ((input[pos] as u64 & 0x3F) << 8) | input[pos + 1] as u64;
            Ok((val, pos + 2))
        }
        2 => {
            if pos + 4 > input.len() {
                return Err(TransportParamError::InvalidVarint);
            }
            let val = ((input[pos] as u64 & 0x3F) << 24)
                | (input[pos + 1] as u64) << 16
                | (input[pos + 2] as u64) << 8
                | input[pos + 3] as u64;
            Ok((val, pos + 4))
        }
        3 => {
            if pos + 8 > input.len() {
                return Err(TransportParamError::InvalidVarint);
            }
            let val = ((input[pos] as u64 & 0x3F) << 56)
                | (input[pos + 1] as u64) << 48
                | (input[pos + 2] as u64) << 40
                | (input[pos + 3] as u64) << 32
                | (input[pos + 4] as u64) << 24
                | (input[pos + 5] as u64) << 16
                | (input[pos + 6] as u64) << 8
                | input[pos + 7] as u64;
            Ok((val, pos + 8))
        }
        _ => unreachable!(),
    }
}

/// Decode a single varint from a value slice.
fn decode_varint_value(value: &[u8]) -> Option<u64> {
    let (val, _) = decode_varint(value, 0).ok()?;
    Some(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_decode_1byte() {
        let input = [0x25]; // prefix=0, value=37
        let (val, pos) = decode_varint(&input, 0).unwrap();
        assert_eq!(val, 37);
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_varint_decode_2byte() {
        let input = [0x7B, 0xBD]; // prefix=1, value=15293
        let (val, pos) = decode_varint(&input, 0).unwrap();
        assert_eq!(val, 15293);
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_parse_transport_params() {
        // Build a minimal transport parameter block:
        // initial_max_data (0x04) = 1048576 (varint-encoded)
        // QUIC varint for 1048576 (0x100000): 4-byte varint = [0x80, 0x10, 0x00, 0x00]
        let mut block = Vec::new();
        // ID = 0x04 (initial_max_data)
        block.push(0x04);
        // Length = 4 (value is 4 varint bytes)
        block.push(0x04);
        // Value = 1048576 in QUIC varint encoding
        block.extend_from_slice(&[0x80, 0x10, 0x00, 0x00]);

        // initial_max_streams_bidi (0x08) = 100 → QUIC varint encoding
        // 100 > 63, so 2-byte varint: prefix 01, value = 100 → [0x40, 0x64]
        block.push(0x08); // ID
        block.push(0x02); // Length = 2
        block.push(0x40); // varint high
        block.push(0x64); // varint low (100)

        let params = TransportParams::parse(&block).unwrap();
        assert_eq!(params.params.len(), 2);
        assert_eq!(params.initial_max_data(), Some(1048576));
        assert_eq!(params.initial_max_streams_bidi(), Some(100));
    }

    #[test]
    fn test_transport_params_empty() {
        let params = TransportParams::parse(&[]).unwrap();
        assert!(params.params.is_empty());
    }
}
