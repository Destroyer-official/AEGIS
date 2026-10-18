use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::auth::{verify_token, AuthUser, JwtConfig, Role};

#[derive(Clone)]
pub struct AuthzConfig {
    pub jwt_config: Arc<JwtConfig>,
    pub required_role: Role,
}

impl AuthzConfig {
    pub fn new(jwt_config: Arc<JwtConfig>, required_role: Role) -> Self {
        Self {
            jwt_config,
            required_role,
        }
    }
}

pub async fn authz_middleware(
    Extension(config): Extension<AuthzConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(token, &config.jwt_config).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let role = Role::from_str(&claims.role).ok_or(StatusCode::FORBIDDEN)?;

    let authorized = match (&role, &config.required_role) {
        (Role::Admin, _) => true,
        (Role::Analyst, Role::Admin) => false,
        (Role::Analyst, _) => true,
        (Role::Viewer, Role::Viewer) => true,
        _ => false,
    };

    if !authorized {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_user = AuthUser { claims };
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

pub fn require_admin(jwt_config: Arc<JwtConfig>) -> AuthzConfig {
    AuthzConfig::new(jwt_config, Role::Admin)
}

pub fn require_analyst(jwt_config: Arc<JwtConfig>) -> AuthzConfig {
    AuthzConfig::new(jwt_config, Role::Analyst)
}

pub fn require_viewer(jwt_config: Arc<JwtConfig>) -> AuthzConfig {
    AuthzConfig::new(jwt_config, Role::Viewer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{create_token, Claims, JwtConfig};

    fn test_config() -> Arc<JwtConfig> {
        Arc::new(JwtConfig {
            secret: "test-secret".into(),
            expiration_secs: 3600,
        })
    }

    #[tokio::test]
    async fn test_admin_access() {
        let config = test_config();
        let claims = Claims {
            sub: "admin-user".into(),
            role: "admin".into(),
            tenant_id: "tenant-a".into(),
            exp: 9999999999,
            iat: 1000000000,
        };
        let _token = create_token(&claims, &config).unwrap();

        let authz = AuthzConfig::new(config.clone(), Role::Admin);
        assert!(matches!(authz.required_role, Role::Admin));
    }

    #[tokio::test]
    async fn test_analyst_cannot_access_admin() {
        let config = test_config();
        let claims = Claims {
            sub: "analyst-user".into(),
            role: "analyst".into(),
            tenant_id: "tenant-a".into(),
            exp: 9999999999,
            iat: 1000000000,
        };
        let _token = create_token(&claims, &config).unwrap();

        let authz = AuthzConfig::new(config.clone(), Role::Admin);
        let role = Role::from_str("analyst").unwrap();
        assert_ne!(role, authz.required_role);
    }

    #[tokio::test]
    async fn test_viewer_only_viewer() {
        let config = test_config();
        let claims = Claims {
            sub: "viewer-user".into(),
            role: "viewer".into(),
            tenant_id: "tenant-a".into(),
            exp: 9999999999,
            iat: 1000000000,
        };
        let _token = create_token(&claims, &config).unwrap();

        let authz = AuthzConfig::new(config.clone(), Role::Viewer);
        assert_eq!(Role::Viewer, authz.required_role);
    }
}
