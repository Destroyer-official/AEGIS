//! CVE correlation pipeline for AEGIS intelligence enrichment.
//! §3.5.2 — Vulnerability Correlation Pipeline
//!
//! Pipeline: CPE string extraction → NVD CVE cache lookup → CVSS v3.1 scoring
//!
//! Enriched records include CVE identifiers, CVSS v3.1 severity scores,
//! affected version ranges, and exploit availability indicators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod stream;

// ───────── Core data model ─────────

/// Common Platform Enumeration (CPE) 2.3 formatted string.
///
/// Format: `cpe:2.3:part:vendor:product:version:update:edition:language:sw_edition:target_sw:target_hw:other`
///
/// Example: `cpe:2.3:a:apache:http_server:2.4.51:*:*:*:*:*:*:*`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct CpeString(pub String);

impl CpeString {
    /// Parse a CPE 2.3 formatted string.
    pub fn parse(raw: &str) -> Result<Self, CveError> {
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() != 13 {
            return Err(CveError::InvalidCpe(format!(
                "Expected 13 colon-separated fields, got {}",
                parts.len()
            )));
        }
        if parts[0] != "cpe" || parts[1] != "2.3" {
            return Err(CveError::InvalidCpe(format!(
                "CPE must start with 'cpe:2.3:', got '{}:{}'",
                parts[0], parts[1]
            )));
        }
        Ok(CpeString(raw.to_string()))
    }

    /// Create a default/empty CpeString.
    pub fn empty() -> Self {
        CpeString("cpe:2.3:a:unknown:*:*:*:*:*:*:*:*:*:*".to_string())
    }

    /// Extract vendor field.
    pub fn vendor(&self) -> &str {
        self.0.split(':').nth(3).unwrap_or("")
    }

    /// Extract product field.
    pub fn product(&self) -> &str {
        self.0.split(':').nth(4).unwrap_or("")
    }

    /// Extract version field.
    pub fn version(&self) -> &str {
        self.0.split(':').nth(5).unwrap_or("")
    }

    /// Convert wildcard fields to regex pattern for matching.
    pub fn to_match_pattern(&self) -> String {
        self.0
            .split(':')
            .map(|part| if part == "*" { ".*" } else { part })
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl std::fmt::Display for CpeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single CVE vulnerability record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveRecord {
    /// CVE identifier (e.g., "CVE-2024-12345").
    pub cve_id: String,
    /// CVSS v3.1 base score (0.0–10.0).
    pub cvss_score: f64,
    /// CVSS v3.1 severity rating.
    pub severity: CvssSeverity,
    /// CVSS v3.1 vector string.
    pub cvss_vector: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Affected CPE strings.
    pub affected_cpes: Vec<CpeString>,
    /// Known exploit available (from NVD CISA KEV or EPSS).
    pub exploit_available: bool,
    /// Publication date.
    pub published: Option<DateTime<Utc>>,
    /// Last modified date.
    pub last_modified: Option<DateTime<Utc>>,
}

/// CVSS v3.1 severity rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CvssSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl CvssSeverity {
    /// Map CVSS score to severity rating per CVSS v3.1 spec.
    pub fn from_score(score: f64) -> Self {
        match score {
            0.0 => CvssSeverity::None,
            0.1..=3.9 => CvssSeverity::Low,
            4.0..=6.9 => CvssSeverity::Medium,
            7.0..=8.9 => CvssSeverity::High,
            9.0..=10.0 => CvssSeverity::Critical,
            _ => CvssSeverity::None,
        }
    }
}

impl std::fmt::Display for CvssSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CvssSeverity::None => write!(f, "NONE"),
            CvssSeverity::Low => write!(f, "LOW"),
            CvssSeverity::Medium => write!(f, "MEDIUM"),
            CvssSeverity::High => write!(f, "HIGH"),
            CvssSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Enrichment result: CPE + all matching CVEs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveEnrichment {
    /// The CPE string that was looked up.
    pub cpe: CpeString,
    /// Matching CVE records sorted by severity (highest first).
    pub vulnerabilities: Vec<CveRecord>,
    /// Total number of matches (may exceed `vulnerabilities` if capped).
    pub total_matches: usize,
    /// When this enrichment was computed.
    pub computed_at: DateTime<Utc>,
}

/// Input for CVE correlation.
#[derive(Debug, Clone)]
pub struct CveQuery {
    /// CPE string to look up.
    pub cpe: CpeString,
    /// Maximum number of CVE records to return (0 = unlimited).
    pub max_results: usize,
    /// Minimum CVSS score to include.
    pub min_cvss: f64,
}

/// Errors from the CVE pipeline.
#[derive(Debug, thiserror::Error)]
pub enum CveError {
    #[error("Invalid CPE string: {0}")]
    InvalidCpe(String),

    #[error("NVD API request failed: {0}")]
    NvdApiError(String),

    #[error("NVD cache miss: {0}")]
    CacheMiss(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ───────── CPE Extraction ─────────

/// CPE extractor: extracts CPE strings from service banners, certificates,
/// and HTTP responses.
pub struct CpeExtractor {
    /// Regex patterns for known product → CPE mappings.
    product_patterns: Vec<ProductPattern>,
}

/// A pattern mapping banner content to a CPE string.
#[derive(Debug, Clone)]
struct ProductPattern {
    /// Regex to match against banner content.
    regex: regex::Regex,
    /// CPE 2.3 template with capture group references.
    cpe_template: String,
    /// Confidence score for this extraction (0.0–1.0).
    confidence: f64,
}

impl Default for CpeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl CpeExtractor {
    pub fn new() -> Self {
        let mut extractor = Self {
            product_patterns: Vec::new(),
        };
        extractor.load_builtin_patterns();
        extractor
    }

    /// Load built-in product detection patterns.
    fn load_builtin_patterns(&mut self) {
        // Common HTTP server patterns
        let patterns: Vec<(&str, &str, f64)> = vec![
            (
                r"Apache/(\d+\.\d+\.\d+)",
                "cpe:2.3:a:apache:http_server:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
            (
                r"nginx/(\d+\.\d+\.\d+)",
                "cpe:2.3:a:nginx:nginx:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
            (
                r"OpenSSH[_ ](\d+\.\d+)",
                "cpe:2.3:a:openbsd:openssh:{version}:*:*:*:*:*:*:*",
                0.85,
            ),
            (
                r"Microsoft-IIS/(\d+\.\d+)",
                "cpe:2.3:a:microsoft:iis:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
            (
                r"PHP/(\d+\.\d+\.\d+)",
                "cpe:2.3:a:php:php:{version}:*:*:*:*:*:*:*",
                0.85,
            ),
            (
                r"MySQL[\s/](\d+\.\d+\.\d+)",
                "cpe:2.3:a:mysql:mysql:{version}:*:*:*:*:*:*:*",
                0.85,
            ),
            (
                r"PostgreSQL(\d+\.\d+)",
                "cpe:2.3:a:postgresql:postgresql:{version}:*:*:*:*:*:*:*",
                0.85,
            ),
            (
                r"Redis v(\d+\.\d+\.\d+)",
                "cpe:2.3:a:redis:redis:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
            (
                r"MongoDB[\s/](\d+\.\d+\.\d+)",
                "cpe:2.3:a:mongodb:mongodb:{version}:*:*:*:*:*:*:*",
                0.85,
            ),
            (
                r"ProFTPD (\d+\.\d+\.\d+)",
                "cpe:2.3:a:proftpd:proftpd:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
            (
                r"vsFTPd (\d+\.\d+\.\d+)",
                "cpe:2.3:a:vsftpd:vsftpd:{version}:*:*:*:*:*:*:*",
                0.9,
            ),
        ];

        for (pattern, template, confidence) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                self.product_patterns.push(ProductPattern {
                    regex: re,
                    cpe_template: template.to_string(),
                    confidence,
                });
            }
        }
    }

    /// Extract CPE strings from a service banner.
    pub fn extract_from_banner(&self, banner: &str) -> Vec<(CpeString, f64)> {
        let mut results = Vec::new();
        for pattern in &self.product_patterns {
            if let Some(caps) = pattern.regex.captures(banner) {
                let version = caps.get(1).map(|m| m.as_str()).unwrap_or("*");
                let cpe_str = pattern.cpe_template.replace("{version}", version);
                results.push((CpeString(cpe_str), pattern.confidence));
            }
        }
        results
    }

    /// Extract CPE from a TLS certificate's subject/issuer fields.
    pub fn extract_from_tls_cert(&self, subject: &str, issuer: &str) -> Vec<(CpeString, f64)> {
        let mut results = Vec::new();
        // Check for known CA products
        let combined = format!("{} {}", subject, issuer);

        // Common TLS stack indicators
        let tls_patterns: Vec<(&str, &str, f64)> = vec![
            (
                r"Let's Encrypt",
                "cpe:2.3:a:letsencrypt:lets_encrypt:*:*:*:*:*:*:*:*",
                0.7,
            ),
            (
                r"DigiCert",
                "cpe:2.3:a:digicert:digicert:*:*:*:*:*:*:*:*",
                0.6,
            ),
            (
                r"Cloudflare",
                "cpe:2.3:a:cloudflare:cloudflare:*:*:*:*:*:*:*:*",
                0.6,
            ),
        ];

        for (pattern, template, confidence) in tls_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&combined) {
                    results.push((CpeString(template.to_string()), confidence));
                }
            }
        }

        results
    }

    /// Extract CPE from an HTTP Server header.
    pub fn extract_from_http_headers(
        &self,
        headers: &HashMap<String, String>,
    ) -> Vec<(CpeString, f64)> {
        let mut results = Vec::new();
        if let Some(server) = headers.get("server").or_else(|| headers.get("Server")) {
            results.extend(self.extract_from_banner(server));
        }
        if let Some(powered_by) = headers
            .get("x-powered-by")
            .or_else(|| headers.get("X-Powered-By"))
        {
            results.extend(self.extract_from_banner(powered_by));
        }
        results
    }
}

// ───────── NVD Cache ─────────

/// Local NVD CVE cache with scheduled refresh.
///
/// Maintains an in-memory index of CVE records keyed by CPE product+version
/// for fast local lookups without NVD API rate limits.
pub struct NvdCache {
    /// CVE records indexed by (vendor, product, version_prefix).
    records: std::sync::RwLock<HashMap<(String, String, String), Vec<CveRecord>>>,
    /// CPE → CVE ID mapping for cross-referencing.
    cpe_to_cve: std::sync::RwLock<HashMap<CpeString, Vec<String>>>,
    /// When the cache was last refreshed.
    last_refresh: std::sync::RwLock<Option<DateTime<Utc>>>,
    /// Cache TTL in seconds.
    refresh_ttl_secs: u64,
}

impl NvdCache {
    pub fn new(refresh_ttl_secs: u64) -> Self {
        Self {
            records: std::sync::RwLock::new(HashMap::new()),
            cpe_to_cve: std::sync::RwLock::new(HashMap::new()),
            last_refresh: std::sync::RwLock::new(None),
            refresh_ttl_secs,
        }
    }

    /// Insert a CVE record into the cache.
    pub fn insert(&self, record: CveRecord) -> Result<(), CveError> {
        let mut cache = self.records.write().unwrap();
        let mut cpe_index = self.cpe_to_cve.write().unwrap();

        for cpe in &record.affected_cpes {
            let key = (
                cpe.vendor().to_string(),
                cpe.product().to_string(),
                cpe.version().to_string(),
            );
            cache.entry(key).or_default().push(record.clone());

            cpe_index
                .entry(cpe.clone())
                .or_default()
                .push(record.cve_id.clone());
        }

        Ok(())
    }

    /// Check if cache needs refresh.
    pub fn needs_refresh(&self) -> bool {
        let last = self.last_refresh.read().unwrap();
        match *last {
            Some(t) => {
                let elapsed = Utc::now().signed_duration_since(t);
                elapsed.num_seconds() as u64 >= self.refresh_ttl_secs
            }
            None => true,
        }
    }

    /// Refresh the cache from the NVD API 2.0.
    ///
    /// Uses the NVD REST API with optional API key for rate limiting.
    /// Downloads CVEs modified since last refresh and updates the index.
    pub async fn refresh(&self) -> Result<(), CveError> {
        // TODO: Implement NVD API 2.0 integration
        // 1. GET https://services.nvd.nist.gov/rest/json/cves/2.0
        //    with lastModStartDate/lastModEndDate window
        // 2. Parse CVE items, extract CPE match criteria
        // 3. Index by (vendor, product, version_prefix)
        // 4. Update last_refresh timestamp
        tracing::info!("NVD cache refresh (stub)");
        *self.last_refresh.write().unwrap() = Some(Utc::now());
        Ok(())
    }

    /// Load cache from a local JSON snapshot file.
    pub fn load_from_file(&self, path: &str) -> Result<(), CveError> {
        let data = std::fs::read_to_string(path)?;
        let records: Vec<CveRecord> =
            serde_json::from_str(&data).map_err(|e| CveError::JsonParse(e.to_string()))?;

        let mut cache = self.records.write().unwrap();
        let mut cpe_index = self.cpe_to_cve.write().unwrap();

        for record in records {
            // Index by all affected CPEs, not just the first
            for cpe in &record.affected_cpes {
                let key = (
                    cpe.vendor().to_string(),
                    cpe.product().to_string(),
                    cpe.version().to_string(),
                );
                cache.entry(key).or_default().push(record.clone());

                cpe_index
                    .entry(cpe.clone())
                    .or_default()
                    .push(record.cve_id.clone());
            }
            // Also index wildcard version for each CPE
            for cpe in &record.affected_cpes {
                let key = (
                    cpe.vendor().to_string(),
                    cpe.product().to_string(),
                    "*".to_string(),
                );
                // Avoid duplicates in the wildcard bucket
                let entries = cache.entry(key).or_default();
                if !entries.iter().any(|e| e.cve_id == record.cve_id) {
                    entries.push(record.clone());
                }
            }
        }

        *self.last_refresh.write().unwrap() = Some(Utc::now());
        Ok(())
    }

    /// Query the cache for CVEs matching a CPE.
    pub fn query(&self, cpe: &CpeString, min_cvss: f64) -> Vec<CveRecord> {
        let cache = self.records.read().unwrap();
        let vendor = cpe.vendor();
        let product = cpe.product();
        let version = cpe.version();

        // Exact match first
        let mut results: Vec<CveRecord> = cache
            .get(&(vendor.to_string(), product.to_string(), version.to_string()))
            .cloned()
            .unwrap_or_default();

        // Wildcard version match (vendor:product:*:*:...)
        results.extend(
            cache
                .get(&(vendor.to_string(), product.to_string(), "*".to_string()))
                .cloned()
                .unwrap_or_default(),
        );

        // Filter by minimum CVSS
        results.retain(|r| r.cvss_score >= min_cvss);

        // Sort by CVSS descending
        results.sort_by(|a, b| b.cvss_score.partial_cmp(&a.cvss_score).unwrap());

        results
    }

    /// Export cache to a local JSON snapshot.
    pub fn export_to_file(&self, path: &str) -> Result<(), CveError> {
        let cache = self.records.read().unwrap();
        // Deduplicate by CVE ID since the same record is stored under multiple CPE keys
        let mut seen = std::collections::HashSet::new();
        let all_records: Vec<&CveRecord> = cache
            .values()
            .flat_map(|v| v.iter())
            .filter(|r| seen.insert(r.cve_id.clone()))
            .collect();
        let json = serde_json::to_string_pretty(&all_records)
            .map_err(|e| CveError::JsonParse(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

// ───────── CVSS v3.1 Scoring ─────────

/// CVSS v3.1 vector components for local scoring.
#[derive(Debug, Clone, Default)]
pub struct Cvss31Vector {
    pub attack_vector: Option<CvssAv>,
    pub attack_complexity: Option<CvssAc>,
    pub privileges_required: Option<CvssPr>,
    pub user_interaction: Option<CvssUi>,
    pub scope: Option<CvssS>,
    pub confidentiality: Option<CvssCia>,
    pub integrity: Option<CvssCia>,
    pub availability: Option<CvssCia>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssAv {
    Network,
    Adjacent,
    Local,
    Physical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssAc {
    Low,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssPr {
    None,
    Low,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssUi {
    None,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssS {
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CvssCia {
    None,
    Low,
    High,
}

impl Cvss31Vector {
    /// Parse a CVSS v3.1 vector string (e.g., "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H").
    pub fn parse(vector: &str) -> Result<Self, CveError> {
        let stripped = vector
            .trim_start_matches("CVSS:3.1/")
            .trim_start_matches("CVSS:3.0/");
        let mut v = Self::default();

        for part in stripped.split('/') {
            let mut kv = part.splitn(2, ':');
            let key = kv.next().unwrap_or("");
            let val = kv.next().unwrap_or("");
            match key {
                "AV" => v.attack_vector = Some(parse_av(val)),
                "AC" => v.attack_complexity = Some(parse_ac(val)),
                "PR" => v.privileges_required = Some(parse_pr(val)),
                "UI" => v.user_interaction = Some(parse_ui(val)),
                "S" => v.scope = Some(parse_scope(val)),
                "C" => v.confidentiality = Some(parse_cia(val)),
                "I" => v.integrity = Some(parse_cia(val)),
                "A" => v.availability = Some(parse_cia(val)),
                _ => {}
            }
        }

        Ok(v)
    }

    /// Compute the CVSS v3.1 base score from vector components.
    ///
    /// Implements the base metric formula per FIRST CVSS v3.1 spec.
    pub fn base_score(&self) -> f64 {
        let av = metric_value_av(self.attack_vector);
        let ac = metric_value_ac(self.attack_complexity);
        let pr = metric_value_pr(self.privileges_required, self.scope);
        let ui = metric_value_ui(self.user_interaction);
        let s = self.scope.unwrap_or(CvssS::Unchanged);
        let c = metric_value_cia(self.confidentiality);
        let i = metric_value_cia(self.integrity);
        let a = metric_value_cia(self.availability);

        let exploitability = 8.22 * av * ac * pr * ui;

        let isc_base = 1.0 - ((1.0 - c) * (1.0 - i) * (1.0 - a));

        let impact = match s {
            CvssS::Unchanged => 6.42 * isc_base,
            CvssS::Changed => {
                if isc_base <= 0.0 {
                    0.0
                } else {
                    7.52 * (isc_base - 0.029) - 3.25 * (isc_base * 0.9731 - 0.02).powi(13)
                }
            }
        };

        if impact <= 0.0 {
            0.0
        } else {
            let raw = (exploitability + impact).min(10.0);
            // Round up to nearest 0.1
            (raw * 10.0).ceil() / 10.0
        }
    }

    /// Derive severity rating from computed base score.
    pub fn severity(&self) -> CvssSeverity {
        CvssSeverity::from_score(self.base_score())
    }
}

fn parse_av(s: &str) -> CvssAv {
    match s {
        "N" => CvssAv::Network,
        "A" => CvssAv::Adjacent,
        "L" => CvssAv::Local,
        "P" => CvssAv::Physical,
        _ => CvssAv::Network,
    }
}

fn parse_ac(s: &str) -> CvssAc {
    match s {
        "L" => CvssAc::Low,
        "H" => CvssAc::High,
        _ => CvssAc::Low,
    }
}

fn parse_pr(s: &str) -> CvssPr {
    match s {
        "N" => CvssPr::None,
        "L" => CvssPr::Low,
        "H" => CvssPr::High,
        _ => CvssPr::None,
    }
}

fn parse_ui(s: &str) -> CvssUi {
    match s {
        "N" => CvssUi::None,
        "R" => CvssUi::Required,
        _ => CvssUi::None,
    }
}

fn parse_scope(s: &str) -> CvssS {
    match s {
        "U" => CvssS::Unchanged,
        "C" => CvssS::Changed,
        _ => CvssS::Unchanged,
    }
}

fn parse_cia(s: &str) -> CvssCia {
    match s {
        "N" => CvssCia::None,
        "L" => CvssCia::Low,
        "H" => CvssCia::High,
        _ => CvssCia::None,
    }
}

fn metric_value_av(av: Option<CvssAv>) -> f64 {
    match av {
        Some(CvssAv::Network) => 0.85,
        Some(CvssAv::Adjacent) => 0.62,
        Some(CvssAv::Local) => 0.55,
        Some(CvssAv::Physical) => 0.20,
        None => 0.85,
    }
}

fn metric_value_ac(ac: Option<CvssAc>) -> f64 {
    match ac {
        Some(CvssAc::Low) => 0.77,
        Some(CvssAc::High) => 0.44,
        None => 0.77,
    }
}

fn metric_value_pr(pr: Option<CvssPr>, scope: Option<CvssS>) -> f64 {
    let is_changed = matches!(scope, Some(CvssS::Changed));
    match pr {
        Some(CvssPr::None) => 0.85,
        Some(CvssPr::Low) => {
            if is_changed {
                0.68
            } else {
                0.62
            }
        }
        Some(CvssPr::High) => {
            if is_changed {
                0.50
            } else {
                0.27
            }
        }
        None => 0.85,
    }
}

fn metric_value_ui(ui: Option<CvssUi>) -> f64 {
    match ui {
        Some(CvssUi::None) => 0.85,
        Some(CvssUi::Required) => 0.62,
        None => 0.85,
    }
}

fn metric_value_cia(cia: Option<CvssCia>) -> f64 {
    match cia {
        Some(CvssCia::None) => 0.00,
        Some(CvssCia::Low) => 0.22,
        Some(CvssCia::High) => 0.56,
        None => 0.00,
    }
}

// ───────── CVE Correlator (pipeline orchestrator) ─────────

/// CVE correlation engine: orchestrates CPE extraction, NVD lookup, and scoring.
pub struct CveCorrelator {
    extractor: CpeExtractor,
    cache: NvdCache,
}

impl CveCorrelator {
    pub fn new(cache_ttl_secs: u64) -> Self {
        Self {
            extractor: CpeExtractor::new(),
            cache: NvdCache::new(cache_ttl_secs),
        }
    }

    /// Get a reference to the CPE extractor.
    pub fn extractor(&self) -> &CpeExtractor {
        &self.extractor
    }

    /// Get a reference to the NVD cache.
    pub fn cache(&self) -> &NvdCache {
        &self.cache
    }

    /// Full enrichment pipeline: extract CPEs from banner, query cache, return CVEs.
    pub fn enrich_from_banner(
        &self,
        banner: &str,
        min_cvss: f64,
        max_results: usize,
    ) -> Vec<CveEnrichment> {
        let cpes = self.extractor.extract_from_banner(banner);

        cpes.into_iter()
            .map(|(cpe, _confidence)| {
                let mut vulns = self.cache.query(&cpe, min_cvss);
                let total = vulns.len();
                if max_results > 0 {
                    vulns.truncate(max_results);
                }
                CveEnrichment {
                    cpe,
                    vulnerabilities: vulns,
                    total_matches: total,
                    computed_at: Utc::now(),
                }
            })
            .collect()
    }

    /// Enrich from HTTP headers.
    pub fn enrich_from_http_headers(
        &self,
        headers: &HashMap<String, String>,
        min_cvss: f64,
        max_results: usize,
    ) -> Vec<CveEnrichment> {
        let cpes = self.extractor.extract_from_http_headers(headers);

        cpes.into_iter()
            .map(|(cpe, _confidence)| {
                let mut vulns = self.cache.query(&cpe, min_cvss);
                let total = vulns.len();
                if max_results > 0 {
                    vulns.truncate(max_results);
                }
                CveEnrichment {
                    cpe,
                    vulnerabilities: vulns,
                    total_matches: total,
                    computed_at: Utc::now(),
                }
            })
            .collect()
    }

    /// Direct CPE lookup against the cache.
    pub fn enrich_cpe(&self, query: &CveQuery) -> CveEnrichment {
        let mut vulns = self.cache.query(&query.cpe, query.min_cvss);
        let total = vulns.len();
        if query.max_results > 0 {
            vulns.truncate(query.max_results);
        }
        CveEnrichment {
            cpe: query.cpe.clone(),
            vulnerabilities: vulns,
            total_matches: total,
            computed_at: Utc::now(),
        }
    }
}

// ───────── Tests ─────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpe_parse_valid() {
        let cpe = CpeString::parse("cpe:2.3:a:apache:http_server:2.4.51:*:*:*:*:*:*:*").unwrap();
        assert_eq!(cpe.vendor(), "apache");
        assert_eq!(cpe.product(), "http_server");
        assert_eq!(cpe.version(), "2.4.51");
    }

    #[test]
    fn test_cpe_parse_invalid() {
        assert!(CpeString::parse("not-a-cpe").is_err());
        assert!(CpeString::parse("cpe:2.0:too:few").is_err());
    }

    #[test]
    fn test_cpe_display() {
        let cpe = CpeString("cpe:2.3:a:nginx:nginx:1.21.0:*:*:*:*:*:*:*".to_string());
        assert_eq!(
            cpe.to_string(),
            "cpe:2.3:a:nginx:nginx:1.21.0:*:*:*:*:*:*:*"
        );
    }

    #[test]
    fn test_cvss_severity_from_score() {
        assert_eq!(CvssSeverity::from_score(0.0), CvssSeverity::None);
        assert_eq!(CvssSeverity::from_score(2.5), CvssSeverity::Low);
        assert_eq!(CvssSeverity::from_score(5.0), CvssSeverity::Medium);
        assert_eq!(CvssSeverity::from_score(8.0), CvssSeverity::High);
        assert_eq!(CvssSeverity::from_score(9.5), CvssSeverity::Critical);
    }

    #[test]
    fn test_cvss_vector_parse() {
        let v = Cvss31Vector::parse("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H").unwrap();
        assert_eq!(v.attack_vector, Some(CvssAv::Network));
        assert_eq!(v.attack_complexity, Some(CvssAc::Low));
        assert_eq!(v.privileges_required, Some(CvssPr::None));
        assert_eq!(v.user_interaction, Some(CvssUi::None));
        assert_eq!(v.scope, Some(CvssS::Unchanged));
        assert_eq!(v.confidentiality, Some(CvssCia::High));
        assert_eq!(v.integrity, Some(CvssCia::High));
        assert_eq!(v.availability, Some(CvssCia::High));
    }

    #[test]
    fn test_cvss_base_score_critical() {
        // AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H → 9.8 CRITICAL
        let v = Cvss31Vector::parse("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H").unwrap();
        let score = v.base_score();
        assert!((9.0..=10.0).contains(&score), "Expected 9.8, got {score}");
        assert_eq!(v.severity(), CvssSeverity::Critical);
    }

    #[test]
    fn test_cvss_base_score_medium() {
        // AV:N/AC:L/PR:L/UI:N/S:U/C:L/I:L/A:N → 5.4 MEDIUM
        let v = Cvss31Vector::parse("CVSS:3.1/AV:N/AC:L/PR:L/UI:N/S:U/C:L/I:L/A:N").unwrap();
        let score = v.base_score();
        assert!((5.0..=6.0).contains(&score), "Expected 5.4, got {score}");
        assert_eq!(v.severity(), CvssSeverity::Medium);
    }

    #[test]
    fn test_extract_from_banner() {
        let extractor = CpeExtractor::new();
        let results = extractor.extract_from_banner("Apache/2.4.51 (Ubuntu)");
        assert!(!results.is_empty());
        assert!(results[0].0 .0.contains("apache"));
        assert!(results[0].0 .0.contains("2.4.51"));
    }

    #[test]
    fn test_extract_nginx() {
        let extractor = CpeExtractor::new();
        let results = extractor.extract_from_banner("nginx/1.21.4");
        assert!(!results.is_empty());
        assert!(results[0].0 .0.contains("nginx"));
    }

    #[test]
    fn test_extract_openssh() {
        let extractor = CpeExtractor::new();
        let results = extractor.extract_from_banner("SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1");
        assert!(!results.is_empty());
        assert!(results[0].0 .0.contains("openssh"));
    }

    #[test]
    fn test_nvd_cache_query_empty() {
        let cache = NvdCache::new(3600);
        let cpe = CpeString::parse("cpe:2.3:a:apache:http_server:2.4.51:*:*:*:*:*:*:*").unwrap();
        let results = cache.query(&cpe, 0.0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_nvd_cache_needs_refresh() {
        let cache = NvdCache::new(3600);
        assert!(cache.needs_refresh());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(CvssSeverity::Critical.to_string(), "CRITICAL");
        assert_eq!(CvssSeverity::High.to_string(), "HIGH");
        assert_eq!(CvssSeverity::Medium.to_string(), "MEDIUM");
        assert_eq!(CvssSeverity::Low.to_string(), "LOW");
        assert_eq!(CvssSeverity::None.to_string(), "NONE");
    }
}
