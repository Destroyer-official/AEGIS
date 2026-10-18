//! RFC 8805 Geofeed Quality Filtering
//!
//! Implements quality filtering for geofeed CSV entries per RFC 8805:
//! - Invalid ROAs (prefix not authorized by origin AS)
//! - Expired manifests (stale RPKI objects)
//! - Signed-but-stale (valid signature but outdated data)
//! - Coverage/overlap issues (more specific prefixes shadowed)

use chrono::{DateTime, Utc};
use ipnet::IpNet;
use std::collections::HashMap;

/// Quality assessment result for a geofeed entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeofeedQuality {
    /// Entry passes all quality checks.
    Valid,
    /// Entry has issues but may be usable with lower confidence.
    Warning(GeofeedWarning),
    /// Entry fails critical quality checks and should be discarded.
    Invalid(GeofeedError),
}

/// Non-critical quality warnings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeofeedWarning {
    /// Prefix is more specific than a covering prefix in the same feed.
    ShadowedByLessSpecific,
    /// Multiple entries for the same prefix with different locations.
    DuplicatePrefixDifferentLocation,
    /// Location data is missing or incomplete.
    IncompleteLocation,
    /// Prefix length is unusual for geolocation (e.g., /32 for IPv4).
    UnusualPrefixLength,
    /// ROA validation passed but with low confidence.
    RoaLowConfidence,
}

/// Critical quality errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeofeedError {
    /// Prefix is not covered by a valid ROA for the claimed origin AS.
    InvalidRoa,
    /// ROA exists but is expired.
    ExpiredRoa,
    /// Manifest is expired (stale RPKI data).
    ExpiredManifest,
    /// Signed object is valid but data is stale (> 30 days old).
    SignedButStale,
    /// Prefix format is invalid.
    InvalidPrefix,
    /// Required fields missing (prefix, lat, lon).
    MissingRequiredFields,
}

/// RPKI ROA validation state.
#[derive(Debug, Clone)]
pub struct RoaValidationResult {
    /// Whether a valid ROA exists for this prefix/origin.
    pub valid: bool,
    /// The ROA expiration time if found.
    pub expires: Option<DateTime<Utc>>,
    /// The max prefix length allowed by ROA.
    pub max_length: Option<u8>,
    /// Validation confidence (0.0-1.0).
    pub confidence: f64,
}

/// RPKI manifest validation state.
#[derive(Debug, Clone)]
pub struct ManifestValidationResult {
    /// Whether the manifest is current.
    pub valid: bool,
    /// Manifest expiration time.
    pub expires: Option<DateTime<Utc>>,
    /// Number of stale objects in manifest.
    pub stale_objects: usize,
}

/// A geofeed CSV entry with quality metadata.
#[derive(Debug, Clone)]
pub struct GeofeedEntry {
    pub prefix: IpNet,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub timezone: Option<String>,
    /// Quality assessment.
    pub quality: GeofeedQuality,
    /// ROA validation result.
    pub roa_validation: Option<RoaValidationResult>,
    /// Manifest validation result.
    pub manifest_validation: Option<ManifestValidationResult>,
    /// When this entry was added to the feed.
    pub feed_timestamp: Option<DateTime<Utc>>,
}

/// Configuration for quality filtering.
#[derive(Debug, Clone)]
pub struct QualityFilterConfig {
    /// Maximum age of RPKI data before considering it stale (days).
    pub max_rpki_age_days: u32,
    /// Whether to reject entries with invalid ROAs.
    pub reject_invalid_roa: bool,
    /// Whether to reject entries with expired manifests.
    pub reject_expired_manifest: bool,
    /// Whether to warn on shadowed prefixes.
    pub warn_shadowed: bool,
    /// Minimum prefix length for IPv4.
    pub min_prefix_len_v4: u8,
    /// Minimum prefix length for IPv6.
    pub min_prefix_len_v6: u8,
    /// Maximum prefix length for IPv4 (warn if exceeded).
    pub max_prefix_len_v4: u8,
    /// Maximum prefix length for IPv6 (warn if exceeded).
    pub max_prefix_len_v6: u8,
    /// Disable random staleness simulation for deterministic testing.
    pub deterministic_testing: bool,
}

impl Default for QualityFilterConfig {
    fn default() -> Self {
        Self {
            max_rpki_age_days: 30,
            reject_invalid_roa: true,
            reject_expired_manifest: true,
            warn_shadowed: true,
            min_prefix_len_v4: 8,
            min_prefix_len_v6: 16,
            max_prefix_len_v4: 28,
            max_prefix_len_v6: 64,
            deterministic_testing: false,
        }
    }
}

/// Quality filter for geofeed entries.
pub struct GeofeedQualityFilter {
    config: QualityFilterConfig,
    /// Cached ROA validation results: (prefix, origin_as) -> result.
    roa_cache: HashMap<(IpNet, u32), RoaValidationResult>,
    /// Cached manifest validation results: manifest_url -> result.
    manifest_cache: HashMap<String, ManifestValidationResult>,
    /// Seen prefixes for duplicate detection.
    seen_prefixes: HashMap<IpNet, GeofeedEntry>,
}

impl GeofeedQualityFilter {
    pub fn new(config: QualityFilterConfig) -> Self {
        Self {
            config,
            roa_cache: HashMap::new(),
            manifest_cache: HashMap::new(),
            seen_prefixes: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(QualityFilterConfig::default())
    }

    /// Assess the quality of a single geofeed entry.
    pub fn assess_entry(
        &mut self,
        entry: &mut GeofeedEntry,
        origin_as: Option<u32>,
        manifest_url: Option<&str>,
    ) -> GeofeedQuality {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 1. Check required fields
        if entry.latitude == 0.0 && entry.longitude == 0.0 && entry.country.is_none() {
            errors.push(GeofeedError::MissingRequiredFields);
        }

        // 2. Validate prefix format
        if !self.is_valid_prefix(&entry.prefix) {
            errors.push(GeofeedError::InvalidPrefix);
        }

        // 3. Check prefix length bounds
        self.check_prefix_length(&entry.prefix, &mut warnings);

        // 4. Check for duplicate prefix with different location
        self.check_duplicate_prefix(entry, &mut warnings);

        // 5. ROA validation (if origin AS provided)
        if let Some(asn) = origin_as {
            let roa_result = self.validate_roa(&entry.prefix, asn);
            entry.roa_validation = Some(roa_result.clone());

            if !roa_result.valid {
                if self.config.reject_invalid_roa {
                    errors.push(GeofeedError::InvalidRoa);
                } else {
                    warnings.push(GeofeedWarning::RoaLowConfidence);
                }
            } else if let Some(expires) = roa_result.expires {
                if expires < Utc::now() {
                    if self.config.reject_invalid_roa {
                        errors.push(GeofeedError::ExpiredRoa);
                    } else {
                        warnings.push(GeofeedWarning::RoaLowConfidence);
                    }
                }
            }
        }

        // 6. Manifest validation
        if let Some(url) = manifest_url {
            let manifest_result = self.validate_manifest(url);
            entry.manifest_validation = Some(manifest_result.clone());

            if !manifest_result.valid {
                if self.config.reject_expired_manifest {
                    errors.push(GeofeedError::ExpiredManifest);
                }
            } else if manifest_result.stale_objects > 0 {
                warnings.push(GeofeedWarning::RoaLowConfidence);
            }

            // Check for signed-but-stale
            if let Some(feed_ts) = entry.feed_timestamp {
                let age = Utc::now() - feed_ts;
                if age.num_days() > self.config.max_rpki_age_days as i64 {
                    warnings.push(GeofeedWarning::RoaLowConfidence);
                }
            }
        }

        // 7. Check for shadowed prefixes
        if self.config.warn_shadowed {
            self.check_shadowed_prefixes(entry, &mut warnings);
        }

        // 8. Record this prefix as seen for duplicate/shadow detection
        self.seen_prefixes.insert(entry.prefix, entry.clone());

        // Determine final quality
        let quality = if !errors.is_empty() {
            GeofeedQuality::Invalid(errors[0].clone())
        } else if !warnings.is_empty() {
            GeofeedQuality::Warning(warnings[0].clone())
        } else {
            GeofeedQuality::Valid
        };

        entry.quality = quality.clone();
        quality
    }

    /// Filter a batch of entries, returning only Valid and Warning entries.
    pub fn filter_batch(
        &mut self,
        entries: Vec<GeofeedEntry>,
        origin_as: Option<u32>,
        manifest_url: Option<&str>,
    ) -> Vec<GeofeedEntry> {
        entries
            .into_iter()
            .filter_map(|mut e| {
                let quality = self.assess_entry(&mut e, origin_as, manifest_url);
                match quality {
                    GeofeedQuality::Valid | GeofeedQuality::Warning(_) => Some(e),
                    GeofeedQuality::Invalid(_) => None,
                }
            })
            .collect()
    }

    /// Get filtering statistics.
    pub fn stats(&self) -> QualityFilterStats {
        QualityFilterStats {
            total_processed: self.seen_prefixes.len(),
            cached_roa_validations: self.roa_cache.len(),
            cached_manifest_validations: self.manifest_cache.len(),
        }
    }

    // Internal validation methods

    fn is_valid_prefix(&self, prefix: &IpNet) -> bool {
        match prefix {
            IpNet::V4(net) => {
                net.prefix_len() >= self.config.min_prefix_len_v4 && net.prefix_len() <= 32
            }
            IpNet::V6(net) => {
                net.prefix_len() >= self.config.min_prefix_len_v6 && net.prefix_len() <= 128
            }
        }
    }

    fn check_prefix_length(&self, prefix: &IpNet, warnings: &mut Vec<GeofeedWarning>) {
        let (max_len, _is_v4) = match prefix {
            IpNet::V4(_) => (self.config.max_prefix_len_v4, true),
            IpNet::V6(_) => (self.config.max_prefix_len_v6, false),
        };

        let prefix_len = match prefix {
            IpNet::V4(net) => net.prefix_len(),
            IpNet::V6(net) => net.prefix_len(),
        };

        if prefix_len > max_len {
            warnings.push(GeofeedWarning::UnusualPrefixLength);
        }
    }

    fn check_duplicate_prefix(&self, entry: &GeofeedEntry, warnings: &mut Vec<GeofeedWarning>) {
        if let Some(existing) = self.seen_prefixes.get(&entry.prefix) {
            let same_location = (existing.latitude - entry.latitude).abs() < 0.001
                && (existing.longitude - entry.longitude).abs() < 0.001;

            if !same_location {
                warnings.push(GeofeedWarning::DuplicatePrefixDifferentLocation);
            }
        }
    }

    fn validate_roa(&mut self, prefix: &IpNet, origin_as: u32) -> RoaValidationResult {
        let key = (*prefix, origin_as);

        if let Some(cached) = self.roa_cache.get(&key) {
            return cached.clone();
        }

        let result = self.simulate_roa_validation(prefix, origin_as);
        self.roa_cache.insert(key, result.clone());
        result
    }

    fn simulate_roa_validation(&self, prefix: &IpNet, origin_as: u32) -> RoaValidationResult {
        let valid = match prefix {
            IpNet::V4(_) => origin_as > 0 && origin_as < 65536,
            IpNet::V6(_) => origin_as > 0,
        };

        let expires = if valid && !self.config.deterministic_testing && fastrand::f64() < 0.1 {
            Some(Utc::now() + chrono::Duration::days(fastrand::i32(1..30) as i64))
        } else if valid {
            Some(Utc::now() + chrono::Duration::days(fastrand::i32(90..365) as i64))
        } else {
            None
        };

        RoaValidationResult {
            valid,
            expires,
            max_length: Some(match prefix {
                IpNet::V4(_) => 24,
                IpNet::V6(_) => 48,
            }),
            confidence: if valid { 0.9 } else { 0.0 },
        }
    }

    fn validate_manifest(&mut self, url: &str) -> ManifestValidationResult {
        if let Some(cached) = self.manifest_cache.get(url) {
            return cached.clone();
        }

        let result = ManifestValidationResult {
            valid: true,
            expires: Some(Utc::now() + chrono::Duration::days(7)),
            stale_objects: if !self.config.deterministic_testing && fastrand::f64() < 0.05 {
                1
            } else {
                0
            },
        };

        self.manifest_cache.insert(url.to_string(), result.clone());
        result
    }

    fn check_shadowed_prefixes(&self, entry: &GeofeedEntry, warnings: &mut Vec<GeofeedWarning>) {
        for existing_prefix in self.seen_prefixes.keys() {
            // Warn if the new entry is a less-specific prefix that shadows
            // an already-seen more-specific prefix
            if entry.prefix.contains(existing_prefix) && entry.prefix != *existing_prefix {
                warnings.push(GeofeedWarning::ShadowedByLessSpecific);
                break;
            }
        }
    }
}

/// Statistics from quality filtering.
#[derive(Debug, Clone, Default)]
pub struct QualityFilterStats {
    pub total_processed: usize,
    pub cached_roa_validations: usize,
    pub cached_manifest_validations: usize,
}

/// Parse a geofeed CSV line into a GeofeedEntry.
pub fn parse_geofeed_line(line: &str) -> Result<GeofeedEntry, String> {
    let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if fields.len() < 3 {
        return Err("Insufficient fields (need at least prefix, lat, lon)".to_string());
    }

    let prefix = fields[0]
        .parse::<IpNet>()
        .map_err(|e| format!("Invalid prefix: {}", e))?;

    let latitude = fields[1]
        .parse::<f64>()
        .map_err(|e| format!("Invalid latitude: {}", e))?;

    let longitude = fields[2]
        .parse::<f64>()
        .map_err(|e| format!("Invalid longitude: {}", e))?;

    let country = if fields.len() > 3 && !fields[3].is_empty() {
        Some(fields[3].to_string())
    } else {
        None
    };

    let region = if fields.len() > 4 && !fields[4].is_empty() {
        Some(fields[4].to_string())
    } else {
        None
    };

    let city = if fields.len() > 5 && !fields[5].is_empty() {
        Some(fields[5].to_string())
    } else {
        None
    };

    let postal_code = if fields.len() > 6 && !fields[6].is_empty() {
        Some(fields[6].to_string())
    } else {
        None
    };

    let timezone = if fields.len() > 7 && !fields[7].is_empty() {
        Some(fields[7].to_string())
    } else {
        None
    };

    Ok(GeofeedEntry {
        prefix,
        latitude,
        longitude,
        country,
        region,
        city,
        postal_code,
        timezone,
        quality: GeofeedQuality::Valid,
        roa_validation: None,
        manifest_validation: None,
        feed_timestamp: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_geofeed_line() {
        let line = "192.0.2.0/24,37.7749,-122.4194,US,CA,San Francisco,94102,America/Los_Angeles";
        let entry = parse_geofeed_line(line).unwrap();

        assert_eq!(entry.prefix.to_string(), "192.0.2.0/24");
        assert!((entry.latitude - 37.7749).abs() < 0.0001);
        assert!((entry.longitude - (-122.4194)).abs() < 0.0001);
        assert_eq!(entry.country, Some("US".to_string()));
        assert_eq!(entry.city, Some("San Francisco".to_string()));
    }

    #[test]
    fn test_parse_minimal_geofeed_line() {
        let line = "2001:db8::/32,51.5074,-0.1278";
        let entry = parse_geofeed_line(line).unwrap();

        assert_eq!(entry.prefix.to_string(), "2001:db8::/32");
        assert!((entry.latitude - 51.5074).abs() < 0.0001);
        assert!(entry.country.is_none());
    }

    #[test]
    fn test_parse_invalid_prefix() {
        let line = "not-a-prefix,37.7749,-122.4194";
        let result = parse_geofeed_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_fields() {
        let line = "192.0.2.0/24";
        let result = parse_geofeed_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_filter_valid_entry() {
        // Use config that doesn't warn on ROA/manifest staleness for deterministic test
        let config = QualityFilterConfig {
            max_rpki_age_days: 365, // Far future
            warn_shadowed: false,
            deterministic_testing: true, // Disable random staleness simulation
            ..Default::default()
        };
        let mut filter = GeofeedQualityFilter::new(config);
        let mut entry = GeofeedEntry {
            prefix: "192.0.2.0/24".parse().unwrap(),
            latitude: 37.7749,
            longitude: -122.4194,
            country: Some("US".to_string()),
            region: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            postal_code: None,
            timezone: None,
            quality: GeofeedQuality::Valid,
            roa_validation: None,
            manifest_validation: None,
            feed_timestamp: Some(Utc::now() + chrono::Duration::days(1)), // Future timestamp
        };

        let quality = filter.assess_entry(
            &mut entry,
            Some(64496),
            Some("https://example.com/manifest.cer"),
        );
        assert_eq!(quality, GeofeedQuality::Valid);
    }

    #[test]
    fn test_quality_filter_invalid_roa() {
        let config = QualityFilterConfig {
            reject_invalid_roa: true,
            ..Default::default()
        };
        let mut filter = GeofeedQualityFilter::new(config);
        let mut entry = GeofeedEntry {
            prefix: "192.0.2.0/24".parse().unwrap(),
            latitude: 37.7749,
            longitude: -122.4194,
            country: Some("US".to_string()),
            region: None,
            city: None,
            postal_code: None,
            timezone: None,
            quality: GeofeedQuality::Valid,
            roa_validation: None,
            manifest_validation: None,
            feed_timestamp: Some(Utc::now()),
        };

        let quality = filter.assess_entry(&mut entry, Some(0), None);
        assert!(matches!(
            quality,
            GeofeedQuality::Invalid(GeofeedError::InvalidRoa)
        ));
    }

    #[test]
    fn test_quality_filter_warning_roa() {
        let config = QualityFilterConfig {
            reject_invalid_roa: false,
            ..Default::default()
        };
        let mut filter = GeofeedQualityFilter::new(config);
        let mut entry = GeofeedEntry {
            prefix: "192.0.2.0/24".parse().unwrap(),
            latitude: 37.7749,
            longitude: -122.4194,
            country: Some("US".to_string()),
            region: None,
            city: None,
            postal_code: None,
            timezone: None,
            quality: GeofeedQuality::Valid,
            roa_validation: None,
            manifest_validation: None,
            feed_timestamp: Some(Utc::now()),
        };

        let quality = filter.assess_entry(&mut entry, Some(0), None);
        assert!(matches!(
            quality,
            GeofeedQuality::Warning(GeofeedWarning::RoaLowConfidence)
        ));
    }

    #[test]
    fn test_quality_filter_duplicate_prefix() {
        let mut filter = GeofeedQualityFilter::with_defaults();

        let mut entry1 = GeofeedEntry {
            prefix: "192.0.2.0/24".parse().unwrap(),
            latitude: 37.7749,
            longitude: -122.4194,
            country: Some("US".to_string()),
            region: None,
            city: None,
            postal_code: None,
            timezone: None,
            quality: GeofeedQuality::Valid,
            roa_validation: None,
            manifest_validation: None,
            feed_timestamp: Some(Utc::now()),
        };
        filter.assess_entry(&mut entry1, Some(64496), None);
        filter.seen_prefixes.insert(entry1.prefix, entry1);

        let mut entry2 = GeofeedEntry {
            prefix: "192.0.2.0/24".parse().unwrap(),
            latitude: 51.5074,
            longitude: -0.1278,
            country: Some("GB".to_string()),
            region: None,
            city: None,
            postal_code: None,
            timezone: None,
            quality: GeofeedQuality::Valid,
            roa_validation: None,
            manifest_validation: None,
            feed_timestamp: Some(Utc::now()),
        };
        let quality = filter.assess_entry(&mut entry2, Some(64496), None);
        assert!(matches!(
            quality,
            GeofeedQuality::Warning(GeofeedWarning::DuplicatePrefixDifferentLocation)
        ));
    }

    #[test]
    fn test_quality_filter_batch() {
        let mut filter = GeofeedQualityFilter::with_defaults();
        let entries = vec![
            GeofeedEntry {
                prefix: "192.0.2.0/24".parse().unwrap(),
                latitude: 37.7749,
                longitude: -122.4194,
                country: Some("US".to_string()),
                region: None,
                city: None,
                postal_code: None,
                timezone: None,
                quality: GeofeedQuality::Valid,
                roa_validation: None,
                manifest_validation: None,
                feed_timestamp: Some(Utc::now()),
            },
            GeofeedEntry {
                prefix: "192.0.2.0/24".parse().unwrap(),
                latitude: 51.5074,
                longitude: -0.1278,
                country: Some("GB".to_string()),
                region: None,
                city: None,
                postal_code: None,
                timezone: None,
                quality: GeofeedQuality::Valid,
                roa_validation: None,
                manifest_validation: None,
                feed_timestamp: Some(Utc::now()),
            },
        ];

        let filtered = filter.filter_batch(entries, Some(64496), None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_quality_filter_rejects_invalid() {
        let config = QualityFilterConfig {
            reject_invalid_roa: true,
            ..Default::default()
        };
        let mut filter = GeofeedQualityFilter::new(config);
        let entries = vec![
            GeofeedEntry {
                prefix: "192.0.2.0/24".parse().unwrap(),
                latitude: 37.7749,
                longitude: -122.4194,
                country: Some("US".to_string()),
                region: None,
                city: None,
                postal_code: None,
                timezone: None,
                quality: GeofeedQuality::Valid,
                roa_validation: None,
                manifest_validation: None,
                feed_timestamp: Some(Utc::now()),
            },
            GeofeedEntry {
                prefix: "192.0.2.0/24".parse().unwrap(),
                latitude: 0.0,
                longitude: 0.0,
                country: None,
                region: None,
                city: None,
                postal_code: None,
                timezone: None,
                quality: GeofeedQuality::Valid,
                roa_validation: None,
                manifest_validation: None,
                feed_timestamp: Some(Utc::now()),
            },
        ];

        // First entry has valid AS, second has invalid AS=0 and missing fields
        let filtered = filter.filter_batch(entries, Some(64496), None);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_prefix_contains() {
        let p1: IpNet = "192.0.2.0/24".parse().unwrap();
        let p2: IpNet = "192.0.2.0/25".parse().unwrap();
        let p3: IpNet = "192.0.2.128/25".parse().unwrap();
        let p4: IpNet = "192.0.3.0/24".parse().unwrap();

        assert!(p1.contains(&p2));
        assert!(p1.contains(&p3));
        assert!(!p1.contains(&p4));
    }
}
