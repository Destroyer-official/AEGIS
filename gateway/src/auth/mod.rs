use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use axum::http::header::AUTHORIZATION;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Analyst,
    Viewer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Analyst => write!(f, "analyst"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "analyst" => Some(Role::Analyst),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    pub fn can_query(&self) -> bool {
        matches!(self, Role::Admin | Role::Analyst)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Role::Admin)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub tenant_id: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_secs: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        let is_prod = std::env::var("AEGIS_ENV")
            .map(|v| v.to_lowercase() == "production")
            .unwrap_or(false)
            || std::env::var("ENVIRONMENT")
                .map(|v| v.to_lowercase() == "production")
                .unwrap_or(false);

        let secret = match std::env::var("JWT_SECRET") {
            Ok(s) => {
                if is_prod && s.len() < 32 {
                    panic!("FATAL: JWT_SECRET must be at least 32 characters in production environments");
                }
                s
            }
            Err(_) => {
                if is_prod {
                    panic!("FATAL: JWT_SECRET environment variable is mandatory in production");
                }
                tracing::warn!("SECURITY WARNING: Running with generated development JWT secret. Set JWT_SECRET in production.");
                "aegis-dev-secret-change-in-prod-min32-bytes-long!".to_string()
            }
        };

        let expiration_secs: i64 = std::env::var("JWT_EXPIRATION_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        Self {
            secret,
            expiration_secs,
        }
    }
}

pub fn create_token(
    claims: &Claims,
    config: &JwtConfig,
) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

pub fn verify_token(
    token: &str,
    config: &JwtConfig,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[derive(Clone)]
pub struct AuthUser {
    pub claims: Claims,
}

impl AuthUser {
    pub fn role(&self) -> Option<Role> {
        Role::from_str(&self.claims.role)
    }

    pub fn tenant_id(&self) -> String {
        self.claims.tenant_id.clone()
    }

    pub fn require_query(&self) -> Result<(), StatusCode> {
        self.role()
            .filter(|r| r.can_query())
            .ok_or(StatusCode::FORBIDDEN)
            .map(|_| ())
    }

    pub fn require_write(&self) -> Result<(), StatusCode> {
        self.role()
            .filter(|r| r.can_write())
            .ok_or(StatusCode::FORBIDDEN)
            .map(|_| ())
    }

    pub fn require_manage_users(&self) -> Result<(), StatusCode> {
        self.role()
            .filter(|r| r.can_manage_users())
            .ok_or(StatusCode::FORBIDDEN)
            .map(|_| ())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<JwtConfig>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(JwtConfig::from_env()));

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let claims = verify_token(token, &config).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser { claims })
    }
}

/// Server-side user account with Argon2id password hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub tenant_id: String,
    pub enabled: bool,
}

/// Thread-safe in-memory User Account Store with Argon2id credential verification
#[derive(Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, UserAccount>>>,
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStore {
    pub fn new() -> Self {
        let mut store = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        };

        // Seed default accounts with Argon2id hashes
        let admin_pw = std::env::var("INITIAL_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "AegisAdmin#Secure2026!".to_string());
        let analyst_pw = std::env::var("INITIAL_ANALYST_PASSWORD")
            .unwrap_or_else(|_| "AegisAnalyst#2026!".to_string());
        let viewer_pw = std::env::var("INITIAL_VIEWER_PASSWORD")
            .unwrap_or_else(|_| "AegisViewer#2026!".to_string());

        store.add_user_sync("admin", &admin_pw, Role::Admin, "tenant-a");
        store.add_user_sync("analyst", &analyst_pw, Role::Analyst, "tenant-a");
        store.add_user_sync("viewer", &viewer_pw, Role::Viewer, "tenant-a");

        store
    }

    /// Helper to hash password with Argon2id and a secure random salt
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| format!("Password hashing failed: {}", e))
    }

    /// Synchronous initialization helper
    fn add_user_sync(&mut self, username: &str, password: &str, role: Role, tenant_id: &str) {
        if let Ok(hash) = Self::hash_password(password) {
            let account = UserAccount {
                username: username.to_string(),
                password_hash: hash,
                role,
                tenant_id: tenant_id.to_string(),
                enabled: true,
            };
            if let Ok(mut users) = self.users.try_write() {
                users.insert(username.to_string(), account);
            }
        }
    }

    /// Verify credentials using Argon2id constant-time verification
    pub async fn authenticate(&self, username: &str, password: &str) -> Option<UserAccount> {
        let users = self.users.read().await;
        let user = users.get(username)?;
        if !user.enabled {
            return None;
        }

        let parsed_hash = PasswordHash::new(&user.password_hash).ok()?;
        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            Some(user.clone())
        } else {
            None
        }
    }

    /// Add or update user account
    #[allow(dead_code)]
    pub async fn upsert_user(
        &self,
        username: &str,
        password: &str,
        role: Role,
        tenant_id: &str,
    ) -> Result<(), String> {
        let hash = Self::hash_password(password)?;
        let account = UserAccount {
            username: username.to_string(),
            password_hash: hash,
            role,
            tenant_id: tenant_id.to_string(),
            enabled: true,
        };
        let mut users = self.users.write().await;
        users.insert(username.to_string(), account);
        Ok(())
    }

    /// List registered users (metadata only, no password hashes)
    #[allow(dead_code)]
    pub async fn list_users(&self) -> Vec<(String, Role, String, bool)> {
        let users = self.users.read().await;
        users
            .values()
            .map(|u| {
                (
                    u.username.clone(),
                    u.role.clone(),
                    u.tenant_id.clone(),
                    u.enabled,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        assert!(Role::Admin.can_query());
        assert!(Role::Admin.can_write());
        assert!(Role::Admin.can_manage_users());
        assert!(Role::Analyst.can_query());
        assert!(!Role::Analyst.can_write());
        assert!(!Role::Analyst.can_manage_users());
        assert!(!Role::Viewer.can_query());
        assert!(!Role::Viewer.can_write());
        assert!(!Role::Viewer.can_manage_users());
    }

    #[test]
    fn test_token_roundtrip() {
        let config = JwtConfig::from_env();
        let claims = Claims {
            sub: "test-user".into(),
            role: "analyst".into(),
            tenant_id: "tenant-a".into(),
            exp: 9999999999,
            iat: 1000000000,
        };
        let token = create_token(&claims, &config).unwrap();
        let decoded = verify_token(&token, &config).unwrap();
        assert_eq!(decoded.sub, "test-user");
        assert_eq!(decoded.role, "analyst");
    }

    #[tokio::test]
    async fn test_argon2_authentication_success() {
        let store = UserStore::new();
        // Default seeded admin
        let user = store.authenticate("admin", "AegisAdmin#Secure2026!").await;
        assert!(user.is_some());
        let u = user.unwrap();
        assert_eq!(u.role, Role::Admin);
        assert_eq!(u.tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn test_authentication_failure_invalid_password() {
        let store = UserStore::new();
        // Trying password prefix bypass must FAIL!
        let user = store.authenticate("admin", "admin123").await;
        assert!(user.is_none());
        let user = store.authenticate("analyst", "admin").await;
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_authentication_failure_unknown_user() {
        let store = UserStore::new();
        let user = store.authenticate("attacker", "admin123").await;
        assert!(user.is_none());
    }
}
