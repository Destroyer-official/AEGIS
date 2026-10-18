//! Tiered Cache with Fallback Chains
//!
//! L1: In-process LRU (100K entries, sub-ms latency)
//! L2: Redis (optional, distributed, ms latency)
//! L3: Local fallback LRU with configurable TTL per tier

use chrono::{DateTime, Utc};
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use thiserror::Error;
use tracing::{debug, info, warn};

#[cfg(feature = "redis")]
use redis::{AsyncCommands, Client, ConnectionManager};

/// Errors for tiered cache operations
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("L1 cache miss: {0}")]
    L1Miss(String),
    #[error("L2 cache error: {0}")]
    L2Error(String),
    #[error("L3 fallback error: {0}")]
    L3Error(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Redis not enabled: compile with `redis` feature")]
    RedisDisabled,
    #[error("Redis connection error: {0}")]
    #[cfg(feature = "redis")]
    RedisConnection(#[from] redis::RedisError),
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: DateTime<Utc>,
    pub tier: CacheTier,
    pub ttl: StdDuration,
    pub access_count: u64,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, tier: CacheTier, ttl: StdDuration) -> Self {
        Self {
            value,
            created_at: Utc::now(),
            tier,
            ttl,
            access_count: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() - self.created_at > chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }

    pub fn record_access(&mut self) {
        self.access_count += 1;
    }
}

/// Cache tier identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTier {
    L1InProcess,
    L2Redis,
    L3Fallback,
}

/// Configuration for each cache tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub tier: CacheTier,
    pub max_entries: usize,
    pub default_ttl: StdDuration,
    pub enabled: bool,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            tier: CacheTier::L1InProcess,
            max_entries: 100_000,
            default_ttl: StdDuration::from_secs(300),
            enabled: true,
        }
    }
}

/// Tier-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub l1: TierConfig,
    pub l2: TierConfig,
    pub l3: TierConfig,
    pub fallback_ttl: StdDuration,
    pub redis_url: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1: TierConfig {
                tier: CacheTier::L1InProcess,
                max_entries: 100_000,
                default_ttl: StdDuration::from_secs(300),
                enabled: true,
            },
            l2: TierConfig {
                tier: CacheTier::L2Redis,
                max_entries: 1_000_000,
                default_ttl: StdDuration::from_secs(3600),
                enabled: cfg!(feature = "redis"),
            },
            l3: TierConfig {
                tier: CacheTier::L3Fallback,
                max_entries: 10_000,
                default_ttl: StdDuration::from_secs(86400),
                enabled: true,
            },
            fallback_ttl: StdDuration::from_secs(86400),
            redis_url: None,
        }
    }
}

/// Statistics for cache performance monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub fallback_triggers: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn total_hits(&self) -> u64 {
        self.l1_hits + self.l2_hits + self.l3_hits
    }

    pub fn total_misses(&self) -> u64 {
        self.l1_misses + self.l2_misses + self.l3_misses
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits() + self.total_misses();
        if total == 0 {
            0.0
        } else {
            self.total_hits() as f64 / total as f64
        }
    }

    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total == 0 {
            0.0
        } else {
            self.l1_hits as f64 / total as f64
        }
    }
}

/// Fallback function type for L3 cache miss
type FallbackFn<K, V> = Arc<dyn Fn(&K) -> Result<V, CacheError> + Send + Sync>;

/// Generic tiered cache with L1/L2/L3 fallback chain
pub struct TieredCache<K, V> {
    config: CacheConfig,
    l1: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    #[cfg(feature = "redis")]
    l2_client: Option<Client>,
    #[cfg(feature = "redis")]
    l2_manager: Arc<RwLock<Option<ConnectionManager>>>,
    l3: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    stats: Arc<RwLock<CacheStats>>,
    fallback_fn: Option<FallbackFn<K, V>>,
}

impl<K, V> TieredCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static + std::fmt::Display,
    V: Clone + Send + Sync + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let l1_cap = NonZeroUsize::new(config.l1.max_entries)
            .ok_or_else(|| CacheError::L1Miss("L1 capacity must be > 0".into()))?;
        let l3_cap = NonZeroUsize::new(config.l3.max_entries)
            .ok_or_else(|| CacheError::L3Error("L3 capacity must be > 0".into()))?;

        #[cfg(feature = "redis")]
        let (l2_client, l2_manager) = if config.l2.enabled {
            let url = config.redis_url.as_deref().unwrap_or("redis://127.0.0.1/");
            let client = Client::open(url)?;
            (Some(client), Arc::new(RwLock::new(None)))
        } else {
            (None, Arc::new(RwLock::new(None)))
        };

        Ok(Self {
            config,
            l1: Arc::new(RwLock::new(LruCache::new(l1_cap))),
            #[cfg(feature = "redis")]
            l2_client,
            #[cfg(feature = "redis")]
            l2_manager,
            l3: Arc::new(RwLock::new(LruCache::new(l3_cap))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            fallback_fn: None,
        })
    }

    pub fn with_fallback<F>(config: CacheConfig, fallback: F) -> Result<Self, CacheError>
    where
        F: Fn(&K) -> Result<V, CacheError> + Send + Sync + 'static,
    {
        let l1_cap = NonZeroUsize::new(config.l1.max_entries)
            .ok_or_else(|| CacheError::L1Miss("L1 capacity must be > 0".into()))?;
        let l3_cap = NonZeroUsize::new(config.l3.max_entries)
            .ok_or_else(|| CacheError::L3Error("L3 capacity must be > 0".into()))?;

        #[cfg(feature = "redis")]
        let (l2_client, l2_manager) = if config.l2.enabled {
            let url = config.redis_url.as_deref().unwrap_or("redis://127.0.0.1/");
            let client = Client::open(url)?;
            (Some(client), Arc::new(RwLock::new(None)))
        } else {
            (None, Arc::new(RwLock::new(None)))
        };

        Ok(Self {
            config,
            l1: Arc::new(RwLock::new(LruCache::new(l1_cap))),
            #[cfg(feature = "redis")]
            l2_client,
            #[cfg(feature = "redis")]
            l2_manager,
            l3: Arc::new(RwLock::new(LruCache::new(l3_cap))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            fallback_fn: Some(Arc::new(fallback)),
        })
    }

    #[cfg(feature = "redis")]
    async fn ensure_l2_connection(&self) -> Result<Option<ConnectionManager>, CacheError> {
        if !self.config.l2.enabled {
            return Ok(None);
        }

        {
            let mgr = self.l2_manager.read();
            if let Some(ref conn) = *mgr {
                return Ok(Some(conn.clone()));
            }
        }

        let client = self.l2_client.as_ref().ok_or(CacheError::RedisDisabled)?;
        let manager = ConnectionManager::new(client.clone()).await?;
        *self.l2_manager.write() = Some(manager.clone());
        Ok(Some(manager))
    }

    /// Get value from cache, traversing tiers L1 -> L2 -> L3 -> fallback
    pub async fn get(&self, key: &K) -> Result<V, CacheError> {
        // L1: In-process LRU
        if self.config.l1.enabled {
            let l1_result = {
                let l1 = self.l1.read();
                l1.peek(key).map(|entry| {
                    if !entry.is_expired() {
                        Ok(entry.value.clone())
                    } else {
                        Err(())
                    }
                })
            };

            match l1_result {
                Some(Ok(value)) => {
                    if let Some(entry) = self.l1.write().get_mut(key) {
                        entry.record_access();
                    }
                    self.stats.write().l1_hits += 1;
                    debug!(key = %key, "L1 cache hit");
                    return Ok(value);
                }
                Some(Err(_)) => {
                    self.l1.write().pop(key);
                }
                None => {}
            }
            self.stats.write().l1_misses += 1;
        }

        // L2: Redis (if enabled)
        #[cfg(feature = "redis")]
        if self.config.l2.enabled {
            if let Ok(Some(mut conn)) = self.ensure_l2_connection().await {
                let key_str = serde_json::to_string(key)?;
                match conn.get::<_, String>(&key_str).await {
                    Ok(val) => {
                        if let Ok(entry) = serde_json::from_str::<CacheEntry<V>>(&val) {
                            if !entry.is_expired() {
                                self.promote_to_l1(key, entry.clone()).await?;
                                self.stats.write().l2_hits += 1;
                                debug!(key = %key, "L2 cache hit");
                                return Ok(entry.value);
                            }
                        }
                    }
                    Err(_) => {}
                }
                self.stats.write().l2_misses += 1;
            }
        }

        // L3: Local fallback LRU
        if self.config.l3.enabled {
            let l3_entry = {
                let mut l3 = self.l3.write();
                let expired = l3.peek(key).is_some_and(|e| e.is_expired());
                if expired {
                    l3.pop(key);
                    None
                } else if let Some(entry) = l3.get_mut(key) {
                    entry.record_access();
                    Some(entry.clone())
                } else {
                    None
                }
            };

            if let Some(entry) = l3_entry {
                self.promote_to_l1(key, entry.clone()).await?;
                self.stats.write().l3_hits += 1;
                debug!(key = %key, "L3 cache hit");
                return Ok(entry.value.clone());
            }
            self.stats.write().l3_misses += 1;
        }

        // Fallback: invoke user-provided function
        self.stats.write().fallback_triggers += 1;
        warn!(key = %key, "Cache miss on all tiers, invoking fallback");

        if let Some(ref fallback) = self.fallback_fn {
            let value = fallback(key)?;
            let entry = CacheEntry::new(
                value.clone(),
                CacheTier::L3Fallback,
                self.config.fallback_ttl,
            );
            self.insert_l3(key, entry).await?;
            return Ok(value);
        }

        Err(CacheError::L3Error(format!("No fallback for key: {}", key)))
    }

    /// Insert into L1 cache
    pub async fn insert_l1(&self, key: K, value: V) -> Result<(), CacheError> {
        if !self.config.l1.enabled {
            return Ok(());
        }
        let entry = CacheEntry::new(value, CacheTier::L1InProcess, self.config.l1.default_ttl);
        let mut l1 = self.l1.write();
        l1.put(key, entry);
        Ok(())
    }

    /// Insert into L3 cache
    async fn insert_l3(&self, key: &K, entry: CacheEntry<V>) -> Result<(), CacheError> {
        if !self.config.l3.enabled {
            return Ok(());
        }
        let mut l3 = self.l3.write();
        l3.put(key.clone(), entry);
        Ok(())
    }

    /// Promote entry from lower tier to L1
    async fn promote_to_l1(&self, key: &K, entry: CacheEntry<V>) -> Result<(), CacheError> {
        if !self.config.l1.enabled {
            return Ok(());
        }
        let mut l1 = self.l1.write();
        let mut promoted = entry;
        promoted.tier = CacheTier::L1InProcess;
        l1.put(key.clone(), promoted);
        Ok(())
    }

    /// Write-through to L2 (Redis)
    #[cfg(feature = "redis")]
    pub async fn write_to_l2(
        &self,
        key: &K,
        value: &V,
        ttl: StdDuration,
    ) -> Result<(), CacheError> {
        if !self.config.l2.enabled || self.l2_client.is_none() {
            return Err(CacheError::RedisDisabled);
        }
        let entry = CacheEntry::new(value.clone(), CacheTier::L2Redis, ttl);
        let key_str = serde_json::to_string(key)?;
        let val_str = serde_json::to_string(&entry)?;
        if let Ok(Some(mut conn)) = self.ensure_l2_connection().await {
            conn.set_ex(key_str, val_str, ttl.as_secs() as usize)
                .await?;
        }
        Ok(())
    }

    /// Invalidate a key from all tiers
    pub async fn invalidate(&self, key: &K) -> Result<(), CacheError> {
        if self.config.l1.enabled {
            self.l1.write().pop(key);
        }
        #[cfg(feature = "redis")]
        if self.config.l2.enabled {
            if let Ok(Some(mut conn)) = self.ensure_l2_connection().await {
                let key_str = serde_json::to_string(key)?;
                conn.del(key_str).await?;
            }
        }
        if self.config.l3.enabled {
            self.l3.write().pop(key);
        }
        Ok(())
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = CacheStats::default();
    }

    /// Clean expired entries from L1 and L3
    pub fn cleanup_expired(&self) {
        if self.config.l1.enabled {
            let mut l1 = self.l1.write();
            let keys_to_remove: Vec<K> = l1
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(k, _)| k.clone())
                .collect();
            let evicted = keys_to_remove.len() as u64;
            for key in &keys_to_remove {
                l1.pop(key);
            }
            self.stats.write().evictions += evicted;
            if evicted > 0 {
                info!("L1 expired: {} entries evicted", evicted);
            }
        }

        if self.config.l3.enabled {
            let mut l3 = self.l3.write();
            let keys_to_remove: Vec<K> = l3
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(k, _)| k.clone())
                .collect();
            let evicted = keys_to_remove.len() as u64;
            for key in &keys_to_remove {
                l3.pop(key);
            }
            self.stats.write().evictions += evicted;
            if evicted > 0 {
                info!("L3 expired: {} entries evicted", evicted);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_l1_basic() {
        let config = CacheConfig::default();
        let cache: TieredCache<IpAddr, String> = TieredCache::new(config).unwrap();

        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        cache.insert_l1(ip, "test-value".to_string()).await.unwrap();

        let result = cache.get(&ip).await.unwrap();
        assert_eq!(result, "test-value");

        let stats = cache.stats();
        assert_eq!(stats.l1_hits, 1);
    }

    #[tokio::test]
    async fn test_l1_miss_l3_fallback() {
        let config = CacheConfig::default();
        let cache =
            TieredCache::with_fallback(config, |key: &IpAddr| Ok(format!("fallback-for-{}", key)))
                .unwrap();

        let ip = IpAddr::from_str("10.0.0.1").unwrap();
        let result = cache.get(&ip).await.unwrap();
        assert_eq!(result, "fallback-for-10.0.0.1");

        let stats = cache.stats();
        assert_eq!(stats.l1_misses, 1);
        assert_eq!(stats.fallback_triggers, 1);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let mut config = CacheConfig::default();
        config.l1.default_ttl = StdDuration::from_millis(50);
        let cache: TieredCache<IpAddr, String> = TieredCache::new(config).unwrap();

        let ip = IpAddr::from_str("192.168.1.2").unwrap();
        cache
            .insert_l1(ip, "short-lived".to_string())
            .await
            .unwrap();

        tokio::time::sleep(StdDuration::from_millis(100)).await;

        let result = cache.get(&ip).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let config = CacheConfig::default();
        let cache: TieredCache<IpAddr, String> = TieredCache::new(config).unwrap();

        let ip = IpAddr::from_str("192.168.1.3").unwrap();
        cache
            .insert_l1(ip, "to-invalidate".to_string())
            .await
            .unwrap();

        cache.invalidate(&ip).await.unwrap();

        let result = cache.get(&ip).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let config = CacheConfig::default();
        let cache: TieredCache<IpAddr, String> = TieredCache::new(config).unwrap();

        let ip1 = IpAddr::from_str("10.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("10.0.0.2").unwrap();

        cache.insert_l1(ip1, "v1".to_string()).await.unwrap();
        let _ = cache.get(&ip1).await;
        let _ = cache.get(&ip2).await; // miss

        let stats = cache.stats();
        assert_eq!(stats.l1_hits, 1);
        assert_eq!(stats.l1_misses, 1);
        assert_eq!(stats.l3_misses, 1); // L3 also misses for ip2
        assert_eq!(stats.total_hits(), 1);
        assert_eq!(stats.total_misses(), 2); // l1_miss + l3_miss
        assert!((stats.hit_rate() - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_entry_expiry() {
        let entry = CacheEntry::new("test", CacheTier::L1InProcess, StdDuration::from_millis(0));
        // TTL of 0 means immediate expiry
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats_defaults() {
        let stats = CacheStats::default();
        assert_eq!(stats.total_hits(), 0);
        assert_eq!(stats.total_misses(), 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.l1.max_entries, 100_000);
        assert_eq!(config.l3.max_entries, 10_000);
        assert!(config.l1.enabled);
        assert!(config.l3.enabled);
    }
}
