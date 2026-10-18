//! Zero-copy TLS Server Hello parser (RFC 8446 §4.1.3).
//!
//! Parses the Server Hello message for JA4S fingerprint computation.
//! Extracts cipher suite, extensions, and ALPN negotiation result.

/// A raw TLS extension parsed from the Server Hello.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtEntry<'a> {
    pub extension_type: u16,
    pub data: &'a [u8],
}

/// Zero-copy parsed TLS Server Hello.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerHello<'a> {
    pub raw: &'a [u8],
    pub legacy_version: u16,
    pub random: &'a [u8; 32],
    pub session_id_echo: &'a [u8],
    pub cipher_suite: u16,
    pub compression_method: u8,
    pub extensions: Vec<ExtEntry<'a>>,
    pub alpn: Option<&'a [u8]>,
    pub supported_version: Option<u16>,
}

impl<'a> ServerHello<'a> {
    /// Compute JA4S fingerprint from this parsed Server Hello.
    pub fn ja4s(&self) -> crate::ja4s::Ja4sFingerprint {
        crate::ja4s::Ja4sFingerprint::compute(self)
    }
}

/// Parse a TLS Server Hello from raw handshake payload bytes.
///
/// Input should be the body of the Handshake message (after the 1-byte
/// msg_type and 3-byte length fields), starting at `server_version`.
///
/// # Layout (RFC 8446 §4.1.3)
/// ```text
/// struct {
///     ProtocolVersion legacy_version;
///     Random random;
///     SessionID session_id_echo;
///     CipherSuite cipher_suite;
///     CompressionMethod compression_method;
///     Extension extensions<6..2^16-1>;
/// } ServerHello;
/// ```
#[inline]
pub fn parse_server_hello(input: &[u8]) -> Result<ServerHello<'_>, ServerHelloError> {
    let mut pos = 0;

    // legacy_version (2 bytes)
    if input.len() < pos + 2 {
        return Err(ServerHelloError::Truncated);
    }
    let legacy_version = u16::from_be_bytes([input[pos], input[pos + 1]]);
    pos += 2;

    // random (32 bytes)
    if input.len() < pos + 32 {
        return Err(ServerHelloError::Truncated);
    }
    let random: &[u8; 32] = input[pos..pos + 32]
        .try_into()
        .map_err(|_| ServerHelloError::Truncated)?;
    pos += 32;

    // session_id_echo (1-byte length + variable)
    if input.len() < pos + 1 {
        return Err(ServerHelloError::Truncated);
    }
    let sid_len = input[pos] as usize;
    pos += 1;
    if input.len() < pos + sid_len {
        return Err(ServerHelloError::Truncated);
    }
    let session_id_echo = &input[pos..pos + sid_len];
    pos += sid_len;

    // cipher_suite (2 bytes)
    if input.len() < pos + 2 {
        return Err(ServerHelloError::Truncated);
    }
    let cipher_suite = u16::from_be_bytes([input[pos], input[pos + 1]]);
    pos += 2;

    // compression_method (1 byte)
    if input.len() < pos + 1 {
        return Err(ServerHelloError::Truncated);
    }
    let compression_method = input[pos];
    pos += 1;

    // extensions (2-byte length + variable)
    if input.len() < pos + 2 {
        return Err(ServerHelloError::Truncated);
    }
    let ext_len = u16::from_be_bytes([input[pos], input[pos + 1]]) as usize;
    pos += 2;
    if input.len() < pos + ext_len {
        return Err(ServerHelloError::Truncated);
    }
    let ext_end = pos + ext_len;

    let mut extensions = Vec::new();
    let mut alpn: Option<&[u8]> = None;
    let mut supported_version: Option<u16> = None;

    let mut ext_pos = pos;
    while ext_pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([input[ext_pos], input[ext_pos + 1]]);
        let ext_data_len = u16::from_be_bytes([input[ext_pos + 2], input[ext_pos + 3]]) as usize;
        ext_pos += 4;

        if ext_pos + ext_data_len > ext_end {
            break;
        }

        let ext_data = &input[ext_pos..ext_pos + ext_data_len];
        extensions.push(ExtEntry {
            extension_type: ext_type,
            data: ext_data,
        });

        match ext_type {
            0x0010 => {
                alpn = Some(ext_data);
            }
            0x002b
                // supported_versions (for TLS 1.3 downgrade protection)
                if ext_data.len() >= 2 => {
                    supported_version = Some(u16::from_be_bytes([ext_data[0], ext_data[1]]));
                }
            _ => {}
        }

        ext_pos += ext_data_len;
    }

    Ok(ServerHello {
        raw: input,
        legacy_version,
        random,
        session_id_echo,
        cipher_suite,
        compression_method,
        extensions,
        alpn,
        supported_version,
    })
}

/// Errors from Server Hello parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerHelloError {
    Truncated,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal TLS 1.2 Server Hello with AES_128_GCM_SHA256
    const TLS12_SERVER_HELLO: &[u8] = &[
        // legacy_version
        0x03, 0x03, // random (32 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, // session_id_echo (empty)
        0x00, // cipher_suite: TLS_RSA_WITH_AES_128_GCM_SHA256
        0x00, 0x9c, // compression_method: null
        0x00, // extensions length: 0
        0x00, 0x00,
    ];

    #[test]
    fn test_parse_server_hello_basic() {
        let sh = parse_server_hello(TLS12_SERVER_HELLO).unwrap();
        assert_eq!(sh.legacy_version, 0x0303);
        assert_eq!(sh.cipher_suite, 0x009c);
        assert!(sh.extensions.is_empty());
    }

    #[test]
    fn test_parse_too_short() {
        let short = [0x03, 0x03];
        assert_eq!(parse_server_hello(&short), Err(ServerHelloError::Truncated));
    }
}
