//! Token bucket rate limiter for per-target interrogation.
//!
//! Enforces rate limits on a per-IP or per-CIDR basis, designed to integrate
//! with Dev 9's gateway quotas while working standalone.
//!
//! # Algorithm
//!
//! Token bucket with configurable:
//! - Rate (tokens per second, default 100)
//! - Burst capacity (default 100)
//! - Per-key isolation (IP or CIDR)
//!
//! # CIDR Aggregation
//!
//! Rate limits can be applied per-/24 (IPv4) or per-/48 (IPv6) CIDR blocks
//! to prevent distributed scanning across a single network range.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limit configuration for a single key.
#[derive(Clone, Debug)]
struct BucketState {
    /// Tokens available
    tokens: f64,
    /// Maximum burst capacity
    capacity: f64,
    /// Refill rate (tokens per second)
    rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
}

impl BucketState {
    /// Create a new bucket.
    fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = now;
    }

    /// Try to consume a token. Returns true if allowed.
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Get the time until the next token is available.
    fn time_until_available(&self, tokens: f64) -> Duration {
        if self.tokens >= tokens {
            Duration::ZERO
        } else {
            let deficit = tokens - self.tokens;
            Duration::from_secs_f64(deficit / self.rate)
        }
    }
}

/// Rate limit configuration.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Default rate limit (tokens per second)
    pub default_rate: f64,
    /// Default burst capacity
    pub default_capacity: f64,
    /// Per-key rate limits (overrides default)
    pub key_limits: HashMap<String, KeyLimit>,
    /// CIDR aggregation mode
    pub cidr_mode: CidrMode,
}

/// Per-key rate limit override.
#[derive(Clone, Debug)]
pub struct KeyLimit {
    /// Rate limit for this key
    pub rate: f64,
    /// Burst capacity for this key
    pub capacity: f64,
}

/// CIDR aggregation mode for rate limiting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CidrMode {
    /// No CIDR aggregation (per-IP)
    None,
    /// Aggregate IPv4 by /24 subnet
    V4_24,
    /// Aggregate IPv6 by /48 subnet
    V6_48,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            default_rate: 100.0,
            default_capacity: 100.0,
            key_limits: HashMap::new(),
            cidr_mode: CidrMode::V4_24,
        }
    }
}

/// Token bucket rate limiter with per-key isolation.
pub struct RateLimiter {
    /// Configuration
    config: RateLimitConfig,
    /// Bucket states per key
    buckets: HashMap<String, BucketState>,
}

impl RateLimiter {
    /// Create a new rate limiter with default config (100 req/s).
    pub fn new() -> Self {
        Self {
            config: RateLimitConfig::default(),
            buckets: HashMap::new(),
        }
    }

    /// Create a rate limiter with custom config.
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: HashMap::new(),
        }
    }

    /// Create a rate limiter with a specific rate.
    pub fn with_rate(rate: f64) -> Self {
        Self {
            config: RateLimitConfig {
                default_rate: rate,
                default_capacity: rate,
                ..RateLimitConfig::default()
            },
            buckets: HashMap::new(),
        }
    }

    /// Try to consume a token for the given IP address.
    ///
    /// Returns `RateLimitResult` indicating whether the request is allowed.
    pub fn check(&mut self, ip: &str) -> RateLimitResult {
        let key = self.normalize_key(ip);
        self.check_key(&key)
    }

    /// Try to consume a token for a specific key.
    pub fn check_key(&mut self, key: &str) -> RateLimitResult {
        let (rate, capacity) = self.config_for_key(key);
        let bucket = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| BucketState::new(rate, capacity));

        // Update rate/capacity if changed
        bucket.rate = rate;
        bucket.capacity = capacity;

        if bucket.try_consume(1.0) {
            RateLimitResult::Allowed
        } else {
            let wait = bucket.time_until_available(1.0);
            RateLimitResult::Denied { retry_after: wait }
        }
    }

    /// Get the configuration for a specific key.
    fn config_for_key(&self, key: &str) -> (f64, f64) {
        if let Some(limit) = self.config.key_limits.get(key) {
            (limit.rate, limit.capacity)
        } else {
            (self.config.default_rate, self.config.default_capacity)
        }
    }

    /// Normalize an IP address to a rate limit key based on CIDR mode.
    fn normalize_key(&self, ip: &str) -> String {
        match self.config.cidr_mode {
            CidrMode::None => ip.to_string(),
            CidrMode::V4_24 => {
                let octets: Vec<&str> = ip.split('.').collect();
                if octets.len() == 4 {
                    format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
                } else {
                    ip.to_string()
                }
            }
            CidrMode::V6_48 => {
                let expanded = Self::expand_ipv6(ip);
                let groups: Vec<&str> = expanded.split(':').collect();
                if groups.len() >= 3 {
                    format!("{}:{}:{}/48", groups[0], groups[1], groups[2])
                } else {
                    ip.to_string()
                }
            }
        }
    }

    /// Expand an IPv6 address to 8 groups, handling `::` zero-compression.
    fn expand_ipv6(ip: &str) -> String {
        if !ip.contains("::") {
            return ip.to_string();
        }
        let parts: Vec<&str> = ip.split("::").collect();
        let left: Vec<&str> = if parts[0].is_empty() {
            vec![]
        } else {
            parts[0].split(':').collect()
        };
        let right: Vec<&str> = if parts.len() > 1 && !parts[1].is_empty() {
            parts[1].split(':').collect()
        } else {
            vec![]
        };
        let missing = 8 - left.len() - right.len();
        let mut groups: Vec<&str> = Vec::with_capacity(8);
        groups.extend_from_slice(&left);
        groups.extend(std::iter::repeat_n("0", missing));
        groups.extend_from_slice(&right);
        groups.join(":")
    }

    /// Get the number of tracked keys.
    pub fn key_count(&self) -> usize {
        self.buckets.len()
    }

    /// Remove expired buckets (older than `expiry`).
    pub fn cleanup(&mut self, expiry: Duration) {
        let now = Instant::now();
        self.buckets
            .retain(|_, bucket| now.duration_since(bucket.last_refill) < expiry);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit check result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Request is denied, retry after the specified duration
    Denied { retry_after: Duration },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_basic() {
        let mut limiter = RateLimiter::with_rate(10.0);
        // First request should be allowed
        assert!(limiter.check("192.168.1.1").is_allowed());
    }

    #[test]
    fn test_rate_limiter_cidr_aggregation() {
        let config = RateLimitConfig {
            default_rate: 2.0,
            default_capacity: 2.0,
            cidr_mode: CidrMode::V4_24,
            key_limits: HashMap::new(),
        };
        let mut limiter = RateLimiter::with_config(config);

        // Both IPs in same /24 should share the bucket
        assert!(limiter.check("10.0.0.1").is_allowed());
        assert!(limiter.check("10.0.0.2").is_allowed());
        // Third request in same /24 should be denied (capacity=2)
        let result = limiter.check("10.0.0.3");
        assert!(!result.is_allowed());

        // Different /24 should be allowed
        assert!(limiter.check("10.0.1.1").is_allowed());
    }

    #[test]
    fn test_rate_limiter_per_key_override() {
        let mut config = RateLimitConfig::default();
        config.key_limits.insert(
            "10.0.0.0/24".to_string(),
            KeyLimit {
                rate: 1.0,
                capacity: 1.0,
            },
        );
        let mut limiter = RateLimiter::with_config(config);

        assert!(limiter.check("10.0.0.1").is_allowed());
        assert!(!limiter.check("10.0.0.2").is_allowed());
    }

    #[test]
    fn test_rate_limiter_refill() {
        let config = RateLimitConfig {
            default_rate: 1000.0,
            default_capacity: 1.0,
            cidr_mode: CidrMode::None,
            key_limits: HashMap::new(),
        };
        let mut limiter = RateLimiter::with_config(config);

        assert!(limiter.check("1.2.3.4").is_allowed());
        assert!(!limiter.check("1.2.3.4").is_allowed());

        // Wait for refill (1 token at 1000/s = 1ms)
        thread::sleep(Duration::from_millis(2));
        assert!(limiter.check("1.2.3.4").is_allowed());
    }

    #[test]
    fn test_rate_limiter_cleanup() {
        let mut limiter = RateLimiter::with_rate(100.0);
        limiter.check("1.2.3.4");
        limiter.check("5.6.7.8");
        assert_eq!(limiter.key_count(), 2);

        // Cleanup with very short expiry should remove all
        limiter.cleanup(Duration::from_secs(0));
        assert_eq!(limiter.key_count(), 0);
    }

    #[test]
    fn test_rate_limit_result() {
        assert!(RateLimitResult::Allowed.is_allowed());
        assert!(!RateLimitResult::Denied {
            retry_after: Duration::from_millis(100)
        }
        .is_allowed());
    }

    #[test]
    fn test_v6_cidr_mode() {
        let config = RateLimitConfig {
            default_rate: 2.0,
            default_capacity: 2.0,
            cidr_mode: CidrMode::V6_48,
            key_limits: HashMap::new(),
        };
        let mut limiter = RateLimiter::with_config(config);

        assert!(limiter.check("2001:db8:1::1").is_allowed());
        assert!(limiter.check("2001:db8:1::2").is_allowed());
        // Same /48 group, should share bucket
        assert!(!limiter.check("2001:db8:1::3").is_allowed());
    }

    #[test]
    fn test_normalize_key_none() {
        let config = RateLimitConfig {
            cidr_mode: CidrMode::None,
            ..RateLimitConfig::default()
        };
        let limiter = RateLimiter::with_config(config);
        assert_eq!(limiter.normalize_key("192.168.1.1"), "192.168.1.1");
    }

    #[test]
    fn test_normalize_key_v4_24() {
        let config = RateLimitConfig {
            cidr_mode: CidrMode::V4_24,
            ..RateLimitConfig::default()
        };
        let limiter = RateLimiter::with_config(config);
        assert_eq!(limiter.normalize_key("192.168.1.100"), "192.168.1.0/24");
    }

    #[test]
    fn test_time_until_available() {
        let config = RateLimitConfig {
            default_rate: 10.0,
            default_capacity: 1.0,
            cidr_mode: CidrMode::None,
            key_limits: HashMap::new(),
        };
        let mut limiter = RateLimiter::with_config(config);
        limiter.check("1.2.3.4"); // consume the token

        let bucket = limiter.buckets.get("1.2.3.4").unwrap();
        let wait = bucket.time_until_available(1.0);
        assert!(wait > Duration::ZERO);
        assert!(wait <= Duration::from_millis(110)); // 1/10 = 100ms
    }

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.default_rate, 100.0);
        assert_eq!(config.default_capacity, 100.0);
        assert_eq!(config.cidr_mode, CidrMode::V4_24);
    }
}
