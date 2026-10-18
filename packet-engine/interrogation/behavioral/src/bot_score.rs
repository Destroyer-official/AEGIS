//! JA4 Bot Score computation (§6.4).
//!
//! JA4 fingerprints can be augmented with a "bot score" metric that
//! quantifies the likelihood a client is automated rather than human.
//! This is derived from TLS handshake anomalies, HTTP header ordering,
//! and request cadence patterns.
//!
//! Bot scores enable the scanner to distinguish between real user-facing
//! services and automated infrastructure (load balancers, CDNs,
//! monitoring agents).
//!
//! # Score Range
//!
//! - **0.0–0.3**: Likely human client (browser)
//! - **0.3–0.7**: Ambiguous (could be automation framework)
//! - **0.7–1.0**: Likely automated (bot/scraper/load balancer)

use std::collections::HashMap;

// ─── TLS Anomaly Types ──────────────────────────────────────────────────────

/// TLS handshake anomalies that indicate automated client behavior.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TlsAnomaly {
    /// Client sent no SNI extension
    MissingSni,
    /// Cipher suite ordering differs from known browser profiles
    UnusualCipherOrder,
    /// ALPN contains non-browser protocols (e.g., only grpc, no h2)
    NonBrowserAlpn,
    /// Request interval is suspiciously regular (< 10ms jitter)
    RapidCadence,
    /// Extension set doesn't match any known browser profile
    SuspiciousExtensions,
    /// TLS version is outdated (pre-TLS 1.2)
    OutdatedTlsVersion,
    /// Client offers compression (rare in modern browsers)
    CompressionExtension,
    /// JA4 fingerprint matches known bot/scraper patterns
    KnownBotFingerprint,
}

impl std::fmt::Display for TlsAnomaly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSni => write!(f, "missing_sni"),
            Self::UnusualCipherOrder => write!(f, "unusual_cipher_order"),
            Self::NonBrowserAlpn => write!(f, "non_browser_alpn"),
            Self::RapidCadence => write!(f, "rapid_cadence"),
            Self::SuspiciousExtensions => write!(f, "suspicious_extensions"),
            Self::OutdatedTlsVersion => write!(f, "outdated_tls_version"),
            Self::CompressionExtension => write!(f, "compression_extension"),
            Self::KnownBotFingerprint => write!(f, "known_bot_fingerprint"),
        }
    }
}

// ─── Bot Score ───────────────────────────────────────────────────────────────

/// Bot score result computed from TLS and HTTP behavioral analysis.
#[derive(Clone, Debug)]
pub struct BotScore {
    /// Composite bot score (0.0 = human, 1.0 = definitely automated)
    pub score: f64,
    /// Detected anomalies contributing to the score
    pub anomalies: Vec<TlsAnomaly>,
    /// Whether the client is classified as automated
    pub is_automated: bool,
    /// Breakdown of score by anomaly category
    pub score_breakdown: HashMap<String, f64>,
}

impl BotScore {
    /// Create a zero-score (human) result.
    pub fn human() -> Self {
        Self {
            score: 0.0,
            anomalies: Vec::new(),
            is_automated: false,
            score_breakdown: HashMap::new(),
        }
    }
}

// ─── Known Browser JA4 Fingerprints ─────────────────────────────────────────

/// Known Chrome JA4 cipher suite ordering (TLS 1.3).
const CHROME_CIPHERS: &[u16] = &[
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
    0xc02b, // ECDHE_ECDSA_AES_128_GCM_SHA256
    0xc02f, // ECDHE_RSA_AES_128_GCM_SHA256
    0xc02c, // ECDHE_ECDSA_AES_256_GCM_SHA384
    0xc030, // ECDHE_RSA_AES_256_GCM_SHA384
    0xcca9, // ECDHE_ECDSA_CHACHA20_POLY1305
    0xcca8, // ECDHE_RSA_CHACHA20_POLY1305
];

/// Known Firefox JA4 cipher suite ordering (TLS 1.3).
const FIREFOX_CIPHERS: &[u16] = &[
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0xc02b, // ECDHE_ECDSA_AES_128_GCM_SHA256
    0xc02f, // ECDHE_RSA_AES_128_GCM_SHA256
    0xcca9, // ECDHE_ECDSA_CHACHA20_POLY1305
    0xcca8, // ECDHE_RSA_CHACHA20_POLY1305
    0xc02c, // ECDHE_ECDSA_AES_256_GCM_SHA384
    0xc030, // ECDHE_RSA_AES_256_GCM_SHA384
];

// ─── Score Computation ──────────────────────────────────────────────────────

/// Compute a bot score from JA4 fingerprint fields and behavioral data.
///
/// This is a heuristic-based scorer using simple rules. The actual
/// scoring would use a trained ML model in production, but this
/// provides a reasonable baseline for identifying obvious bots.
///
/// # Arguments
/// * `cipher_suite_order` — Ordered list of cipher suites from the Client Hello
/// * `extension_types` — Set of extension types present
/// * `sni_present` — Whether SNI was provided
/// * `alpn_protocols` — ALPN protocol list
/// * `tls_version` — Negotiated TLS version
/// * `cadence_ms` — Average interval between requests in milliseconds (0 if unknown)
pub fn compute_bot_score(
    cipher_suite_order: &[u16],
    extension_types: &[u16],
    sni_present: bool,
    alpn_protocols: &[String],
    tls_version: u16,
    cadence_ms: u64,
) -> BotScore {
    let mut anomalies = Vec::new();
    let mut breakdown = HashMap::new();

    // Check 1: Missing SNI
    if !sni_present {
        anomalies.push(TlsAnomaly::MissingSni);
        breakdown.insert("missing_sni".into(), 0.15);
    }

    // Check 2: Unusual cipher ordering
    if !cipher_suite_order.is_empty() {
        let is_chrome = cipher_suite_order == CHROME_CIPHERS;
        let is_firefox = cipher_suite_order == FIREFOX_CIPHERS;
        if !is_chrome && !is_firefox && cipher_suite_order.len() >= 4 {
            anomalies.push(TlsAnomaly::UnusualCipherOrder);
            breakdown.insert("unusual_cipher_order".into(), 0.20);
        }
    }

    // Check 3: Non-browser ALPN
    if !alpn_protocols.is_empty() {
        let has_h2 = alpn_protocols.iter().any(|p| p == "h2");
        let has_http11 = alpn_protocols.iter().any(|p| p == "http/1.1");
        if !has_h2 && !has_http11 {
            anomalies.push(TlsAnomaly::NonBrowserAlpn);
            breakdown.insert("non_browser_alpn".into(), 0.25);
        }
    }

    // Check 4: Rapid cadence (very regular timing)
    if cadence_ms > 0 && cadence_ms < 10 {
        anomalies.push(TlsAnomaly::RapidCadence);
        breakdown.insert("rapid_cadence".into(), 0.30);
    }

    // Check 5: Suspicious extensions (too few or too many)
    if extension_types.len() < 5 || extension_types.len() > 30 {
        anomalies.push(TlsAnomaly::SuspiciousExtensions);
        breakdown.insert("suspicious_extensions".into(), 0.10);
    }

    // Check 6: Outdated TLS version
    if tls_version < 0x0303 {
        anomalies.push(TlsAnomaly::OutdatedTlsVersion);
        breakdown.insert("outdated_tls_version".into(), 0.15);
    }

    // Check 7: Compression extension present (0x000d is sig_alg, 0x0015 is padding)
    // Compression is extension 0x000a — very rare in modern browsers
    if extension_types.contains(&0x000a) {
        anomalies.push(TlsAnomaly::CompressionExtension);
        breakdown.insert("compression_extension".into(), 0.10);
    }

    // Compute composite score (capped at 1.0)
    let raw_score: f64 = breakdown.values().sum();
    let score = raw_score.min(1.0);
    let is_automated = score >= 0.7;

    BotScore {
        score,
        anomalies,
        is_automated,
        score_breakdown: breakdown,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_score() {
        let bs = BotScore::human();
        assert_eq!(bs.score, 0.0);
        assert!(!bs.is_automated);
        assert!(bs.anomalies.is_empty());
    }

    #[test]
    fn test_chrome_like_client_low_score() {
        let bs = compute_bot_score(
            CHROME_CIPHERS,
            &[0x0000, 0x000d, 0x0010, 0x002b, 0x0033], // SNI, sig_alg, ALPN, supported_versions, key_share
            true,
            &["h2".into(), "http/1.1".into()],
            0x0303,
            0, // unknown cadence
        );
        // Chrome-like client should have a low bot score
        assert!(
            bs.score < 0.3,
            "Chrome-like client score {} should be < 0.3",
            bs.score
        );
        assert!(!bs.is_automated);
    }

    #[test]
    fn test_bot_like_client_high_score() {
        let bs = compute_bot_score(
            &[0x00ff, 0x0005, 0x000a], // unusual cipher order
            &[0x000a],                 // only compression extension
            false,                     // no SNI
            &["grpc".into()],          // non-browser ALPN
            0x0301,                    // TLS 1.0 (outdated)
            5,                         // very rapid cadence
        );
        // Bot-like client should have a high score
        assert!(
            bs.score >= 0.7,
            "Bot-like client score {} should be >= 0.7",
            bs.score
        );
        assert!(bs.is_automated);
        assert!(bs.anomalies.len() >= 4);
    }

    #[test]
    fn test_anomaly_display() {
        assert_eq!(TlsAnomaly::MissingSni.to_string(), "missing_sni");
        assert_eq!(TlsAnomaly::RapidCadence.to_string(), "rapid_cadence");
        assert_eq!(
            TlsAnomaly::KnownBotFingerprint.to_string(),
            "known_bot_fingerprint"
        );
    }

    #[test]
    fn test_score_capped_at_one() {
        // Multiple anomalies should not exceed 1.0
        let bs = compute_bot_score(&[], &[], false, &[], 0x0301, 1);
        assert!(bs.score <= 1.0);
    }

    #[test]
    fn test_breakdown_populated() {
        let bs = compute_bot_score(CHROME_CIPHERS, &[0x0000], false, &["h2".into()], 0x0303, 0);
        // Missing SNI should be in breakdown
        assert!(bs.score_breakdown.contains_key("missing_sni"));
    }
}
