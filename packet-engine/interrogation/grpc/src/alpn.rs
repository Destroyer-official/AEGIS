//! ALPN (Application-Layer Protocol Negotiation) for gRPC (§6.3).
//!
//! Constructs the ALPN extension data for TLS Client Hello to advertise
//! gRPC support alongside standard HTTP/2.
//!
//! ALPN extension format (RFC 7301):
//! ```text
//! ExtensionType: 0x0010 (application_layer_protocol_negotiation)
//! Data:
//!   2-byte list_length
//!   Per protocol:
//!     1-byte protocol_name_length
//!     protocol_name (UTF-8)
//! ```

/// gRPC ALPN negotiation data for TLS Client Hello.
///
/// Contains `\x02h2\x05grpc` — advertising both HTTP/2 and gRPC protocols.
pub struct GrpcAlpn;

impl GrpcAlpn {
    /// Raw ALPN extension data: 2-byte list length + h2 (3 bytes) + grpc (5 bytes)
    pub const ALPN_DATA: [u8; 10] = [
        0x00, 0x08, // list length: 8 bytes total (3 + 5)
        0x02, // h2 length
        b'h', b'2', // h2
        0x04, // grpc length
        b'g', b'r', b'p', b'c', // grpc
    ];

    /// ALPN extension type (RFC 7301)
    pub const EXTENSION_TYPE: u16 = 0x0010;

    /// Build the complete ALPN extension for a TLS Client Hello.
    /// Returns the extension type (2 bytes) + extension data length (2 bytes) + data.
    pub fn build_extension() -> [u8; 14] {
        let mut ext = [0u8; 14];
        ext[0..2].copy_from_slice(&Self::EXTENSION_TYPE.to_be_bytes());
        ext[2..4].copy_from_slice(&(Self::ALPN_DATA.len() as u16).to_be_bytes());
        ext[4..14].copy_from_slice(&Self::ALPN_DATA);
        ext
    }

    /// Parse ALPN data from extension bytes to extract the negotiated protocol list.
    pub fn parse_alpn_list(data: &[u8]) -> Vec<String> {
        let mut protocols = Vec::new();
        if data.len() < 2 {
            return protocols;
        }

        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        let mut pos = 2;
        let end = std::cmp::min(2 + list_len, data.len());

        while pos < end {
            let proto_len = data[pos] as usize;
            pos += 1;
            if pos + proto_len > end {
                break;
            }
            if let Ok(proto) = std::str::from_utf8(&data[pos..pos + proto_len]) {
                protocols.push(proto.to_string());
            }
            pos += proto_len;
        }

        protocols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpn_extension_length() {
        let ext = GrpcAlpn::build_extension();
        assert_eq!(ext.len(), 14);
    }

    #[test]
    fn test_alpn_extension_type() {
        let ext = GrpcAlpn::build_extension();
        assert_eq!(u16::from_be_bytes([ext[0], ext[1]]), 0x0010);
    }

    #[test]
    fn test_alpn_data_length() {
        let ext = GrpcAlpn::build_extension();
        // ext[2..4] is the extension data length (ALPN_DATA.len())
        let data_len = u16::from_be_bytes([ext[2], ext[3]]);
        assert_eq!(data_len, GrpcAlpn::ALPN_DATA.len() as u16);
    }

    #[test]
    fn test_alpn_h2_and_grpc() {
        let ext = GrpcAlpn::build_extension();
        let data = &ext[4..];
        let protos = GrpcAlpn::parse_alpn_list(data);
        assert_eq!(protos.len(), 2);
        assert_eq!(protos[0], "h2");
        assert_eq!(protos[1], "grpc");
    }

    #[test]
    fn test_parse_alpn_list_empty() {
        let protos = GrpcAlpn::parse_alpn_list(&[]);
        assert!(protos.is_empty());
    }

    #[test]
    fn test_parse_alpn_list_single() {
        // list length (2 bytes) + protocol length (1 byte) + protocol name (2 bytes)
        let data = [0x00, 0x03, 0x02, b'h', b'2'];
        let protos = GrpcAlpn::parse_alpn_list(&data);
        assert_eq!(protos, vec!["h2"]);
    }
}
