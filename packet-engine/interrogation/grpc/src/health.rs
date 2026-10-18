//! gRPC Health Checking protocol (grpc.health.v1.Health/Check).
//!
//! Implements the standard gRPC health check probe for service discovery.
//! See: https://github.com/grpc/grpc/blob/master/doc/health-checking.md

use crate::frame::{decode_grpc_message, encode_grpc_message, DataFrame, HeadersFrame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckRequest {
    service: String,
}

impl HealthCheckRequest {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_owned(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let tag: u8 = (1 << 3) | 2;
        let len = self.service.len() as u8;
        let mut out = Vec::with_capacity(2 + self.service.len());
        out.push(tag);
        out.push(len);
        out.extend_from_slice(self.service.as_bytes());
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingStatus {
    Unknown = 0,
    Serving = 1,
    NotServing = 2,
    ServiceUnknown = 3,
}

impl ServingStatus {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(ServingStatus::Unknown),
            1 => Some(ServingStatus::Serving),
            2 => Some(ServingStatus::NotServing),
            3 => Some(ServingStatus::ServiceUnknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthParseError {
    TooShort,
    InvalidField,
    UnknownStatus(u32),
}

fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), HealthParseError> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = offset;
    loop {
        if pos >= data.len() {
            return Err(HealthParseError::TooShort);
        }
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(HealthParseError::InvalidField);
        }
    }
    Ok((result, pos))
}

fn skip_field(data: &[u8], offset: usize) -> Result<usize, HealthParseError> {
    if offset >= data.len() {
        return Err(HealthParseError::TooShort);
    }
    let tag = data[offset];
    let wire_type = tag & 0x07;
    match wire_type {
        0 => {
            let (_, end) = read_varint(data, offset + 1)?;
            Ok(end)
        }
        2 => {
            let (len, start) = read_varint(data, offset + 1)?;
            Ok(start + len as usize)
        }
        _ => Err(HealthParseError::InvalidField),
    }
}

impl HealthCheckResponse {
    pub fn decode(data: &[u8]) -> Result<Self, HealthParseError> {
        let mut offset = 0;
        let mut status: Option<u32> = None;
        while offset < data.len() {
            let tag = data[offset];
            let field_number = tag >> 3;
            let wire_type = tag & 0x07;
            if field_number == 1 && wire_type == 0 {
                let (val, next) = read_varint(data, offset + 1)?;
                status = Some(val as u32);
                offset = next;
            } else {
                offset = skip_field(data, offset)?;
            }
        }
        let status_val = status.ok_or(HealthParseError::InvalidField)?;
        ServingStatus::from_u32(status_val)
            .ok_or(HealthParseError::UnknownStatus(status_val))
            .map(|s| HealthCheckResponse { status: s })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckResponse {
    pub status: ServingStatus,
}

pub fn build_health_probe(service: &str) -> Vec<u8> {
    let mut headers = HeadersFrame::new(1);
    headers
        .add_header(":method", "POST")
        .add_header(":path", "/grpc.health.v1.Health/Check")
        .add_header(":scheme", "https")
        .add_header("content-type", "application/grpc")
        .add_header("te", "trailers");
    let header_bytes = headers.serialize();

    let req = HealthCheckRequest::new(service);
    let grpc_msg = encode_grpc_message(&req.encode());
    let mut data = DataFrame::new(1);
    data.set_payload(&grpc_msg);
    let data_bytes = data.serialize();

    let mut probe = Vec::with_capacity(header_bytes.len() + data_bytes.len());
    probe.extend_from_slice(&header_bytes);
    probe.extend_from_slice(&data_bytes);
    probe
}

pub fn parse_health_response(data: &[u8]) -> Result<ServingStatus, HealthParseError> {
    let mut offset = 0;
    while offset + 9 <= data.len() {
        let frame_type = data[offset + 3];
        let payload_len = ((data[offset] as usize) << 16)
            | ((data[offset + 1] as usize) << 8)
            | (data[offset + 2] as usize);
        if frame_type == 0x00 {
            let payload_start = offset + 9;
            let payload_end = payload_start + payload_len;
            if payload_end > data.len() {
                return Err(HealthParseError::TooShort);
            }
            let payload = &data[payload_start..payload_end];
            if payload.len() < 5 {
                return Err(HealthParseError::TooShort);
            }
            let msg = decode_grpc_message(payload).map_err(|_| HealthParseError::TooShort)?;
            let resp = HealthCheckResponse::decode(msg)?;
            return Ok(resp.status);
        }
        offset += 9 + payload_len;
    }
    Err(HealthParseError::TooShort)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_request_encode_short_service() {
        let req = HealthCheckRequest::new("");
        let enc = req.encode();
        assert_eq!(enc, [0x0A, 0x00]);
    }

    #[test]
    fn health_request_encode_with_name() {
        let req = HealthCheckRequest::new("grpc");
        let enc = req.encode();
        assert_eq!(enc, [0x0A, 0x04, b'g', b'r', b'p', b'c']);
    }

    #[test]
    fn health_request_tag_is_field1_wiretype2() {
        let req = HealthCheckRequest::new("x");
        let enc = req.encode();
        assert_eq!(enc[0], (1 << 3) | 2);
    }

    #[test]
    fn health_response_decode_serving() {
        let data = [0x08, 0x01];
        let resp = HealthCheckResponse::decode(&data).unwrap();
        assert_eq!(resp.status, ServingStatus::Serving);
    }

    #[test]
    fn health_response_decode_not_serving() {
        let data = [0x08, 0x02];
        let resp = HealthCheckResponse::decode(&data).unwrap();
        assert_eq!(resp.status, ServingStatus::NotServing);
    }

    #[test]
    fn health_response_decode_unknown() {
        let data = [0x08, 0x00];
        let resp = HealthCheckResponse::decode(&data).unwrap();
        assert_eq!(resp.status, ServingStatus::Unknown);
    }

    #[test]
    fn health_response_decode_service_unknown() {
        let data = [0x08, 0x03];
        let resp = HealthCheckResponse::decode(&data).unwrap();
        assert_eq!(resp.status, ServingStatus::ServiceUnknown);
    }

    #[test]
    fn health_response_decode_invalid_status() {
        let data = [0x08, 0x05];
        let r = HealthCheckResponse::decode(&data);
        assert!(matches!(r, Err(HealthParseError::UnknownStatus(5))));
    }

    #[test]
    fn health_response_decode_empty() {
        let r = HealthCheckResponse::decode(&[]);
        assert!(matches!(r, Err(HealthParseError::InvalidField)));
    }

    #[test]
    fn health_response_decode_too_short() {
        let r = HealthCheckResponse::decode(&[0x08]);
        assert!(matches!(r, Err(HealthParseError::TooShort)));
    }

    #[test]
    fn build_health_probe_has_two_frames() {
        let probe = build_health_probe("test.svc");
        assert!(probe.len() > 18);
        assert_eq!(probe[3], 0x01);
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let second_offset = 9 + first_len;
        assert_eq!(probe[second_offset + 3], 0x00);
        assert!(probe.len() > second_offset + 9);
    }

    #[test]
    fn build_health_probe_headers_frame_stream_id() {
        let probe = build_health_probe("svc");
        let id = ((probe[5] as u32) << 24)
            | ((probe[6] as u32) << 16)
            | ((probe[7] as u32) << 8)
            | (probe[8] as u32);
        assert_eq!(id, 1);
    }

    #[test]
    fn build_health_probe_data_frame_stream_id() {
        let probe = build_health_probe("svc");
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let off = 9 + first_len;
        let id = ((probe[off + 5] as u32) << 24)
            | ((probe[off + 6] as u32) << 16)
            | ((probe[off + 7] as u32) << 8)
            | (probe[off + 8] as u32);
        assert_eq!(id, 1);
    }

    #[test]
    fn build_health_probe_data_has_grpc_prefix() {
        let probe = build_health_probe("svc");
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let off = 9 + first_len + 9;
        assert_eq!(probe[off], 0x00);
    }

    #[test]
    fn build_health_probe_data_contains_request() {
        let probe = build_health_probe("grpc");
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let data_payload_off = 9 + first_len + 9 + 5;
        assert_eq!(probe[data_payload_off], 0x0A);
        assert_eq!(probe[data_payload_off + 1], 4);
        assert_eq!(&probe[data_payload_off + 2..data_payload_off + 6], b"grpc");
    }

    #[test]
    fn parse_health_response_roundtrip() {
        let resp = HealthCheckResponse {
            status: ServingStatus::Serving,
        };
        let encoded = [0x08, 0x01];
        let decoded = HealthCheckResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.status, resp.status);
    }

    #[test]
    fn health_request_new_service() {
        let req = HealthCheckRequest::new("my.service");
        assert_eq!(req.service, "my.service");
    }
}
