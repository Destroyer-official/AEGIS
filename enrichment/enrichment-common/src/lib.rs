//! AEGIS Enrichment Common Utilities
//!
//! Shared components for the enrichment pipeline:
//! - Tiered cache with L1/L2/L3 fallback
//! - Data freshness SLA tracking
//! - Enrichment quality audit logging

pub mod audit;
pub mod cache;
pub mod freshness;

// Re-export key types
pub use audit::{AuditConfig, AuditEntry, AuditError, AuditLog, EnrichmentTier, FallbackReason};
pub use cache::{
    CacheConfig, CacheEntry, CacheError, CacheStats, CacheTier, TierConfig, TieredCache,
};
pub use freshness::{
    Alert, AlertLevel, FreshnessConfig, FreshnessError, FreshnessSource, FreshnessTracker,
    SourceFreshness,
};
