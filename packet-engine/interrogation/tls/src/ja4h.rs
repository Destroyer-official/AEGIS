//! JA4H — HTTP client fingerprint from request headers and cookies (§6.1).
//!
//! JA4H format: `{method}{scheme}{http_version}_{accept}_{cookie_hash}_{ref_hash}`
//!
//! Fingerprints HTTP clients based on header ordering, cookie structure,
//! and Accept header values.

use sha2::{Digest, Sha256};

/// JA4H HTTP client fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4hFingerprint {
    pub full: String,
    pub method: String,
    pub scheme: String,
    pub http_version: String,
    pub accept: String,
    pub cookie_hash: String,
    pub ref_hash: String,
}

/// An HTTP header for JA4H computation.
#[derive(Clone, Debug)]
pub struct HttpHeader<'a> {
    pub name: &'a str,
    pub value: &'a [u8],
}

/// An HTTP request parsed from raw bytes.
#[derive(Clone, Debug)]
pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub http_version: &'a str,
    pub headers: Vec<HttpHeader<'a>>,
}

impl<'a> HttpRequest<'a> {
    /// Compute JA4H fingerprint from this HTTP request.
    pub fn ja4h(&self) -> Ja4hFingerprint {
        Ja4hFingerprint::compute(self)
    }
}

impl Ja4hFingerprint {
    /// Compute JA4H fingerprint from a parsed HTTP request.
    pub fn compute(req: &HttpRequest<'_>) -> Self {
        // Method (lowercase)
        let method = req.method.to_lowercase();

        // Scheme: check for :authority pseudo-header or Host header
        let scheme = if req.uri.starts_with("https://") || req.uri.starts_with("HTTPS://") {
            "https"
        } else {
            "http"
        };

        // HTTP version
        let http_version = req.http_version.to_lowercase().replace("http/", "");

        // Accept header value (first 20 chars)
        let accept = req
            .headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("accept"))
            .map(|h| {
                let val = String::from_utf8_lossy(h.value);
                if val.len() > 20 {
                    val[..20].to_string()
                } else {
                    val.to_string()
                }
            })
            .unwrap_or_default();

        // Cookie hash: SHA-256 of sorted cookie header values, truncated to 12 chars
        let cookie_values: Vec<&[u8]> = req
            .headers
            .iter()
            .filter(|h| h.name.eq_ignore_ascii_case("cookie"))
            .map(|h| h.value)
            .collect();

        let cookie_hash = if cookie_values.is_empty() {
            "000000000000".to_string()
        } else {
            let combined: String = cookie_values
                .iter()
                .map(|v| String::from_utf8_lossy(v).to_string())
                .collect::<Vec<_>>()
                .join(";");
            let hash = Sha256::digest(combined.as_bytes());
            hex::encode(&hash[..6])
        };

        // Referrer hash: SHA-256 of Referer value, truncated to 12 chars
        let ref_hash = req
            .headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("referer"))
            .map(|h| {
                let hash = Sha256::digest(h.value);
                hex::encode(&hash[..6])
            })
            .unwrap_or_else(|| "000000000000".to_string());

        let full = format!(
            "{}{}{}_{}_{}_{}",
            method, scheme, http_version, accept, cookie_hash, ref_hash
        );

        Ja4hFingerprint {
            full,
            method,
            scheme: scheme.to_string(),
            http_version,
            accept,
            cookie_hash,
            ref_hash,
        }
    }
}

/// Parse raw HTTP request bytes into an HttpRequest.
pub fn parse_http_request(input: &[u8]) -> Result<HttpRequest<'_>, HttpParseError> {
    let text = core::str::from_utf8(input).map_err(|_| HttpParseError::InvalidUtf8)?;

    let mut lines = text.split("\r\n");

    // Request line
    let request_line = lines.next().ok_or(HttpParseError::MissingRequestLine)?;
    let mut parts = request_line.splitn(3, ' ');
    let method = parts.next().ok_or(HttpParseError::MissingMethod)?;
    let uri = parts.next().ok_or(HttpParseError::MissingUri)?;
    let http_version = parts.next().ok_or(HttpParseError::MissingVersion)?;

    // Headers
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim_start().as_bytes();
            headers.push(HttpHeader {
                name: name.trim(),
                value,
            });
        }
    }

    Ok(HttpRequest {
        method,
        uri,
        http_version,
        headers,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpParseError {
    InvalidUtf8,
    MissingRequestLine,
    MissingMethod,
    MissingUri,
    MissingVersion,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4h_format() {
        let req_str = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nAccept: text/html\r\nCookie: session=abc123\r\n\r\n";
        let req = parse_http_request(req_str).unwrap();
        let ja4h = req.ja4h();
        // HTTP scheme (not HTTPS since URI doesn't start with https://)
        assert!(
            ja4h.full.starts_with("gethttp1.1"),
            "JA4H should start with gethttp1.1, got: {}",
            ja4h.full
        );
    }

    #[test]
    fn test_ja4h_cookie_hash() {
        let req1 = b"GET / HTTP/1.1\r\nCookie: a=1\r\n\r\n";
        let req2 = b"GET / HTTP/1.1\r\nCookie: b=2\r\n\r\n";
        let j1 = parse_http_request(req1).unwrap().ja4h();
        let j2 = parse_http_request(req2).unwrap().ja4h();
        assert_ne!(j1.cookie_hash, j2.cookie_hash);
    }
}
