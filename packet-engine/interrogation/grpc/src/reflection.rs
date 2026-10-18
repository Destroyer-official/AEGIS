//! gRPC Server Reflection protocol (grpc.reflection.v1alpha).
//!
//! Implements ServerReflectionInfo for service discovery via grpcurl.
//! See: https://github.com/grpc/grpc/blob/master/doc/server-reflection.md

use crate::frame::{encode_grpc_message, DataFrame, HeadersFrame};

pub fn encode_list_services() -> Vec<u8> {
    vec![0x2A, 0x00]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReflectionParseError {
    TooShort,
    InvalidField,
    ParseError,
}

fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), ReflectionParseError> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = offset;
    loop {
        if pos >= data.len() {
            return Err(ReflectionParseError::TooShort);
        }
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(ReflectionParseError::InvalidField);
        }
    }
    Ok((result, pos))
}

fn skip_field(data: &[u8], offset: usize) -> Result<usize, ReflectionParseError> {
    if offset >= data.len() {
        return Err(ReflectionParseError::TooShort);
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
        _ => Err(ReflectionParseError::InvalidField),
    }
}

pub fn decode_list_services(data: &[u8]) -> Result<Vec<String>, ReflectionParseError> {
    let mut services = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        if offset + 2 > data.len() {
            break;
        }
        let tag = data[offset];
        let field_number = tag >> 3;
        let wire_type = tag & 0x07;
        if field_number == 1 && wire_type == 2 {
            let (len, start) = read_varint(data, offset + 1)?;
            let field_end = start + len as usize;
            if field_end > data.len() {
                return Err(ReflectionParseError::TooShort);
            }
            let field_data = &data[start..field_end];
            let mut inner_off = 0;
            while inner_off < field_data.len() {
                if inner_off + 2 > field_data.len() {
                    break;
                }
                let inner_tag = field_data[inner_off];
                let inner_field = inner_tag >> 3;
                let inner_wire = inner_tag & 0x07;
                if inner_field == 1 && inner_wire == 2 {
                    let (name_len, name_start) = read_varint(field_data, inner_off + 1)?;
                    let name_end = name_start + name_len as usize;
                    if name_end > field_data.len() {
                        return Err(ReflectionParseError::TooShort);
                    }
                    let name = core::str::from_utf8(&field_data[name_start..name_end])
                        .map_err(|_| ReflectionParseError::ParseError)?;
                    services.push(name.to_owned());
                    inner_off = name_end;
                } else {
                    inner_off = skip_field(field_data, inner_off)?;
                }
            }
            offset = field_end;
        } else {
            offset = skip_field(data, offset)?;
        }
    }
    Ok(services)
}

pub fn build_reflection_probe() -> Vec<u8> {
    let mut headers = HeadersFrame::new(1);
    headers
        .add_header(":method", "POST")
        .add_header(
            ":path",
            "/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo",
        )
        .add_header(":scheme", "https")
        .add_header("content-type", "application/grpc")
        .add_header("te", "trailers");
    let header_bytes = headers.serialize();

    let request = encode_list_services();
    let grpc_msg = encode_grpc_message(&request);
    let mut data = DataFrame::new(1);
    data.set_payload(&grpc_msg);
    let data_bytes = data.serialize();

    let mut probe = Vec::with_capacity(header_bytes.len() + data_bytes.len());
    probe.extend_from_slice(&header_bytes);
    probe.extend_from_slice(&data_bytes);
    probe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_list_services_bytes() {
        let enc = encode_list_services();
        assert_eq!(enc, [0x2A, 0x00]);
    }

    #[test]
    fn encode_list_services_tag_is_field5_wiretype2() {
        let enc = encode_list_services();
        assert_eq!(enc[0], (5 << 3) | 2);
        assert_eq!(enc[1], 0x00);
    }

    #[test]
    fn decode_list_services_single_service() {
        // field 1 (tag 0x0A) = ServiceResponse, inner field 1 (tag 0x0A) = name "foo"
        let data = [0x0A, 0x05, 0x0A, 0x03, b'f', b'o', b'o'];
        let services = decode_list_services(&data).unwrap();
        assert_eq!(services, vec!["foo"]);
    }

    #[test]
    fn decode_list_services_multiple() {
        let data = [
            0x0A, 0x05, 0x0A, 0x03, b'a', b'b', b'c', // svc abc
            0x0A, 0x04, 0x0A, 0x02, b'x', b'y', // svc xy
        ];
        let services = decode_list_services(&data).unwrap();
        assert_eq!(services, vec!["abc", "xy"]);
    }

    #[test]
    fn decode_list_services_empty() {
        let services = decode_list_services(&[]).unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn decode_list_services_nested_message() {
        // field 2 (tag 0x12) = skip, then field 1 = service
        let data = [
            0x12, 0x02, 0x08, 0x01, // skip field 2
            0x0A, 0x03, 0x0A, 0x01, b'z', // svc z
        ];
        let services = decode_list_services(&data).unwrap();
        assert_eq!(services, vec!["z"]);
    }

    #[test]
    fn decode_list_services_too_short() {
        let data = [0x0A, 0x10];
        let r = decode_list_services(&data);
        assert!(matches!(r, Err(ReflectionParseError::TooShort)));
    }

    #[test]
    fn decode_list_services_bad_utf8() {
        let data = [0x0A, 0x03, 0x0A, 0x01, 0xFF];
        let r = decode_list_services(&data);
        assert!(matches!(r, Err(ReflectionParseError::ParseError)));
    }

    #[test]
    fn build_reflection_probe_two_frames() {
        let probe = build_reflection_probe();
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        assert_eq!(probe[3], 0x01);
        let second_offset = 9 + first_len;
        assert_eq!(probe[second_offset + 3], 0x00);
        assert!(probe.len() > second_offset + 9);
    }

    #[test]
    fn build_reflection_probe_headers_stream_id() {
        let probe = build_reflection_probe();
        let id = ((probe[5] as u32) << 24)
            | ((probe[6] as u32) << 16)
            | ((probe[7] as u32) << 8)
            | (probe[8] as u32);
        assert_eq!(id, 1);
    }

    #[test]
    fn build_reflection_probe_data_frame_stream_id() {
        let probe = build_reflection_probe();
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
    fn build_reflection_probe_data_has_grpc_prefix() {
        let probe = build_reflection_probe();
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let off = 9 + first_len + 9;
        assert_eq!(probe[off], 0x00);
    }

    #[test]
    fn build_reflection_probe_data_contains_list_services() {
        let probe = build_reflection_probe();
        let first_len =
            ((probe[0] as usize) << 16) | ((probe[1] as usize) << 8) | (probe[2] as usize);
        let off = 9 + first_len + 9 + 5;
        assert_eq!(probe[off], 0x2A);
        assert_eq!(probe[off + 1], 0x00);
    }

    #[test]
    fn decode_list_services_roundtrip() {
        let original = vec![
            0x0A, 0x06, 0x0A, 0x04, b't', b'e', b's', b't', 0x0A, 0x05, 0x0A, 0x03, b'a', b'p',
            b'p',
        ];
        let services = decode_list_services(&original).unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0], "test");
        assert_eq!(services[1], "app");
    }
}
