//! Certificate Transparency (CT) log monitor (RFC 6962).
//!
//! Watches CT log entries and alerts when certificates match JA4+ fingerprints
//! from our watchlist. Operates on Merkle tree head timestamps and SCT
//! (Signed Certificate Timestamp) data.
//!
//! # Design
//!
//! The monitor maintains a local state of the latest seen MTH (Merkle Tree
//! Head) for each log. On each poll cycle it:
//! 1. Fetches the latest MTH from each configured log
//! 2. Compares with the stored MTH to detect new entries
//! 3. For each new certificate entry, extracts the SAN, issuer, and serial
//! 4. Computes the JA4 fingerprint from the raw TLS Client Hello (if available)
//!    or derives a JA4S (server fingerprint) from the certificate's SAN pattern
//! 5. Checks against the watchlist and emits alerts for matches
//!
//! # Wire Format
//!
//! CT log entries use Merkle Tree Leaf format:
//! ```text
//! struct MerkleTreeLeaf {
//!     version: u8,          // 0x00
//!     type: u8,             // 0x00 = pre-certificate
//!     timestamp: u64,       // milliseconds since epoch
//!     entry_type: u16,      // 0 = x509_entry, 1 = precert_entry
//!     ...
//! }
//! ```

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// CT Log entry types (RFC 6962 §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtEntryType {
    X509 = 0,
    Precert = 1,
}

impl CtEntryType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            0 => Some(Self::X509),
            1 => Some(Self::Precert),
            _ => None,
        }
    }
}

/// CT log entry parsed from a Merkle Tree Leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtLogEntry {
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
    /// Entry type (X.509 or pre-certificate)
    pub entry_type: CtEntryType,
    /// Certificate serial number (raw bytes)
    pub serial_number: Vec<u8>,
    /// Issuer DN (Distinguished Name, raw bytes)
    pub issuer_dn: Vec<u8>,
    /// Subject Alternative Names (DNS names)
    pub sans: Vec<String>,
    /// Not Before (seconds since epoch)
    pub not_before: u64,
    /// Not After (seconds since epoch)
    pub not_after: u64,
    /// Raw certificate bytes
    pub cert_der: Vec<u8>,
}

/// Parse a DER-encoded X.509 certificate to extract basic fields.
///
/// This is a minimal ASN.1 parser that extracts:
/// - Serial number (field 1 in Certificate SEQUENCE)
/// - Issuer DN (field 4)
/// - Subject DN (field 5)
/// - Validity (field 6)
/// - SAN extension (from Extensions → SubjectAltName)
pub fn parse_x509_basic(cert_der: &[u8]) -> Result<CtLogEntry, CtParseError> {
    if cert_der.len() < 30 {
        return Err(CtParseError::TooShort);
    }

    // Check SEQUENCE tag (0x30)
    if cert_der[0] != 0x30 {
        return Err(CtParseError::InvalidTag);
    }

    let (_, mut pos) = parse_asn1_length(cert_der, 1)?;

    // TBSCertificate SEQUENCE
    if pos >= cert_der.len() || cert_der[pos] != 0x30 {
        return Err(CtParseError::InvalidTag);
    }
    let (tbs_len, tbs_start) = parse_asn1_length(cert_der, pos + 1)?;
    let tbs_end = tbs_start + tbs_len;
    pos = tbs_start;

    // Version [0] EXPLICIT (optional)
    if pos < cert_der.len() && cert_der[pos] == 0xa0 {
        let (ver_len, ver_body) = parse_asn1_length(cert_der, pos + 1)?;
        pos = ver_body + ver_len;
    }

    // Serial number (INTEGER)
    if pos >= cert_der.len() || cert_der[pos] != 0x02 {
        return Err(CtParseError::InvalidTag);
    }
    let (serial_len, serial_start) = parse_asn1_length(cert_der, pos + 1)?;
    let serial_number = cert_der[serial_start..serial_start + serial_len].to_vec();
    pos = serial_start + serial_len;

    // Signature algorithm (skip)
    pos = skip_asn1_field(cert_der, pos)?;

    // Issuer SEQUENCE
    if pos >= cert_der.len() || cert_der[pos] != 0x30 {
        return Err(CtParseError::InvalidTag);
    }
    let (issuer_len, issuer_start) = parse_asn1_length(cert_der, pos + 1)?;
    let issuer_dn = cert_der[issuer_start..issuer_start + issuer_len].to_vec();
    pos = issuer_start + issuer_len;

    // Validity SEQUENCE
    if pos >= cert_der.len() || cert_der[pos] != 0x30 {
        return Err(CtParseError::InvalidTag);
    }
    let (validity_len, validity_start) = parse_asn1_length(cert_der, pos + 1)?;
    let validity_end = validity_start + validity_len;

    // Parse notBefore and notAfter (UTCTime or GeneralizedTime)
    let not_before = parse_asn1_time(cert_der, validity_start)?;
    let not_after = parse_asn1_time(cert_der, skip_asn1_field(cert_der, validity_start)?)?;
    pos = validity_end;

    // Subject SEQUENCE (skip)
    pos = skip_asn1_field(cert_der, pos)?;

    // SubjectPublicKeyInfo SEQUENCE (skip)
    pos = skip_asn1_field(cert_der, pos)?;

    // Extensions [3] EXPLICIT (optional — contains SAN)
    let mut sans = Vec::new();
    if pos < tbs_end && cert_der[pos] == 0xa3 {
        let (_, ext_content) = parse_asn1_length(cert_der, pos + 1)?;
        // ext_content points to the inner SEQUENCE tag (0x30)
        if ext_content < cert_der.len() && cert_der[ext_content] == 0x30 {
            let (_, seq_body) = parse_asn1_length(cert_der, ext_content + 1)?;
            sans = extract_sans_from_extensions(cert_der, seq_body);
        }
    }

    Ok(CtLogEntry {
        timestamp: 0,
        entry_type: CtEntryType::X509,
        serial_number,
        issuer_dn,
        sans,
        not_before,
        not_after,
        cert_der: cert_der.to_vec(),
    })
}

/// Extract Subject Alternative Names from an X.509 Extensions block.
fn extract_sans_from_extensions(data: &[u8], start: usize) -> Vec<String> {
    let mut sans = Vec::new();
    let mut pos = start;

    while pos + 4 < data.len() {
        // SEQUENCE (extension)
        if data[pos] != 0x30 {
            break;
        }
        let (ext_len, ext_body) = match parse_asn1_length(data, pos + 1) {
            Ok(v) => v,
            Err(_) => break,
        };
        let ext_end = ext_body + ext_len;
        if ext_end > data.len() {
            break;
        }

        // Extension OID (2.5.29.17 = SubjectAltName)
        if ext_body < data.len() && data[ext_body] == 0x06 {
            let (oid_len, oid_start) = match parse_asn1_length(data, ext_body + 1) {
                Ok(v) => v,
                Err(_) => {
                    pos = ext_end;
                    continue;
                }
            };
            let oid_end = oid_start + oid_len;
            if oid_end <= data.len() && data[oid_start..oid_end] == [0x55, 0x1d, 0x11] {
                // SubjectAltName OID found — parse the value
                if oid_end < ext_end && data[oid_end] == 0x04 {
                    if let Ok((val_len, val_start)) = parse_asn1_length(data, oid_end + 1) {
                        let val_end = val_start + val_len;
                        if val_end <= data.len() {
                            // OCTET STRING containing SEQUENCE of GeneralName
                            let seq_start = if val_start < data.len() && data[val_start] == 0x30 {
                                val_start + 1
                            } else {
                                val_start
                            };
                            sans = parse_general_names(data, seq_start);
                        }
                    }
                }
            }
        }
        pos = ext_end;
    }
    sans
}

/// Parse GeneralName entries (RFC 5280 §4.2.1.6).
fn parse_general_names(data: &[u8], start: usize) -> Vec<String> {
    let mut sans = Vec::new();
    let mut pos = start;

    while pos < data.len() {
        let tag = data[pos];
        let wire_type = tag & 0x07;
        let name_type = (tag & 0x1F) as usize;

        if wire_type == 2 {
            // IMPLICIT [name_type] OCTET STRING
            if let Ok((name_len, name_start)) = parse_asn1_length(data, pos + 1) {
                let name_end = name_start + name_len;
                if name_end <= data.len() && name_type == 2 {
                    // dNSName (tag 2)
                    if let Ok(name) = std::str::from_utf8(&data[name_start..name_end]) {
                        sans.push(name.to_string());
                    }
                }
                pos = name_end;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    sans
}

/// Parse an ASN.1 length field.
fn parse_asn1_length(data: &[u8], pos: usize) -> Result<(usize, usize), CtParseError> {
    if pos >= data.len() {
        return Err(CtParseError::Truncated);
    }
    let first = data[pos];
    if first & 0x80 == 0 {
        // Short form
        Ok((first as usize, pos + 1))
    } else {
        let num_bytes = (first & 0x7f) as usize;
        if num_bytes == 0 || pos + 1 + num_bytes > data.len() {
            return Err(CtParseError::Truncated);
        }
        let mut length: usize = 0;
        for i in 0..num_bytes {
            length = (length << 8) | data[pos + 1 + i] as usize;
        }
        Ok((length, pos + 1 + num_bytes))
    }
}

/// Skip an ASN.1 field (tag + length + value).
fn skip_asn1_field(data: &[u8], pos: usize) -> Result<usize, CtParseError> {
    if pos >= data.len() {
        return Err(CtParseError::Truncated);
    }
    let (len, body_start) = parse_asn1_length(data, pos + 1)?;
    Ok(body_start + len)
}

/// Parse an ASN.1 time field (UTCTime or GeneralizedTime).
fn parse_asn1_time(data: &[u8], pos: usize) -> Result<u64, CtParseError> {
    if pos >= data.len() {
        return Err(CtParseError::Truncated);
    }
    let tag = data[pos];
    let (len, body) = parse_asn1_length(data, pos + 1)?;
    if body + len > data.len() {
        return Err(CtParseError::Truncated);
    }
    let time_str =
        std::str::from_utf8(&data[body..body + len]).map_err(|_| CtParseError::InvalidUtf8)?;

    // Remove trailing 'Z' if present
    let time_str = time_str.trim_end_matches('Z');

    match tag {
        0x17 => {
            // UTCTime: YYMMDDHHMMSSZ (13 bytes) or YYMMDDHHMM+HHMM
            if time_str.len() < 10 {
                return Err(CtParseError::InvalidTime);
            }
            let year: i32 = time_str[..2]
                .parse()
                .map_err(|_| CtParseError::InvalidTime)?;
            let year_final = if year >= 50 { 1900 + year } else { 2000 + year };
            parse_time_components(year_final, &time_str[2..])
        }
        0x18 => {
            // GeneralizedTime: YYYYMMDDHHMMSSZ (15 bytes)
            if time_str.len() < 14 {
                return Err(CtParseError::InvalidTime);
            }
            let year: i32 = time_str[..4]
                .parse()
                .map_err(|_| CtParseError::InvalidTime)?;
            parse_time_components(year, &time_str[4..])
        }
        _ => Err(CtParseError::InvalidTag),
    }
}

/// Parse time components from MMDDHHMMSS string.
fn parse_time_components(year: i32, rest: &str) -> Result<u64, CtParseError> {
    if rest.len() < 10 {
        return Err(CtParseError::InvalidTime);
    }
    let month: u32 = rest[..2].parse().map_err(|_| CtParseError::InvalidTime)?;
    let day: u32 = rest[2..4].parse().map_err(|_| CtParseError::InvalidTime)?;
    let hour: u32 = rest[4..6].parse().map_err(|_| CtParseError::InvalidTime)?;
    let min: u32 = rest[6..8].parse().map_err(|_| CtParseError::InvalidTime)?;
    let sec: u32 = rest[8..10].parse().map_err(|_| CtParseError::InvalidTime)?;

    // Simplified epoch calculation (no leap seconds)
    let days = (year - 1970) * 365 + ((year - 1968) / 4) - ((year - 1900) / 100)
        + ((year - 1600) / 400)
        + month_days(year, month) as i32
        + (day as i32 - 1);
    let epoch = days as u64 * 86400 + hour as u64 * 3600 + min as u64 * 60 + sec as u64;
    Ok(epoch)
}

/// Days in month (non-leap).
fn month_days(year: i32, month: u32) -> u32 {
    let days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut total = 0;
    for m in 1..month {
        total += days[m as usize];
    }
    if month > 2 && is_leap(year) {
        total += 1;
    }
    total
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// CT Merkle Tree Head (MTH).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleTreeHead {
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
    /// Tree size (number of leaves)
    pub tree_size: u64,
    /// Root hash (32 bytes for SHA-256)
    pub root_hash: [u8; 32],
}

/// Parse a CT MTH from raw bytes (RFC 6962 §3.5).
///
/// ```text
/// struct TreeHead {
///     timestamp: u64,     // 8 bytes big-endian
///     tree_size: u64,     // 8 bytes big-endian
///     root_hash: [u8; 32], // 32 bytes
/// }
/// ```
pub fn parse_merkle_tree_head(data: &[u8]) -> Result<MerkleTreeHead, CtParseError> {
    if data.len() < 48 {
        return Err(CtParseError::TooShort);
    }
    let timestamp = u64::from_be_bytes(data[0..8].try_into().unwrap());
    let tree_size = u64::from_be_bytes(data[8..16].try_into().unwrap());
    let mut root_hash = [0u8; 32];
    root_hash.copy_from_slice(&data[16..48]);
    Ok(MerkleTreeHead {
        timestamp,
        tree_size,
        root_hash,
    })
}

/// Serialize a MTH to bytes.
pub fn serialize_merkle_tree_head(mth: &MerkleTreeHead) -> Vec<u8> {
    let mut out = Vec::with_capacity(48);
    out.extend_from_slice(&mth.timestamp.to_be_bytes());
    out.extend_from_slice(&mth.tree_size.to_be_bytes());
    out.extend_from_slice(&mth.root_hash);
    out
}

/// Watchlist entry for CT monitoring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchEntry {
    /// Human-readable label
    pub label: String,
    /// JA4 fingerprint prefix to match (e.g., "t13d1516h2")
    pub ja4_prefix: String,
    /// Maximum number of allowed matches before alerting
    pub threshold: u32,
}

/// Alert emitted when a CT certificate matches a watchlist entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtAlert {
    /// Watchlist entry that matched
    pub watch_entry: WatchEntry,
    /// Matching certificate
    pub entry: CtLogEntry,
    /// Current match count for this watch entry
    pub match_count: u32,
    /// Timestamp of the alert
    pub alert_timestamp: u64,
}

/// CT Monitor — watches multiple logs and matches against a watchlist.
#[derive(Clone, Debug)]
pub struct CtMonitor {
    /// Configured CT log URLs
    pub logs: Vec<String>,
    /// Watchlist entries
    pub watchlist: Vec<WatchEntry>,
    /// Latest MTH per log URL
    pub mths: HashMap<String, MerkleTreeHead>,
    /// Match counters per watchlist label
    pub match_counts: HashMap<String, u32>,
}

impl Default for CtMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl CtMonitor {
    /// Create a new CT monitor.
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            watchlist: Vec::new(),
            mths: HashMap::new(),
            match_counts: HashMap::new(),
        }
    }

    /// Add a CT log to monitor.
    pub fn add_log(&mut self, url: String) {
        self.logs.push(url);
    }

    /// Add a watchlist entry.
    pub fn add_watch(&mut self, entry: WatchEntry) {
        self.match_counts.entry(entry.label.clone()).or_insert(0);
        self.watchlist.push(entry);
    }

    /// Check a certificate against the watchlist.
    ///
    /// For X.509 certificates, we derive a pseudo-JA4 fingerprint from the
    /// SAN pattern: `x509_{san_count}_{first_san_hash}`.
    pub fn check_certificate(&mut self, entry: &CtLogEntry) -> Vec<CtAlert> {
        let mut alerts = Vec::new();

        // Derive a pseudo-JA4 fingerprint from the certificate
        let derived_ja4 = derive_ja4_from_cert(entry);

        for watch in &self.watchlist {
            if derived_ja4.starts_with(&watch.ja4_prefix) {
                let count = self.match_counts.entry(watch.label.clone()).or_insert(0);
                *count += 1;

                if *count == watch.threshold {
                    let alert = CtAlert {
                        watch_entry: watch.clone(),
                        entry: entry.clone(),
                        match_count: *count,
                        alert_timestamp: current_timestamp_ms(),
                    };
                    alerts.push(alert);
                }
            }
        }

        alerts
    }

    /// Get the current MTH for a log.
    pub fn get_mth(&self, log_url: &str) -> Option<&MerkleTreeHead> {
        self.mths.get(log_url)
    }

    /// Update the MTH for a log.
    pub fn update_mth(&mut self, log_url: String, mth: MerkleTreeHead) {
        self.mths.insert(log_url, mth);
    }

    /// Check if a new MTH indicates new entries.
    pub fn has_new_entries(&self, log_url: &str, new_mth: &MerkleTreeHead) -> bool {
        match self.mths.get(log_url) {
            Some(old) => new_mth.tree_size > old.tree_size,
            None => true, // Never seen this log before
        }
    }
}

/// Derive a pseudo-JA4 fingerprint from an X.509 certificate.
///
/// Format: `x509_{san_count}_{san_hash}`
pub fn derive_ja4_from_cert(entry: &CtLogEntry) -> String {
    use sha2::{Digest, Sha256};

    let san_count = entry.sans.len();
    let mut hasher = Sha256::new();
    for san in &entry.sans {
        hasher.update(san.as_bytes());
    }
    let hash = hasher.finalize();
    let hash_hex: String = hash[..6].iter().map(|b| format!("{:02x}", b)).collect();

    format!("x509_{:02}_{}", san_count, hash_hex)
}

/// Get current timestamp in milliseconds since epoch.
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtParseError {
    TooShort,
    InvalidTag,
    Truncated,
    InvalidUtf8,
    InvalidTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_merkle_tree_head() {
        let mth = MerkleTreeHead {
            timestamp: 1693000000000,
            tree_size: 1000000,
            root_hash: [0xAB; 32],
        };
        let bytes = serialize_merkle_tree_head(&mth);
        assert_eq!(bytes.len(), 48);
        let parsed = parse_merkle_tree_head(&bytes).unwrap();
        assert_eq!(parsed.timestamp, 1693000000000);
        assert_eq!(parsed.tree_size, 1000000);
        assert_eq!(parsed.root_hash, [0xAB; 32]);
    }

    #[test]
    fn test_parse_mth_too_short() {
        assert_eq!(
            parse_merkle_tree_head(&[0u8; 47]),
            Err(CtParseError::TooShort)
        );
    }

    #[test]
    fn test_parse_x509_basic() {
        // Test with a minimal DER that triggers a parse error (not a real cert)
        let short_der = [0x30, 0x05, 0x02, 0x01, 0x00];
        let result = parse_x509_basic(&short_der);
        // The cert is too short to parse fully, should get an error
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_x509_too_short() {
        assert_eq!(parse_x509_basic(&[0x00]), Err(CtParseError::TooShort));
    }

    #[test]
    fn test_parse_x509_invalid_tag() {
        let data = vec![0xFF; 40]; // starts with invalid tag 0xFF
        assert_eq!(parse_x509_basic(&data), Err(CtParseError::InvalidTag));
    }

    #[test]
    fn test_derive_ja4_from_cert() {
        let entry = CtLogEntry {
            timestamp: 0,
            entry_type: CtEntryType::X509,
            serial_number: vec![0x01],
            issuer_dn: vec![],
            sans: vec!["example.com".to_string(), "www.example.com".to_string()],
            not_before: 0,
            not_after: 0,
            cert_der: vec![],
        };
        let ja4 = derive_ja4_from_cert(&entry);
        assert!(ja4.starts_with("x509_02_"));
        // Format: "x509_02_" (8) + 12 hex chars = 20
        assert_eq!(ja4.len(), 20);
    }

    #[test]
    fn test_ct_monitor_check_match() {
        let mut monitor = CtMonitor::new();
        monitor.add_watch(WatchEntry {
            label: "suspicious_ca".to_string(),
            ja4_prefix: "x509_01_".to_string(),
            threshold: 1,
        });

        let entry = CtLogEntry {
            timestamp: 0,
            entry_type: CtEntryType::X509,
            serial_number: vec![],
            issuer_dn: vec![],
            sans: vec!["bad.example.com".to_string()],
            not_before: 0,
            not_after: 0,
            cert_der: vec![],
        };

        let alerts = monitor.check_certificate(&entry);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].match_count, 1);
    }

    #[test]
    fn test_ct_monitor_no_match() {
        let mut monitor = CtMonitor::new();
        monitor.add_watch(WatchEntry {
            label: "suspicious_ca".to_string(),
            ja4_prefix: "x509_99_".to_string(),
            threshold: 1,
        });

        let entry = CtLogEntry {
            timestamp: 0,
            entry_type: CtEntryType::X509,
            serial_number: vec![],
            issuer_dn: vec![],
            sans: vec!["safe.example.com".to_string()],
            not_before: 0,
            not_after: 0,
            cert_der: vec![],
        };

        let alerts = monitor.check_certificate(&entry);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_ct_monitor_threshold() {
        let mut monitor = CtMonitor::new();
        monitor.add_watch(WatchEntry {
            label: "rare_cert".to_string(),
            ja4_prefix: "x509_01_".to_string(),
            threshold: 3,
        });

        let entry = CtLogEntry {
            timestamp: 0,
            entry_type: CtEntryType::X509,
            serial_number: vec![],
            issuer_dn: vec![],
            sans: vec!["target.com".to_string()],
            not_before: 0,
            not_after: 0,
            cert_der: vec![],
        };

        // First 2 checks: no alert (below threshold)
        assert!(monitor.check_certificate(&entry).is_empty());
        assert!(monitor.check_certificate(&entry).is_empty());
        // Third check: alert
        let alerts = monitor.check_certificate(&entry);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].match_count, 3);
    }

    #[test]
    fn test_has_new_entries() {
        let mut monitor = CtMonitor::new();
        let url = "https://ct.googleapis.com/logs/argon2024/".to_string();

        let mth1 = MerkleTreeHead {
            timestamp: 1000,
            tree_size: 100,
            root_hash: [0x01; 32],
        };
        assert!(monitor.has_new_entries(&url, &mth1));

        monitor.update_mth(url.clone(), mth1.clone());
        assert!(!monitor.has_new_entries(&url, &mth1));

        let mth2 = MerkleTreeHead {
            timestamp: 2000,
            tree_size: 110,
            root_hash: [0x02; 32],
        };
        assert!(monitor.has_new_entries(&url, &mth2));
    }

    #[test]
    fn test_ct_entry_type_roundtrip() {
        assert_eq!(CtEntryType::from_u16(0), Some(CtEntryType::X509));
        assert_eq!(CtEntryType::from_u16(1), Some(CtEntryType::Precert));
        assert_eq!(CtEntryType::from_u16(2), None);
    }

    #[test]
    fn test_parse_asn1_length_short() {
        let data = [0x05]; // length 5
        let (len, pos) = parse_asn1_length(&data, 0).unwrap();
        assert_eq!(len, 5);
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_parse_asn1_length_long() {
        let data = [0x82, 0x01, 0x00]; // length 256
        let (len, pos) = parse_asn1_length(&data, 0).unwrap();
        assert_eq!(len, 256);
        assert_eq!(pos, 3);
    }

    #[test]
    fn test_parse_asn1_length_truncated() {
        let data = [0x82]; // incomplete
        assert_eq!(parse_asn1_length(&data, 0), Err(CtParseError::Truncated));
    }
}
