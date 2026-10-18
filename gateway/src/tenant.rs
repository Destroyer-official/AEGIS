use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::auth::{verify_token, JwtConfig};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TenantApiKey {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TenantStore {
    tenants: Arc<RwLock<HashMap<String, TenantConfig>>>,
    quotas: Arc<RwLock<HashMap<String, TenantQuotaUsage>>>,
    api_keys: Arc<RwLock<Vec<TenantApiKey>>>,
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
        if tenants.contains_key(&config.id) {
            return Err(StatusCode::CONFLICT);
        }
        let id = config.id.clone();
        tenants.insert(id.clone(), config.clone());
        info!("Created tenant: {}", id);
        Ok(config)
    }

    pub async fn update_tenant(
        &self,
        tenant_id: &str,
        config: TenantConfig,
    ) -> Result<TenantConfig, StatusCode> {
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

    pub async fn create_api_key(
        &self,
        tenant_id: &str,
        name: &str,
        scopes: Vec<String>,
    ) -> Result<TenantApiKey, StatusCode> {
        let key_id = uuid::Uuid::new_v4().to_string();
        let raw_key = format!("aegis_{}_{}", tenant_id, uuid::Uuid::new_v4());
        let key_hash = format!("{:x}", md5::compute(raw_key.as_bytes()));

        let api_key = TenantApiKey {
            id: key_id,
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            key_hash,
            scopes,
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: None,
        };

        let mut keys = self.api_keys.write().await;
        keys.push(api_key.clone());
        info!("Created API key '{}' for tenant {}", name, tenant_id);
        Ok(api_key)
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

                    if store.get_tenant(tenant_id).await.is_none() {
                        warn!("Unknown tenant: {}", tenant_id);
                        return Err(StatusCode::FORBIDDEN);
                    }

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
    async fn test_scan_quota_enforcement() {
        let store = TenantStore::new();

        store.tenants.write().await.insert(
            "test".to_string(),
            TenantConfig {
                id: "test".to_string(),
                name: "Test".to_string(),
                max_rate_rps: 10,
                max_scans_per_day: 2,
                max_queries_per_hour: 100,
                data_retention_days: 30,
                enabled: true,
            },
        );

        assert!(store.check_scan_quota("test").await.is_ok());
        assert!(store.check_scan_quota("test").await.is_ok());
        assert_eq!(
            store.check_scan_quota("test").await.unwrap_err(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn test_disabled_tenant() {
        let store = TenantStore::new();

        store.tenants.write().await.insert(
            "disabled".to_string(),
            TenantConfig {
                id: "disabled".to_string(),
                name: "Disabled".to_string(),
                max_rate_rps: 10,
                max_scans_per_day: 100,
                max_queries_per_hour: 100,
                data_retention_days: 30,
                enabled: false,
            },
        );

        assert_eq!(
            store.check_scan_quota("disabled").await.unwrap_err(),
            StatusCode::FORBIDDEN
        );
    }

    #[tokio::test]
    async fn test_api_key_lifecycle() {
        let store = TenantStore::new();

        let key = store
            .create_api_key("tenant-a", "test-key", vec!["read".to_string()])
            .await
            .unwrap();
        assert_eq!(key.tenant_id, "tenant-a");
        assert_eq!(key.name, "test-key");

        let keys = store.list_api_keys("tenant-a").await;
        assert_eq!(keys.len(), 1);

        store.revoke_api_key(&key.id).await.unwrap();
        let keys = store.list_api_keys("tenant-a").await;
        assert_eq!(keys.len(), 0);
    }
}
