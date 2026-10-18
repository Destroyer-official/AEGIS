use axum::http::StatusCode;
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::AuthUser;

const MAX_LIMITERS: usize = 10_000;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: std::env::var("RATE_LIMIT_RPS")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .unwrap_or(100),
            burst_size: std::env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "200".into())
                .parse()
                .unwrap_or(200),
        }
    }
}

type Limiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
>;

pub struct TokenBucketRateLimiter {
    limiters: RwLock<HashMap<String, Arc<Limiter>>>,
    config: RateLimitConfig,
}

impl TokenBucketRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn get_limiter(&self, key: &str) -> Arc<Limiter> {
        {
            let limiters = self.limiters.read().await;
            if let Some(limiter) = limiters.get(key) {
                return Arc::clone(limiter);
            }
        }

        let mut limiters = self.limiters.write().await;
        // Bounded capacity check: prevent unbounded memory accumulation
        if limiters.len() >= MAX_LIMITERS {
            // Prune down to 75% capacity to prevent memory exhaustion under key spray attacks
            let keys_to_remove: Vec<String> =
                limiters.keys().take(MAX_LIMITERS / 4).cloned().collect();
            for k in keys_to_remove {
                limiters.remove(&k);
            }
        }

        let quota = Quota::per_second(NonZeroU32::new(self.config.requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(self.config.burst_size).unwrap());

        let limiter = Arc::new(RateLimiter::direct(quota));
        limiters.insert(key.to_string(), Arc::clone(&limiter));
        limiter
    }

    /// Fast non-blocking rate check: returns TOO_MANY_REQUESTS immediately instead of hanging threads
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), StatusCode> {
        let limiter = self.get_limiter(key).await;
        match limiter.check() {
            Ok(()) => Ok(()),
            Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RateLimitStatus {
    pub key: String,
    pub allowed: bool,
    pub remaining: u64,
}

pub async fn rate_limit_status(
    user: AuthUser,
    limiter: axum::extract::Extension<Arc<TokenBucketRateLimiter>>,
) -> axum::Json<RateLimitStatus> {
    let limiter_inner = limiter.get_limiter(&user.claims.sub).await;
    let allowed = limiter_inner.check().is_ok();

    axum::Json(RateLimitStatus {
        key: user.claims.sub,
        allowed,
        remaining: if allowed { 1 } else { 0 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_immediate_rejection() {
        let config = RateLimitConfig {
            requests_per_second: 1,
            burst_size: 1,
        };
        let limiter = TokenBucketRateLimiter::new(config);

        // First request allowed
        assert!(limiter.check_rate_limit("test-user").await.is_ok());

        // Immediate subsequent request must be rejected with 429 without blocking
        assert_eq!(
            limiter.check_rate_limit("test-user").await.unwrap_err(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}
