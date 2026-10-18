//! HTTP/2 frame construction for gRPC RPCs (RFC 9113 §4).

pub const FRAME_TYPE_DATA: u8 = 0x00;
pub const FRAME_TYPE_HEADERS: u8 = 0x01;
pub const FRAME_TYPE_SETTINGS: u8 = 0x04;
pub const FRAME_TYPE_GOAWAY: u8 = 0x07;
pub const FRAME_TYPE_WINDOW_UPDATE: u8 = 0x08;
pub const FRAME_TYPE_CONTINUATION: u8 = 0x09;

pub const GRPC_MESSAGE_PREFIX_LEN: usize = 5;

const FLAG_END_STREAM: u8 = 0x01;
const FLAG_END_HEADERS: u8 = 0x04;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadersFrame {
    stream_id: u32,
    headers: Vec<(String, String)>,
}

impl HeadersFrame {
    pub fn new(stream_id: u32) -> Self {
        Self {
            stream_id,
            headers: Vec::new(),
        }
    }

    pub fn add_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut hblock = Vec::new();
        for (name, value) in &self.headers {
            hblock.push(0x00);
            hblock.push(name.len() as u8);
            hblock.extend_from_slice(name.as_bytes());
            hblock.push(value.len() as u8);
            hblock.extend_from_slice(value.as_bytes());
        }
        let len = hblock.len() as u32;
        let mut frame = Vec::with_capacity(9 + hblock.len());
        frame.push(((len >> 16) & 0xFF) as u8);
        frame.push(((len >> 8) & 0xFF) as u8);
        frame.push((len & 0xFF) as u8);
        frame.push(FRAME_TYPE_HEADERS);
        frame.push(FLAG_END_HEADERS);
        frame.push(((self.stream_id >> 24) & 0x7F) as u8);
        frame.push(((self.stream_id >> 16) & 0xFF) as u8);
        frame.push(((self.stream_id >> 8) & 0xFF) as u8);
        frame.push((self.stream_id & 0xFF) as u8);
        frame.extend_from_slice(&hblock);
        frame
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFrame {
    stream_id: u32,
    payload: Vec<u8>,
}

impl DataFrame {
    pub fn new(stream_id: u32) -> Self {
        Self {
            stream_id,
            payload: Vec::new(),
        }
    }

    pub fn set_payload(&mut self, data: &[u8]) -> &mut Self {
        self.payload = data.to_vec();
        self
    }

    pub fn serialize(&self) -> Vec<u8> {
        let len = self.payload.len() as u32;
        let mut frame = Vec::with_capacity(9 + self.payload.len());
        frame.push(((len >> 16) & 0xFF) as u8);
        frame.push(((len >> 8) & 0xFF) as u8);
        frame.push((len & 0xFF) as u8);
        frame.push(FRAME_TYPE_DATA);
        frame.push(FLAG_END_STREAM);
        frame.push(((self.stream_id >> 24) & 0x7F) as u8);
        frame.push(((self.stream_id >> 16) & 0xFF) as u8);
        frame.push(((self.stream_id >> 8) & 0xFF) as u8);
        frame.push((self.stream_id & 0xFF) as u8);
        frame.extend_from_slice(&self.payload);
        frame
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrpcMessageError {
    TooShort,
    InvalidCompressionFlag,
    LengthMismatch,
}

pub fn encode_grpc_message(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(GRPC_MESSAGE_PREFIX_LEN + data.len());
    out.push(0x00);
    let len = data.len() as u32;
    out.push(((len >> 24) & 0xFF) as u8);
    out.push(((len >> 16) & 0xFF) as u8);
    out.push(((len >> 8) & 0xFF) as u8);
    out.push((len & 0xFF) as u8);
    out.extend_from_slice(data);
    out
}

pub fn decode_grpc_message(data: &[u8]) -> Result<&[u8], GrpcMessageError> {
    if data.len() < GRPC_MESSAGE_PREFIX_LEN {
        return Err(GrpcMessageError::TooShort);
    }
    if data[0] != 0x00 {
        return Err(GrpcMessageError::InvalidCompressionFlag);
    }
    let declared_len = ((data[1] as u32) << 24)
        | ((data[2] as u32) << 16)
        | ((data[3] as u32) << 8)
        | (data[4] as u32);
    let payload = &data[GRPC_MESSAGE_PREFIX_LEN..];
    if payload.len() < declared_len as usize {
        return Err(GrpcMessageError::LengthMismatch);
    }
    Ok(&payload[..declared_len as usize])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrailerFrame {
    stream_id: u32,
    trailers: Vec<(String, String)>,
}

impl TrailerFrame {
    pub fn new(stream_id: u32) -> Self {
        Self {
            stream_id,
            trailers: Vec::new(),
        }
    }

    pub fn add_trailer(&mut self, name: &str, value: &str) -> &mut Self {
        self.trailers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut hblock = Vec::new();
        for (name, value) in &self.trailers {
            hblock.push(0x00);
            hblock.push(name.len() as u8);
            hblock.extend_from_slice(name.as_bytes());
            hblock.push(value.len() as u8);
            hblock.extend_from_slice(value.as_bytes());
        }
        let len = hblock.len() as u32;
        let flags = FLAG_END_STREAM | FLAG_END_HEADERS;
        let mut frame = Vec::with_capacity(9 + hblock.len());
        frame.push(((len >> 16) & 0xFF) as u8);
        frame.push(((len >> 8) & 0xFF) as u8);
        frame.push((len & 0xFF) as u8);
        frame.push(FRAME_TYPE_HEADERS);
        frame.push(flags);
        frame.push(((self.stream_id >> 24) & 0x7F) as u8);
        frame.push(((self.stream_id >> 16) & 0xFF) as u8);
        frame.push(((self.stream_id >> 8) & 0xFF) as u8);
        frame.push((self.stream_id & 0xFF) as u8);
        frame.extend_from_slice(&hblock);
        frame
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrailerParseError {
    TooShort,
    InvalidFrameType,
    HpackError,
}

fn read_hpack_string(data: &[u8], offset: usize) -> Result<(&str, usize), TrailerParseError> {
    if offset >= data.len() {
        return Err(TrailerParseError::HpackError);
    }
    let len = data[offset] as usize;
    let start = offset + 1;
    if start + len > data.len() {
        return Err(TrailerParseError::HpackError);
    }
    let s = core::str::from_utf8(&data[start..start + len])
        .map_err(|_| TrailerParseError::HpackError)?;
    Ok((s, start + len))
}

pub fn parse_trailers(data: &[u8]) -> Result<Vec<(String, String)>, TrailerParseError> {
    if data.len() < 9 {
        return Err(TrailerParseError::TooShort);
    }
    let frame_type = data[3];
    if frame_type != FRAME_TYPE_HEADERS {
        return Err(TrailerParseError::InvalidFrameType);
    }
    let payload_len = ((data[0] as usize) << 16) | ((data[1] as usize) << 8) | (data[2] as usize);
    let payload = &data[9..];
    if payload.len() < payload_len {
        return Err(TrailerParseError::HpackError);
    }
    let mut offset = 0;
    let mut trailers = Vec::new();
    while offset < payload_len {
        if payload[offset] != 0x00 {
            offset += 1;
            continue;
        }
        offset += 1;
        let (name, off) = read_hpack_string(payload, offset)?;
        offset = off;
        let (value, off) = read_hpack_string(payload, offset)?;
        offset = off;
        trailers.push((name.to_owned(), value.to_owned()));
    }
    Ok(trailers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_frame_frame_type() {
        let f = HeadersFrame::new(1);
        let s = f.serialize();
        assert_eq!(s[3], FRAME_TYPE_HEADERS);
    }

    #[test]
    fn headers_frame_stream_id() {
        let f = HeadersFrame::new(5);
        let s = f.serialize();
        let id =
            ((s[5] as u32) << 24) | ((s[6] as u32) << 16) | ((s[7] as u32) << 8) | (s[8] as u32);
        assert_eq!(id, 5);
    }

    #[test]
    fn headers_frame_end_headers_flag() {
        let f = HeadersFrame::new(1);
        let s = f.serialize();
        assert_eq!(s[4], FLAG_END_HEADERS);
    }

    #[test]
    fn headers_frame_serialization_single_header() {
        let mut f = HeadersFrame::new(1);
        f.add_header(":method", "POST");
        let s = f.serialize();
        let payload_len = ((s[0] as usize) << 16) | ((s[1] as usize) << 8) | (s[2] as usize);
        assert_eq!(payload_len, 14);
        assert_eq!(s[3], FRAME_TYPE_HEADERS);
        assert_eq!(s[9], 0x00);
        assert_eq!(s[10], 7);
        assert_eq!(&s[11..18], b":method");
        assert_eq!(s[18], 4);
        assert_eq!(&s[19..23], b"POST");
    }

    #[test]
    fn data_frame_serialization() {
        let mut f = DataFrame::new(1);
        f.set_payload(b"hello");
        let s = f.serialize();
        assert_eq!(s[3], FRAME_TYPE_DATA);
        assert_eq!(s[4], FLAG_END_STREAM);
        assert_eq!(&s[9..], b"hello");
    }

    #[test]
    fn data_frame_empty_payload() {
        let f = DataFrame::new(3);
        let s = f.serialize();
        assert_eq!(s.len(), 9);
        assert_eq!(s[3], FRAME_TYPE_DATA);
        assert_eq!(s[4], FLAG_END_STREAM);
    }

    #[test]
    fn data_frame_stream_id() {
        let mut f = DataFrame::new(7);
        f.set_payload(b"x");
        let s = f.serialize();
        let id =
            ((s[5] as u32) << 24) | ((s[6] as u32) << 16) | ((s[7] as u32) << 8) | (s[8] as u32);
        assert_eq!(id, 7);
    }

    #[test]
    fn grpc_encode_decode_roundtrip() {
        let payload = b"gRPC payload data";
        let encoded = encode_grpc_message(payload);
        assert_eq!(encoded[0], 0x00);
        assert_eq!(encoded.len(), 5 + payload.len());
        let decoded = decode_grpc_message(&encoded).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn grpc_decode_too_short() {
        let r = decode_grpc_message(&[0x00, 0x00]);
        assert!(matches!(r, Err(GrpcMessageError::TooShort)));
    }

    #[test]
    fn grpc_decode_bad_compression_flag() {
        let r = decode_grpc_message(&[0x01, 0x00, 0x00, 0x00, 0x00]);
        assert!(matches!(r, Err(GrpcMessageError::InvalidCompressionFlag)));
    }

    #[test]
    fn grpc_decode_length_mismatch() {
        let r = decode_grpc_message(&[0x00, 0x00, 0x00, 0x00, 0x0A]);
        assert!(matches!(r, Err(GrpcMessageError::LengthMismatch)));
    }

    #[test]
    fn grpc_decode_length_mismatch_real() {
        let r = decode_grpc_message(&[0x00, 0x00, 0x00, 0x00, 0x05, 0x01]);
        assert!(matches!(r, Err(GrpcMessageError::LengthMismatch)));
    }

    #[test]
    fn trailer_frame_flags() {
        let mut t = TrailerFrame::new(1);
        t.add_trailer("grpc-status", "0");
        let s = t.serialize();
        assert_eq!(s[3], FRAME_TYPE_HEADERS);
        assert_eq!(s[4], FLAG_END_STREAM | FLAG_END_HEADERS);
    }

    #[test]
    fn trailer_frame_serialization() {
        let mut t = TrailerFrame::new(1);
        t.add_trailer("grpc-status", "0");
        let s = t.serialize();
        assert_eq!(s[9], 0x00);
        assert_eq!(s[10], 11);
        assert_eq!(&s[11..22], b"grpc-status");
        assert_eq!(s[22], 1);
        assert_eq!(s[23], b'0');
    }

    #[test]
    fn parse_trailers_roundtrip() {
        let mut t = TrailerFrame::new(1);
        t.add_trailer("grpc-status", "0");
        t.add_trailer("grpc-message", "OK");
        let s = t.serialize();
        let trailers = parse_trailers(&s).unwrap();
        assert_eq!(trailers.len(), 2);
        assert_eq!(trailers[0], ("grpc-status".to_owned(), "0".to_owned()));
        assert_eq!(trailers[1], ("grpc-message".to_owned(), "OK".to_owned()));
    }

    #[test]
    fn parse_trailers_too_short() {
        let r = parse_trailers(&[0x00; 5]);
        assert!(matches!(r, Err(TrailerParseError::TooShort)));
    }

    #[test]
    fn parse_trailers_wrong_frame_type() {
        let mut data = [0u8; 9];
        data[3] = FRAME_TYPE_DATA;
        let r = parse_trailers(&data);
        assert!(matches!(r, Err(TrailerParseError::InvalidFrameType)));
    }

    #[test]
    fn hpack_literal_encoding_correctness() {
        let mut f = HeadersFrame::new(1);
        f.add_header("content-type", "application/grpc");
        let s = f.serialize();
        let payload_len = ((s[0] as usize) << 16) | ((s[1] as usize) << 8) | (s[2] as usize);
        assert_eq!(payload_len, s.len() - 9);
        assert_eq!(s[9], 0x00);
        let name_len = s[10] as usize;
        assert_eq!(name_len, 12);
        assert_eq!(&s[11..23], b"content-type");
        let val_len = s[23] as usize;
        assert_eq!(val_len, 16);
        assert_eq!(&s[24..40], b"application/grpc");
    }
}
