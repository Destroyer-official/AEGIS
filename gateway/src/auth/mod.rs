use async_trait::async_trait;
use axum::http::header::AUTHORIZATION;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Analyst,
    Viewer,
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
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "aegis-dev-secret-change-in-prod".to_string());
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
}
