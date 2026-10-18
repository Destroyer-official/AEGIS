use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::auth::{verify_token, JwtConfig};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantConfig {
    pub id: String,
    pub name: String,
    pub max_rate_rps: u32,
    pub max_scans_per_day: u32,
    pub max_queries_per_hour: u32,
    pub data_retention_days: u32,
    pub enabled: bool,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Tenant".to_string(),
            max_rate_rps: 100,
            max_scans_per_day: 50,
            max_queries_per_hour: 1000,
            data_retention_days: 90,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantQuotaUsage {
    pub tenant_id: String,
    pub scans_today: u32,
    pub queries_this_hour: u32,
    pub last_reset_scans: String,
    pub last_reset_queries: String,
}

/// Stored API Key record - stores only verifier hash, NEVER the raw secret
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantApiKey {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// Returned ONCE when creating an API key, containing the plaintext raw key secret
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub raw_key: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Clone)]
pub struct TenantStore {
    tenants: Arc<RwLock<HashMap<String, TenantConfig>>>,
    quotas: Arc<RwLock<HashMap<String, TenantQuotaUsage>>>,
    api_keys: Arc<RwLock<Vec<TenantApiKey>>>,
}

impl Default for TenantStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantStore {
    pub fn new() -> Self {
        let mut tenants = HashMap::new();

        tenants.insert(
            "tenant-a".to_string(),
            TenantConfig {
                id: "tenant-a".to_string(),
                name: "Acme Corp".to_string(),
                max_rate_rps: 200,
                max_scans_per_day: 100,
                max_queries_per_hour: 5000,
                data_retention_days: 180,
                enabled: true,
            },
        );

        tenants.insert(
            "tenant-b".to_string(),
            TenantConfig {
                id: "tenant-b".to_string(),
                name: "Beta Industries".to_string(),
                max_rate_rps: 50,
                max_scans_per_day: 20,
                max_queries_per_hour: 500,
                data_retention_days: 30,
                enabled: true,
            },
        );

        tenants.insert(
            "tenant-c".to_string(),
            TenantConfig {
                id: "tenant-c".to_string(),
                name: "Charlie Labs".to_string(),
                max_rate_rps: 100,
                max_scans_per_day: 50,
                max_queries_per_hour: 1000,
                data_retention_days: 90,
                enabled: true,
            },
        );

        Self {
            tenants: Arc::new(RwLock::new(tenants)),
            quotas: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> Option<TenantConfig> {
        let tenants = self.tenants.read().await;
        tenants.get(tenant_id).cloned()
    }

    pub async fn get_all_tenants(&self) -> Vec<TenantConfig> {
        let tenants = self.tenants.read().await;
        tenants.values().cloned().collect()
    }

    pub async fn create_tenant(&self, config: TenantConfig) -> Result<TenantConfig, StatusCode> {
        let mut tenants = self.tenants.write().await;
        let id = config.id.clone();
        if tenants.contains_key(&id) {
            return Err(StatusCode::CONFLICT);
        }
        tenants.insert(id.clone(), config.clone());
        info!("Created tenant: {}", id);
        Ok(config)
    }

    pub async fn update_tenant(
        &self,
        tenant_id: &str,
        config: TenantConfig,
    ) -> Result<TenantConfig, StatusCode> {
        // Enforce tenant ID consistency: path ID must match body ID to prevent tenant hijacking
        if config.id != tenant_id {
            warn!(
                "Tenant update failed: body id '{}' does not match path id '{}'",
                config.id, tenant_id
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        let mut tenants = self.tenants.write().await;
        if !tenants.contains_key(tenant_id) {
            return Err(StatusCode::NOT_FOUND);
        }
        tenants.insert(tenant_id.to_string(), config.clone());
        info!("Updated tenant: {}", tenant_id);
        Ok(config)
    }

    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), StatusCode> {
        let mut tenants = self.tenants.write().await;
        if tenants.remove(tenant_id).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
        info!("Deleted tenant: {}", tenant_id);
        Ok(())
    }

    pub async fn get_quota_usage(&self, tenant_id: &str) -> TenantQuotaUsage {
        let quotas = self.quotas.read().await;
        quotas.get(tenant_id).cloned().unwrap_or(TenantQuotaUsage {
            tenant_id: tenant_id.to_string(),
            ..Default::default()
        })
    }

    pub async fn check_scan_quota(&self, tenant_id: &str) -> Result<(), StatusCode> {
        let config = self
            .get_tenant(tenant_id)
            .await
            .ok_or(StatusCode::NOT_FOUND)?;

        if !config.enabled {
            warn!("Tenant {} is disabled", tenant_id);
            return Err(StatusCode::FORBIDDEN);
        }

        let mut quotas = self.quotas.write().await;
        let usage = quotas
            .entry(tenant_id.to_string())
            .or_insert_with(|| TenantQuotaUsage {
                tenant_id: tenant_id.to_string(),
                ..Default::default()
            });

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if usage.last_reset_scans != today {
            usage.scans_today = 0;
            usage.last_reset_scans = today;
        }

        if usage.scans_today >= config.max_scans_per_day {
            warn!(
                "Tenant {} scan quota exceeded: {}/{}",
                tenant_id, usage.scans_today, config.max_scans_per_day
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        usage.scans_today += 1;
        Ok(())
    }

    pub async fn check_query_quota(&self, tenant_id: &str) -> Result<(), StatusCode> {
        let config = self
            .get_tenant(tenant_id)
            .await
            .ok_or(StatusCode::NOT_FOUND)?;

        if !config.enabled {
            return Err(StatusCode::FORBIDDEN);
        }

        let mut quotas = self.quotas.write().await;
        let usage = quotas
            .entry(tenant_id.to_string())
            .or_insert_with(|| TenantQuotaUsage {
                tenant_id: tenant_id.to_string(),
                ..Default::default()
            });

        let hour = chrono::Utc::now().format("%Y-%m-%dT%H").to_string();
        if usage.last_reset_queries != hour {
            usage.queries_this_hour = 0;
            usage.last_reset_queries = hour;
        }

        if usage.queries_this_hour >= config.max_queries_per_hour {
            warn!(
                "Tenant {} query quota exceeded: {}/{}",
                tenant_id, usage.queries_this_hour, config.max_queries_per_hour
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        usage.queries_this_hour += 1;
        Ok(())
    }

    /// Creates an API key with high-entropy CSPRNG bytes and stores a SHA-256 verifier.
    /// Returns the raw key secret ONCE in CreateApiKeyResponse.
    pub async fn create_api_key(
        &self,
        tenant_id: &str,
        name: &str,
        scopes: Vec<String>,
    ) -> Result<CreateApiKeyResponse, StatusCode> {
        if self.get_tenant(tenant_id).await.is_none() {
            return Err(StatusCode::NOT_FOUND);
        }

        let key_id = uuid::Uuid::new_v4().to_string();

        // 32-byte cryptographically secure random entropy
        let mut entropy = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let raw_key = format!("aegis_{}_{}", tenant_id, hex::encode(entropy));

        // SHA-256 hash verifier
        let key_hash = format!("{:x}", Sha256::digest(raw_key.as_bytes()));
        let created_at = chrono::Utc::now().to_rfc3339();

        let api_key_record = TenantApiKey {
            id: key_id.clone(),
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            key_hash,
            scopes: scopes.clone(),
            created_at: created_at.clone(),
            expires_at: None,
        };

        let mut keys = self.api_keys.write().await;
        keys.push(api_key_record);
        info!("Created API key '{}' for tenant {}", name, tenant_id);

        Ok(CreateApiKeyResponse {
            id: key_id,
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            raw_key,
            scopes,
            created_at,
            expires_at: None,
        })
    }

    /// Verifies a raw API key against stored SHA-256 verifiers, checking expiry, scopes, and tenant status.
    #[allow(dead_code)]
    pub async fn verify_api_key(
        &self,
        raw_key: &str,
        required_scope: &str,
    ) -> Result<TenantApiKey, StatusCode> {
        let hash = format!("{:x}", Sha256::digest(raw_key.as_bytes()));

        let keys = self.api_keys.read().await;
        let key = keys
            .iter()
            .find(|k| k.key_hash == hash)
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Verify tenant is active
        let tenant = self
            .get_tenant(&key.tenant_id)
            .await
            .ok_or(StatusCode::FORBIDDEN)?;
        if !tenant.enabled {
            warn!("API key used for disabled tenant {}", key.tenant_id);
            return Err(StatusCode::FORBIDDEN);
        }

        // Verify expiration if configured
        if let Some(expires_at) = &key.expires_at {
            if let Ok(exp_time) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if chrono::Utc::now() > exp_time {
                    warn!("API key {} has expired", key.id);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }

        // Verify scopes (allow wildcard "*" or specific scope match)
        if !key.scopes.iter().any(|s| s == "*" || s == required_scope) {
            warn!(
                "API key {} lacks required scope '{}' (has: {:?})",
                key.id, required_scope, key.scopes
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(key)
    }

    pub async fn list_api_keys(&self, tenant_id: &str) -> Vec<TenantApiKey> {
        let keys = self.api_keys.read().await;
        keys.iter()
            .filter(|k| k.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    pub async fn revoke_api_key(&self, key_id: &str) -> Result<(), StatusCode> {
        let mut keys = self.api_keys.write().await;
        let len_before = keys.len();
        keys.retain(|k| k.id != key_id);
        if keys.len() == len_before {
            return Err(StatusCode::NOT_FOUND);
        }
        info!("Revoked API key: {}", key_id);
        Ok(())
    }
}

pub async fn tenant_isolation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let tenant_store = request.extensions().get::<Arc<TenantStore>>().cloned();
    let jwt_config = request.extensions().get::<Arc<JwtConfig>>().cloned();

    if let (Some(store), Some(config)) = (tenant_store, jwt_config) {
        if let Some(auth_header) = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Ok(claims) = verify_token(token, &config) {
                    let tenant_id = &claims.tenant_id;

                    let tenant_config = store
                        .get_tenant(tenant_id)
                        .await
                        .ok_or(StatusCode::FORBIDDEN)?;

                    if !tenant_config.enabled {
                        warn!("Disabled tenant attempted access: {}", tenant_id);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn scan_quota_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let tenant_store = request.extensions().get::<Arc<TenantStore>>().cloned();
    let jwt_config = request.extensions().get::<Arc<JwtConfig>>().cloned();

    if let (Some(store), Some(config)) = (tenant_store, jwt_config) {
        if let Some(auth_header) = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Ok(claims) = verify_token(token, &config) {
                    store.check_scan_quota(&claims.tenant_id).await?;
                }
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn query_quota_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let tenant_store = request.extensions().get::<Arc<TenantStore>>().cloned();
    let jwt_config = request.extensions().get::<Arc<JwtConfig>>().cloned();

    if let (Some(store), Some(config)) = (tenant_store, jwt_config) {
        if let Some(auth_header) = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Ok(claims) = verify_token(token, &config) {
                    store.check_query_quota(&claims.tenant_id).await?;
                }
            }
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_store_crud() {
        let store = TenantStore::new();

        let all = store.get_all_tenants().await;
        assert_eq!(all.len(), 3);

        let tenant = store.get_tenant("tenant-a").await.unwrap();
        assert_eq!(tenant.name, "Acme Corp");
        assert_eq!(tenant.max_scans_per_day, 100);

        let new_tenant = TenantConfig {
            id: "tenant-d".to_string(),
            name: "Delta Inc".to_string(),
            max_rate_rps: 75,
            max_scans_per_day: 30,
            max_queries_per_hour: 800,
            data_retention_days: 60,
            enabled: true,
        };
        store.create_tenant(new_tenant).await.unwrap();
        assert_eq!(store.get_all_tenants().await.len(), 4);

        store.delete_tenant("tenant-d").await.unwrap();
        assert_eq!(store.get_all_tenants().await.len(), 3);
    }

    #[tokio::test]
    async fn test_tenant_update_id_mismatch_rejected() {
        let store = TenantStore::new();
        let hijacked_config = TenantConfig {
            id: "tenant-b".to_string(),
            name: "Hijack Attempt".to_string(),
            max_rate_rps: 10,
            max_scans_per_day: 10,
            max_queries_per_hour: 10,
            data_retention_days: 10,
            enabled: true,
        };
        // Updating tenant-a with body id tenant-b must be rejected!
        let result = store.update_tenant("tenant-a", hijacked_config).await;
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_api_key_lifecycle_and_verification() {
        let store = TenantStore::new();

        let resp = store
            .create_api_key("tenant-a", "test-key", vec!["read".to_string()])
            .await
            .unwrap();
        assert_eq!(resp.tenant_id, "tenant-a");
        assert_eq!(resp.name, "test-key");
        assert!(resp.raw_key.starts_with("aegis_tenant-a_"));

        // Listed keys contain only metadata, never the raw key
        let keys = store.list_api_keys("tenant-a").await;
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, resp.id);

        // Verify valid key with valid scope
        let verified = store.verify_api_key(&resp.raw_key, "read").await;
        assert!(verified.is_ok());

        // Verify valid key with invalid scope fails
        let invalid_scope = store.verify_api_key(&resp.raw_key, "admin:write").await;
        assert_eq!(invalid_scope.unwrap_err(), StatusCode::FORBIDDEN);

        // Verify invalid key string fails
        let fake = store.verify_api_key("aegis_fake_key_12345", "read").await;
        assert_eq!(fake.unwrap_err(), StatusCode::UNAUTHORIZED);

        // Revoke key
        store.revoke_api_key(&resp.id).await.unwrap();
        let after_revocation = store.verify_api_key(&resp.raw_key, "read").await;
        assert_eq!(after_revocation.unwrap_err(), StatusCode::UNAUTHORIZED);
    }
}
