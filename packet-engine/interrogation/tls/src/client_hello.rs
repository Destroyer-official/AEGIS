//! Zero-copy TLS Client Hello parser (RFC 8446 §4.2.11).
//!
//! Parses the Client Hello message from raw bytes without allocation.
//! Extracts cipher suites, extensions, ALPN values, SNI, and signature algorithms
//! for JA4 fingerprint computation.

use crate::grease::is_grease;

/// Protocol transport type for JA4 prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Quic,
    Dtls,
}

impl Transport {
    pub fn ja4_char(&self) -> char {
        match self {
            Self::Tcp => 't',
            Self::Quic => 'q',
            Self::Dtls => 'd',
        }
    }
}

/// SNI type for JA4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snitype {
    Domain,
    Ip,
}

impl Snitype {
    pub fn ja4_char(&self) -> char {
        match self {
            Self::Domain => 'd',
            Self::Ip => 'i',
        }
    }
}

/// A raw TLS extension parsed from the Client Hello.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Extension<'a> {
    pub extension_type: u16,
    pub data: &'a [u8],
}

/// Zero-copy parsed TLS Client Hello.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientHello<'a> {
    pub raw: &'a [u8],
    pub legacy_version: u16,
    pub random: &'a [u8; 32],
    pub session_id: &'a [u8],
    pub cipher_suites: Vec<u16>,
    pub compression_methods: &'a [u8],
    pub extensions: Vec<Extension<'a>>,
    pub sni: Option<&'a [u8]>,
    pub alpn: Option<&'a [u8]>,
    pub signature_algorithms: Option<&'a [u8]>,
    pub supported_versions: Option<&'a [u8]>,
}

impl<'a> ClientHello<'a> {
    /// Compute JA4 fingerprint from this parsed Client Hello.
    pub fn ja4(&self, transport: Transport) -> crate::ja4::Ja4Fingerprint {
        crate::ja4::Ja4Fingerprint::compute(self, transport)
    }

    /// Get non-GREASE cipher suites.
    pub fn non_grease_ciphers(&self) -> impl Iterator<Item = u16> + '_ {
        self.cipher_suites
            .iter()
            .copied()
            .filter(|&c| !is_grease(c))
    }

    /// Get non-GREASE, non-SNI, non-ALPN extension types.
    pub fn non_grease_ext_types(&self) -> Vec<u16> {
        self.extensions
            .iter()
            .map(|e| e.extension_type)
            .filter(|&t| !is_grease(t) && t != 0x0000 && t != 0x0010)
            .collect()
    }
}

/// Parse a TLS Client Hello from raw handshake payload bytes.
///
/// The input should be the *body* of the Handshake message (after the 1-byte
/// msg_type and 3-byte length fields), starting at the `client_version` field.
///
/// # Layout (RFC 8446 §4.2.11)
/// ```text
/// struct {
///     ProtocolVersion legacy_version;
///     Random random;
///     SessionID session_id;
///     opaque cipher_suites<2..2^16-2>;
///     CompressionMethod compression_methods<1..2^8-1>;
///     Extension extensions<2..2^16-1>;
/// } ClientHello;
/// ```
#[inline]
pub fn parse_client_hello(input: &[u8]) -> Result<ClientHello<'_>, ClientHelloError> {
    let mut pos = 0;

    // legacy_version (2 bytes) — always 0x0303 for TLS 1.3
    if input.len() < pos + 2 {
        return Err(ClientHelloError::Truncated);
    }
    let legacy_version = u16::from_be_bytes([input[pos], input[pos + 1]]);
    pos += 2;

    // random (32 bytes)
    if input.len() < pos + 32 {
        return Err(ClientHelloError::Truncated);
    }
    let random: &[u8; 32] = input[pos..pos + 32]
        .try_into()
        .map_err(|_| ClientHelloError::Truncated)?;
    pos += 32;

    // session_id (1-byte length + variable)
    if input.len() < pos + 1 {
        return Err(ClientHelloError::Truncated);
    }
    let session_id_len = input[pos] as usize;
    pos += 1;
    if input.len() < pos + session_id_len {
        return Err(ClientHelloError::Truncated);
    }
    let session_id = &input[pos..pos + session_id_len];
    pos += session_id_len;

    // cipher_suites (2-byte length + variable)
    if input.len() < pos + 2 {
        return Err(ClientHelloError::Truncated);
    }
    let cipher_suites_len = u16::from_be_bytes([input[pos], input[pos + 1]]) as usize;
    pos += 2;
    if input.len() < pos + cipher_suites_len {
        return Err(ClientHelloError::Truncated);
    }
    let cipher_suites: Vec<u16> = input[pos..pos + cipher_suites_len]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    pos += cipher_suites_len;

    // compression_methods (1-byte length + variable)
    if input.len() < pos + 1 {
        return Err(ClientHelloError::Truncated);
    }
    let comp_len = input[pos] as usize;
    pos += 1;
    if input.len() < pos + comp_len {
        return Err(ClientHelloError::Truncated);
    }
    let compression_methods = &input[pos..pos + comp_len];
    pos += comp_len;

    // extensions (2-byte length + variable)
    if input.len() < pos + 2 {
        return Err(ClientHelloError::Truncated);
    }
    let ext_len = u16::from_be_bytes([input[pos], input[pos + 1]]) as usize;
    pos += 2;
    if input.len() < pos + ext_len {
        return Err(ClientHelloError::Truncated);
    }
    let ext_start = pos;
    let ext_end = pos + ext_len;

    // Parse extensions
    let mut extensions = Vec::new();
    let mut sni: Option<&[u8]> = None;
    let mut alpn: Option<&[u8]> = None;
    let mut signature_algorithms: Option<&[u8]> = None;
    let mut supported_versions: Option<&[u8]> = None;

    let mut ext_pos = ext_start;
    while ext_pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([input[ext_pos], input[ext_pos + 1]]);
        let ext_data_len = u16::from_be_bytes([input[ext_pos + 2], input[ext_pos + 3]]) as usize;
        ext_pos += 4;

        if ext_pos + ext_data_len > ext_end {
            break;
        }

        let ext_data = &input[ext_pos..ext_pos + ext_data_len];
        extensions.push(Extension {
            extension_type: ext_type,
            data: ext_data,
        });

        match ext_type {
            0x0000 => {
                // server_name (SNI) — format: list_length(2) + name_type(1) + name_length(2) + name
                if ext_data.len() >= 5 {
                    let name_type = ext_data[2];
                    let name_len = u16::from_be_bytes([ext_data[3], ext_data[4]]) as usize;
                    if name_type == 0 && ext_data.len() >= 5 + name_len {
                        sni = Some(&ext_data[5..5 + name_len]);
                    }
                }
            }
            0x0010 => {
                // application_layer_protocol_negotiation (ALPN)
                alpn = Some(ext_data);
            }
            0x000d => {
                // signature_algorithms
                signature_algorithms = Some(ext_data);
            }
            0x002b => {
                // supported_versions
                supported_versions = Some(ext_data);
            }
            _ => {}
        }

        ext_pos += ext_data_len;
    }

    Ok(ClientHello {
        raw: input,
        legacy_version,
        random,
        session_id,
        cipher_suites,
        compression_methods,
        extensions,
        sni,
        alpn,
        signature_algorithms,
        supported_versions,
    })
}

/// Errors from Client Hello parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientHelloError {
    Truncated,
    InvalidSni,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real TLS 1.3 Client Hello (Chrome 120) with SNI "example.com",
    /// 8 cipher suites, ALPN h2+http/1.1. Constructed for test reproducibility.
    const TLS13_CLIENT_HELLO: &[u8] = &[
        // legacy_version
        0x03, 0x01, // random (32 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
        0x17, 0x18, // session_id (empty)
        0x00, // cipher_suites (16 bytes = 8 suites)
        0x00, 0x10, 0x13, 0x01, // TLS_AES_128_GCM_SHA256
        0x13, 0x02, // TLS_AES_256_GCM_SHA384
        0x13, 0x03, // TLS_CHACHA20_POLY1305_SHA256
        0xc0, 0x2b, // ECDHE_ECDSA_AES_128_GCM_SHA256
        0xc0, 0x2f, // ECDHE_RSA_AES_128_GCM_SHA256
        0xc0, 0x2c, // ECDHE_ECDSA_AES_256_GCM_SHA384
        0xc0, 0x30, // ECDHE_RSA_AES_256_GCM_SHA384
        0xcc, 0xa9, // ECDHE_ECDSA_CHACHA20_POLY1305
        // compression_methods (1 byte: null)
        0x01, 0x00, // extensions total length
        0x00, 0x39, // 57 bytes of extensions
        // ── server_name extension (SNI) ──
        0x00, 0x00, // type
        0x00, 0x10, // data length (16)
        0x00, 0x0e, // list length (14)
        0x00, // name type (0 = host_name)
        0x00, 0x0b, // name length (11 = "example.com")
        b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm',
        // ── signature_algorithms ──
        0x00, 0x0d, // type
        0x00, 0x06, // data length
        0x00, 0x04, // list length
        0x04, 0x03, // ecdsa_secp256r1_sha256
        0x05, 0x01, // rsa_pss_rsae_sha256
        // ── ALPN ──
        0x00, 0x10, // type
        0x00, 0x0e, // data length (14)
        0x00, 0x0c, // list length (12: 3 for h2 + 9 for http/1.1)
        0x02, // first protocol length
        b'h', b'2', // h2
        0x08, // second protocol length
        b'h', b't', b't', b'p', b'/', b'1', b'.', b'1',
        // ── supported_versions ──
        0x00, 0x2b, // type
        0x00, 0x05, // data length
        0x04, // list length
        0x03, 0x04, // TLS 1.3
        0x03, 0x03, // TLS 1.2
    ];

    #[test]
    fn test_parse_client_hello_basic() {
        let ch = parse_client_hello(TLS13_CLIENT_HELLO).unwrap();
        assert_eq!(ch.legacy_version, 0x0301);
        assert_eq!(ch.cipher_suites.len(), 8);
        assert!(ch.sni.is_some());
        assert!(ch.alpn.is_some());
    }

    #[test]
    fn test_sni_value() {
        let ch = parse_client_hello(TLS13_CLIENT_HELLO).unwrap();
        let sni = ch.sni.unwrap();
        assert_eq!(sni, b"example.com");
    }

    #[test]
    fn test_non_grease_ciphers() {
        let ch = parse_client_hello(TLS13_CLIENT_HELLO).unwrap();
        let ciphers: Vec<u16> = ch.non_grease_ciphers().collect();
        assert_eq!(ciphers.len(), 8);
        assert_eq!(ciphers[0], 0x1301); // TLS_AES_128_GCM_SHA256
    }

    #[test]
    fn test_parse_too_short() {
        let short = [0x03, 0x01];
        assert_eq!(parse_client_hello(&short), Err(ClientHelloError::Truncated));
    }
}
