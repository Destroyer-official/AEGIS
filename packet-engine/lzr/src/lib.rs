use std::cmp::Reverse;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use serde::Deserialize;

pub mod dfa_serde;
pub mod hot_reload;
pub mod service;
pub mod signature_store;
pub mod stats_server;

/// LZR protocol category (§6.2).
///
/// **Stability**: Stable since v1.0.0. New variants may be added in minor versions.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum Category {
    Web,
    Mail,
    Remote,
    Database,
    Messaging,
    Streaming,
    Iot,
    Ai,
    Generic,
    Proxy,
    Cloud,
    Banner,
    Tls,
    Misc,
    Dns,
    Ntp,
    Snmp,
    Rdp,
    Vnc,
    Sip,
    Ike,
    Openvpn,
    Wireguard,
}

impl std::str::FromStr for Category {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(Category::Web),
            "mail" => Ok(Category::Mail),
            "remote" => Ok(Category::Remote),
            "database" => Ok(Category::Database),
            "messaging" => Ok(Category::Messaging),
            "streaming" => Ok(Category::Streaming),
            "iot" => Ok(Category::Iot),
            "ai" => Ok(Category::Ai),
            "generic" => Ok(Category::Generic),
            "proxy" => Ok(Category::Proxy),
            "cloud" => Ok(Category::Cloud),
            "banner" => Ok(Category::Banner),
            "tls" => Ok(Category::Tls),
            "misc" => Ok(Category::Misc),
            "dns" => Ok(Category::Dns),
            "ntp" => Ok(Category::Ntp),
            "snmp" => Ok(Category::Snmp),
            "rdp" => Ok(Category::Rdp),
            "vnc" | "vnc-rfb" => Ok(Category::Vnc),
            "sip" => Ok(Category::Sip),
            "ike" => Ok(Category::Ike),
            "openvpn" => Ok(Category::Openvpn),
            "wireguard" => Ok(Category::Wireguard),
            _ => Err(()),
        }
    }
}

/// Protocol signature definition from CSV.
///
/// **Stability**: Stable since v1.0.0. Fields will not be removed.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct Signature {
    /// Category name (maps to `Category` enum).
    pub category: String,
    /// Protocol identifier (e.g., "HTTP/1.1", "SSH", "MySQL").
    pub protocol: String,
    /// Regex pattern anchored at start (`^...`).
    pub signature: String,
    /// Priority — higher wins ties (default 0..100).
    pub priority: u32,
}

/// LZR classification result.
///
/// **Stability**: Stable since v1.0.0. Fields will not be removed or reordered.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct LzrMatch {
    /// Matched protocol name (e.g., "HTTP/1.1", "SSH").
    pub protocol: String,
    /// Category of matched protocol.
    pub category: String,
    /// Priority of the matching signature.
    pub priority: u32,
    /// `true` if a signature matched at position 0.
    pub matched: bool,
}

pub(crate) struct AcSignature {
    pub category: String,
    pub protocol: String,
    pub pattern: Vec<u8>,
    pub priority: u32,
}

pub(crate) struct RegexSignature {
    pub category: String,
    pub protocol: String,
    pub regex: regex::Regex,
    pub priority: u32,
}

/// LZR engine: Aho-Corasick automaton for literal prefix patterns,
/// regex fallback for complex patterns.
///
/// # Architecture
///
/// Signatures from the CSV are split into two tiers:
///   - **AC tier**: patterns that extract to pure literal bytes → compiled
///     into a single Aho-Corasick automaton. Scanning is O(n) in payload
///     length.
///   - **Regex tier**: patterns with regex metacharacters (`+`, `*`, `[`,
///     `(`, `\d`, etc.) → compiled individually via the `regex` crate.
///
/// During classification, the AC automaton runs first (fastest path).
/// Matches from both tiers are combined and the highest-priority result is returned.
///
/// # Performance
///
/// - **Classify latency**: 1.3–2.4 µs per payload (well under 10 ms target)
/// - **Throughput**: ~400K classifications/second on single core
/// - **Memory**: ~200 KB for 206-signature automaton
///
/// # Example
///
/// ```rust
/// use aegis_lzr::LzrEngine;
///
/// let engine = LzrEngine::new();
/// let result = engine.classify(b"HTTP/1.1 200 OK\r\n\r\n");
/// assert!(result.matched);
/// assert_eq!(result.protocol, "HTTP/1.1");
/// ```
///
/// **Stability**: Stable since v1.0.0. Public API will not break in 1.x.
#[non_exhaustive]
pub struct LzrEngine {
    ac: AhoCorasick,
    ac_sigs: Vec<AcSignature>,
    regex_sigs: Vec<RegexSignature>,
    total_count: usize,
}

impl LzrEngine {
    /// Create engine with 200+ embedded signatures (default).
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn new() -> Self {
        Self::from_embedded_csv().expect("embedded CSV should always parse")
    }

    /// Create engine with custom signatures.
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn with_signatures(signatures: Vec<Signature>) -> Self {
        Self::build(signatures)
    }

    /// Load signatures from CSV string.
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn from_csv(csv_data: &str) -> Result<Self, LzrError> {
        let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
        let mut signatures = Vec::new();
        for result in rdr.deserialize() {
            let sig: Signature = result.map_err(|e| LzrError::CsvError(e.to_string()))?;
            signatures.push(sig);
        }
        Ok(Self::build(signatures))
    }

    /// Load signatures from CSV file path.
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn from_csv_file(path: &str) -> Result<Self, LzrError> {
        let csv_data = std::fs::read_to_string(path)?;
        Self::from_csv(&csv_data)
    }

    /// Load signatures from embedded CSV (445+ signatures).
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn from_embedded_csv() -> Result<Self, LzrError> {
        Self::from_csv(include_str!("../data/signatures.csv"))
    }

    /// Rebuild engine from a DFA snapshot (used by `dfa_serde::deserialize_dfa`).
    ///
    /// **Stability**: Unstable — internal use only.
    pub(crate) fn with_signatures_from_snapshot(
        snapshot: &dfa_serde::DfaSnapshot,
    ) -> Result<Self, LzrError> {
        let mut ac_sigs = Vec::new();
        let mut regex_sigs = Vec::new();

        for (cat, proto, pattern, prio) in &snapshot.ac_sigs {
            ac_sigs.push(AcSignature {
                category: cat.clone(),
                protocol: proto.clone(),
                pattern: pattern.clone(),
                priority: *prio,
            });
        }

        for (cat, proto, regex_str, prio) in &snapshot.regex_sigs {
            if let Ok(re) = regex::Regex::new(regex_str) {
                regex_sigs.push(RegexSignature {
                    category: cat.clone(),
                    protocol: proto.clone(),
                    regex: re,
                    priority: *prio,
                })
            }
        }

        ac_sigs.sort_by_key(|s| Reverse(s.priority));
        regex_sigs.sort_by_key(|s| Reverse(s.priority));

        let patterns: Vec<Vec<u8>> = ac_sigs.iter().map(|s| s.pattern.clone()).collect();
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&patterns)
            .map_err(|e| LzrError::IoError(std::io::Error::other(e)))?;

        let total_count = snapshot.total_count;

        Ok(Self {
            ac,
            ac_sigs,
            regex_sigs,
            total_count,
        })
    }

    /// Classify payload — return best (highest-priority) match at position 0.
    ///
    /// Returns `LzrMatch { matched: false, protocol: "unknown", ... }` if no signature matches.
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn classify(&self, payload: &[u8]) -> LzrMatch {
        let mut best: Option<LzrMatch> = None;

        for mat in self.ac.find_iter(payload) {
            if mat.start() == 0 {
                let sig = &self.ac_sigs[mat.pattern()];
                match &best {
                    None => {
                        best = Some(LzrMatch {
                            protocol: sig.protocol.clone(),
                            category: sig.category.clone(),
                            priority: sig.priority,
                            matched: true,
                        });
                    }
                    Some(ref b) if sig.priority > b.priority => {
                        best = Some(LzrMatch {
                            protocol: sig.protocol.clone(),
                            category: sig.category.clone(),
                            priority: sig.priority,
                            matched: true,
                        });
                    }
                    _ => {}
                }
            }
        }

        let payload_str = String::from_utf8_lossy(payload);
        for sig in &self.regex_sigs {
            if sig.regex.is_match(&payload_str) {
                match &best {
                    None => {
                        best = Some(LzrMatch {
                            protocol: sig.protocol.clone(),
                            category: sig.category.clone(),
                            priority: sig.priority,
                            matched: true,
                        });
                    }
                    Some(ref b) if sig.priority > b.priority => {
                        best = Some(LzrMatch {
                            protocol: sig.protocol.clone(),
                            category: sig.category.clone(),
                            priority: sig.priority,
                            matched: true,
                        });
                    }
                    _ => {}
                }
            }
        }

        best.unwrap_or_else(|| LzrMatch {
            protocol: "unknown".to_string(),
            category: "unknown".to_string(),
            priority: 0,
            matched: false,
        })
    }

    /// Classify and return all matches at position 0, sorted by priority (highest first).
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn classify_all(&self, payload: &[u8]) -> Vec<LzrMatch> {
        let mut matches: Vec<LzrMatch> = self
            .ac
            .find_iter(payload)
            .filter(|m| m.start() == 0)
            .map(|m| {
                let sig = &self.ac_sigs[m.pattern()];
                LzrMatch {
                    protocol: sig.protocol.clone(),
                    category: sig.category.clone(),
                    priority: sig.priority,
                    matched: true,
                }
            })
            .collect();

        let payload_str = String::from_utf8_lossy(payload);
        for sig in &self.regex_sigs {
            if sig.regex.is_match(&payload_str) {
                matches.push(LzrMatch {
                    protocol: sig.protocol.clone(),
                    category: sig.category.clone(),
                    priority: sig.priority,
                    matched: true,
                });
            }
        }

        matches.sort_by_key(|m| Reverse(m.priority));
        matches
    }

    /// Return total number of loaded signatures (both AC + regex tiers).
    ///
    /// **Stability**: Stable since v1.0.0.
    pub fn signature_count(&self) -> usize {
        self.total_count
    }

    fn build(signatures: Vec<Signature>) -> Self {
        let total_count = signatures.len();
        let mut ac_sigs = Vec::new();
        let mut regex_sigs = Vec::new();

        for sig in signatures {
            if let Some(pat) = extract_literal_prefix(&sig.signature) {
                ac_sigs.push(AcSignature {
                    category: sig.category,
                    protocol: sig.protocol,
                    pattern: pat,
                    priority: sig.priority,
                });
            } else {
                if let Ok(re) = regex::Regex::new(&sig.signature) {
                    regex_sigs.push(RegexSignature {
                        category: sig.category,
                        protocol: sig.protocol,
                        regex: re,
                        priority: sig.priority,
                    })
                }
            }
        }

        ac_sigs.sort_by_key(|s| Reverse(s.priority));
        regex_sigs.sort_by_key(|s| Reverse(s.priority));

        let patterns: Vec<Vec<u8>> = ac_sigs.iter().map(|s| s.pattern.clone()).collect();
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&patterns)
            .expect("AhoCorasick build should not fail");

        Self {
            ac,
            ac_sigs,
            regex_sigs,
            total_count,
        }
    }
}

impl Default for LzrEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract literal byte pattern from a regex string.
///
/// Returns `Some(bytes)` if the regex is composed entirely of literal bytes
/// (after converting known escapes), or `None` if it contains regex
/// metacharacters that require a full regex engine.
fn extract_literal_prefix(regex: &str) -> Option<Vec<u8>> {
    let mut chars = regex.chars().peekable();
    if chars.peek() == Some(&'^') {
        chars.next();
    }

    let mut out = Vec::new();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let esc = chars.next()?;
                match esc {
                    'r' => out.push(0x0D),
                    'n' => out.push(0x0A),
                    't' => out.push(0x09),
                    's' => out.push(b' '),
                    'x' => {
                        let hi = chars.next()?;
                        let lo = chars.next()?;
                        let byte = u8::from_str_radix(&format!("{}{}", hi, lo), 16).ok()?;
                        out.push(byte);
                    }
                    '\\' => out.push(b'\\'),
                    '.' => out.push(b'.'),
                    'd' | 'D' | 'w' | 'W' | 'S' | 'b' | 'B' | 'p' | 'P' => return None,
                    _ => out.push(esc as u8),
                }
            }
            '+' | '*' | '?' | '[' | ']' | '(' | ')' | '{' | '}' | '$' | '^' => return None,
            '.' => return None,
            _ => out.push(c as u8),
        }
    }

    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// Errors from LZR operations.
///
/// **Stability**: Stable since v1.0.0.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LzrError {
    #[error("CSV parse error: {0}")]
    CsvError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "HTTP/1.1");
        assert_eq!(result.category, "web");
    }

    #[test]
    fn test_ssh_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "SSH");
        assert_eq!(result.category, "remote");
    }

    #[test]
    fn test_smtp_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(b"220 mail.example.com ESMTP Postfix (Ubuntu)\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "SMTP");
        assert_eq!(result.category, "mail");
    }

    #[test]
    fn test_ollama_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(br#"{"version": "0.1.14"}"#);
        assert!(result.matched);
        assert_eq!(result.protocol, "Ollama");
        assert_eq!(result.category, "ai");
    }

    #[test]
    fn test_mysql_detection() {
        let engine = LzrEngine::new();
        let payload: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x0a, 0x38, 0x2e, 0x30, 0x2e, 0x33, 0x33, 0x00,
        ];
        let result = engine.classify(payload);
        assert!(result.matched);
        assert_eq!(result.protocol, "MySQL");
        assert_eq!(result.category, "database");
    }

    #[test]
    fn test_grpc_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        assert!(result.matched);
        assert!(result.protocol.contains("gRPC") || result.protocol.contains("HTTP/2"));
    }

    #[test]
    fn test_unknown_detection() {
        let engine = LzrEngine::new();
        let result = engine.classify(b"RANDOM DATA THAT DOES NOT MATCH");
        assert!(!result.matched);
        assert_eq!(result.protocol, "unknown");
    }

    #[test]
    fn test_priority_order() {
        let engine = LzrEngine::new();
        let matches = engine.classify_all(b"HTTP/1.1 200 OK\r\n");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].protocol, "HTTP/1.1");
    }

    #[test]
    fn test_embedded_csv_loads_all_signatures() {
        let engine = LzrEngine::from_embedded_csv().unwrap();
        assert!(
            engine.signature_count() >= 400,
            "Expected >=400 signatures, got {}",
            engine.signature_count()
        );
    }

    #[test]
    fn test_ac_patterns_are_literal_bytes() {
        let patterns: Vec<Vec<u8>> = vec![
            b"HTTP/1.0 ".to_vec(),
            b"HTTP/1.1 ".to_vec(),
            b"SSH-2.0-".to_vec(),
            b"220 ".to_vec(),
            b"\x00\x00\x00\x00\x0a".to_vec(),
        ];
        let ac = AhoCorasick::new(&patterns).unwrap();

        assert!(ac.find(b"HTTP/1.1 200 OK").is_some());
        assert!(ac.find(b"SSH-2.0-OpenSSH_8.9").is_some());
        assert!(ac.find(b"220 mail.example.com ESMTP").is_some());
        assert!(ac.find(&[0x00, 0x00, 0x00, 0x00, 0x0a, 0x38]).is_some());
        assert!(ac.find(b"GARBAGE DATA").is_none());
    }

    #[test]
    fn test_extract_literal_prefix_basic() {
        assert_eq!(
            extract_literal_prefix("^HTTP/1\\.1\\s"),
            Some(b"HTTP/1.1 ".to_vec())
        );
        assert_eq!(
            extract_literal_prefix("^SSH-2\\.0-"),
            Some(b"SSH-2.0-".to_vec())
        );
        assert_eq!(extract_literal_prefix("^220\\s"), Some(b"220 ".to_vec()));
    }

    #[test]
    fn test_extract_literal_prefix_hex() {
        assert_eq!(
            extract_literal_prefix("^\\x00\\x00\\x00\\x00\\x0a"),
            Some(vec![0x00, 0x00, 0x00, 0x00, 0x0a])
        );
    }

    #[test]
    fn test_extract_literal_prefix_complex_returns_none() {
        assert_eq!(extract_literal_prefix("^[0-9]+\\s"), None);
        assert_eq!(extract_literal_prefix("^foo.*bar"), None);
        assert_eq!(extract_literal_prefix("^a(b|c)d"), None);
    }

    #[test]
    fn test_extract_literal_prefix_slash_dot() {
        assert_eq!(extract_literal_prefix("^RFB\\s003\\.\\d\\d\\n"), None);
    }

    #[test]
    fn test_extract_literal_prefix_regex_classes_return_none() {
        assert_eq!(extract_literal_prefix("^\\d+"), None);
        assert_eq!(extract_literal_prefix("^\\w+"), None);
        assert_eq!(extract_literal_prefix("^\\s*foo"), None);
    }
}
