#![allow(clippy::result_large_err)]

mod audit_log;
mod auth;
mod grpc_handler;
mod middleware;
mod query;
mod rate_limit;
mod scan;
mod tenant;
mod ws;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use grpc_handler::GrpcQueryService;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::ws::ws_stream_handler;
use audit_log::{audit_middleware, AuditLog};
use auth::{create_token, AuthUser, Claims, JwtConfig, LoginRequest, TokenResponse, UserStore};
use middleware::{
    authz_middleware, init_tracing, require_admin, require_analyst, require_viewer,
    trace_middleware,
};
use query::{
    execute_trino_query, ColumnDef, QueryResult, TrinoConfig, TrinoQuery, TrinoRouter,
    TrinoStatementStats,
};
use rate_limit::{rate_limit_status, RateLimitConfig, RateLimitStatus, TokenBucketRateLimiter};
use scan::{CreateScanRequest, ScanJob, ScanOptions, ScanStore};
use tenant::{
    query_quota_middleware, scan_quota_middleware, tenant_isolation_middleware,
    CreateApiKeyResponse, TenantApiKey, TenantConfig, TenantStore,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AEGIS Gateway API",
        version = "0.1.0",
        description = "AEGIS Platform REST/WebSocket/gRPC Gateway"
    ),
    paths(
        health, login, query_handler, rate_limit_handler, admin_action, manage_users,
        list_tenants_handler, create_tenant_handler, update_tenant_handler, delete_tenant_handler,
        list_tenant_api_keys_handler, create_tenant_api_key_handler, revoke_tenant_api_key_handler,
        tenant_audit_log_handler, tenant_audit_log_export_handler, tenant_invite_user_handler,
        list_scans_handler, start_scan_handler, cancel_scan_handler
    ),
    components(schemas(
        HealthResponse, TokenResponse, LoginRequest, QueryResult, RateLimitStatus,
        TrinoQuery, TrinoStatementStats, ColumnDef, Claims, ErrorResponse,
        CreateApiKeyResponse, TenantApiKey, TenantConfig, CreateApiKeyRequest,
        InviteUserRequest, CreateScanRequest, ScanJob, ScanOptions
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "query", description = "Trino query endpoints"),
        (name = "admin", description = "Admin-only endpoints"),
        (name = "scans", description = "Scan orchestration endpoints"),
        (name = "rate-limit", description = "Rate limit status")
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Health check OK", body = HealthResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse)
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    error: String,
    code: u16,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct TenantAction {
    action: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    config: Option<TenantConfig>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct AuditAction {
    action: String,
    #[serde(default)]
    tenant_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
async fn login(request: axum::extract::Request) -> Result<Json<TokenResponse>, StatusCode> {
    let config = request
        .extensions()
        .get::<Arc<JwtConfig>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_store = request
        .extensions()
        .get::<Arc<UserStore>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let payload: LoginRequest =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Authenticate with Argon2id hashed credentials against user database
    let user = user_store
        .authenticate(&payload.username, &payload.password)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user.username.clone(),
        role: user.role.to_string(),
        tenant_id: user.tenant_id.clone(),
        exp: now + config.expiration_secs as usize,
        iat: now,
    };

    let token = create_token(&claims, &config).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse {
        token,
        expires_in: config.expiration_secs as u64,
    }))
}

#[utoipa::path(
    post,
    path = "/query",
    tag = "query",
    security(("bearerAuth" = [])),
    request_body = TrinoQuery,
    responses(
        (status = 200, description = "Query results", body = QueryResult),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn query_handler(
    user: AuthUser,
    request: axum::extract::Request,
) -> Result<Json<QueryResult>, StatusCode> {
    user.require_query()?;
    let trino = request
        .extensions()
        .get::<Arc<TrinoRouter>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let trino_query: TrinoQuery =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    execute_trino_query(user, trino_query, Extension(trino)).await
}

#[utoipa::path(
    get,
    path = "/rate-limit/status",
    tag = "rate-limit",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Rate limit status", body = RateLimitStatus),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
async fn rate_limit_handler(
    user: AuthUser,
    Extension(limiter): Extension<Arc<TokenBucketRateLimiter>>,
) -> Json<RateLimitStatus> {
    rate_limit_status(user, Extension(limiter)).await
}

async fn rate_limit_middleware(
    user: AuthUser,
    State(limiter): State<Arc<TokenBucketRateLimiter>>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    limiter.check_rate_limit(&user.claims.sub).await?;
    Ok(next.run(request).await)
}

#[utoipa::path(
    post,
    path = "/admin/action",
    tag = "admin",
    security(("bearerAuth" = [])),
    request_body = AuditAction,
    responses(
        (status = 200, description = "Admin action performed"),
        (status = 400, description = "Unknown action", body = ErrorResponse),
        (status = 403, description = "Requires admin role", body = ErrorResponse)
    )
)]
async fn admin_action(
    user: AuthUser,
    Extension(audit): Extension<Arc<AuditLog>>,
    Json(body): Json<AuditAction>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_write()?;
    match body.action.as_str() {
        "export" => {
            let entries = audit.export_entries(body.tenant_id.as_deref()).await;
            Ok(Json(serde_json::to_value(entries).unwrap()))
        }
        "verify" => match audit.verify_log().await {
            Ok(entries) => Ok(Json(
                serde_json::json!({"valid": true, "count": entries.len()}),
            )),
            Err(errors) => Ok(Json(serde_json::json!({"valid": false, "errors": errors}))),
        },
        "key" => Ok(Json(serde_json::json!({
            "verifying_key": hex::encode(audit.get_verifying_key().to_bytes()),
        }))),
        "key_hex" => Ok(Json(serde_json::json!({
            "verifying_key_hex": audit.verifying_key_hex(),
        }))),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[utoipa::path(
    post,
    path = "/admin/users",
    tag = "admin",
    security(("bearerAuth" = [])),
    request_body = TenantAction,
    responses(
        (status = 200, description = "User management action"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Requires manage_users permission", body = ErrorResponse)
    )
)]
async fn manage_users(
    user: AuthUser,
    Extension(store): Extension<Arc<TenantStore>>,
    Json(body): Json<TenantAction>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    match body.action.as_str() {
        "list" => {
            tracing::info!(tenant = %user.tenant_id(), "listing tenants");
            let tenants = store.get_all_tenants().await;
            Ok(Json(serde_json::to_value(tenants).unwrap()))
        }
        "create" => {
            let config = body.config.ok_or(StatusCode::BAD_REQUEST)?;
            store.create_tenant(config).await?;
            Ok(Json(serde_json::json!({"status": "created"})))
        }
        "update" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            let config = body.config.ok_or(StatusCode::BAD_REQUEST)?;
            store.update_tenant(&id, config).await?;
            Ok(Json(serde_json::json!({"status": "updated"})))
        }
        "delete" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            store.delete_tenant(&id).await?;
            Ok(Json(serde_json::json!({"status": "deleted"})))
        }
        "quota" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            let usage = store.get_quota_usage(&id).await;
            Ok(Json(serde_json::to_value(usage).unwrap()))
        }
        "apikey_create" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            let name = body.name.ok_or(StatusCode::BAD_REQUEST)?;
            let key = store
                .create_api_key(&id, &name, vec!["read".to_string()])
                .await?;
            Ok(Json(serde_json::to_value(key).unwrap()))
        }
        "apikey_list" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            let keys = store.list_api_keys(&id).await;
            Ok(Json(serde_json::to_value(keys).unwrap()))
        }
        "apikey_revoke" => {
            let id = body.id.ok_or(StatusCode::BAD_REQUEST)?;
            store.revoke_api_key(&id).await?;
            Ok(Json(serde_json::json!({"status": "revoked"})))
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct InviteUserRequest {
    pub email: String,
    #[serde(default = "default_invite_role")]
    pub role: String,
}

fn default_invite_role() -> String {
    "viewer".into()
}

#[utoipa::path(
    get,
    path = "/admin/tenants",
    tag = "admin",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List all tenants", body = Vec<TenantConfig>),
        (status = 403, description = "Requires manage_users permission", body = ErrorResponse)
    )
)]
async fn list_tenants_handler(
    user: AuthUser,
    Extension(store): Extension<Arc<TenantStore>>,
) -> Result<Json<Vec<TenantConfig>>, StatusCode> {
    user.require_manage_users()?;
    let tenants = store.get_all_tenants().await;
    Ok(Json(tenants))
}

#[utoipa::path(
    post,
    path = "/admin/tenants",
    tag = "admin",
    security(("bearerAuth" = [])),
    request_body = TenantConfig,
    responses(
        (status = 200, description = "Tenant created", body = TenantConfig),
        (status = 400, description = "Invalid configuration", body = ErrorResponse),
        (status = 403, description = "Requires manage_users permission", body = ErrorResponse)
    )
)]
async fn create_tenant_handler(
    user: AuthUser,
    Extension(store): Extension<Arc<TenantStore>>,
    Json(config): Json<TenantConfig>,
) -> Result<Json<TenantConfig>, StatusCode> {
    user.require_manage_users()?;
    store.create_tenant(config.clone()).await?;
    Ok(Json(config))
}

#[utoipa::path(
    put,
    path = "/admin/tenants/{id}",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    request_body = TenantConfig,
    responses(
        (status = 200, description = "Tenant updated", body = TenantConfig),
        (status = 400, description = "Invalid configuration", body = ErrorResponse),
        (status = 404, description = "Tenant not found", body = ErrorResponse)
    )
)]
async fn update_tenant_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(store): Extension<Arc<TenantStore>>,
    Json(config): Json<TenantConfig>,
) -> Result<Json<TenantConfig>, StatusCode> {
    user.require_manage_users()?;
    store.update_tenant(&id, config.clone()).await?;
    Ok(Json(config))
}

#[utoipa::path(
    delete,
    path = "/admin/tenants/{id}",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    responses(
        (status = 200, description = "Tenant deleted"),
        (status = 404, description = "Tenant not found", body = ErrorResponse)
    )
)]
async fn delete_tenant_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(store): Extension<Arc<TenantStore>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    store.delete_tenant(&id).await?;
    Ok(Json(serde_json::json!({"status": "deleted", "id": id})))
}

#[utoipa::path(
    get,
    path = "/admin/tenants/{id}/api-keys",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    responses(
        (status = 200, description = "List tenant API keys", body = Vec<TenantApiKey>),
        (status = 403, description = "Requires manage_users permission", body = ErrorResponse)
    )
)]
async fn list_tenant_api_keys_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(store): Extension<Arc<TenantStore>>,
) -> Result<Json<Vec<TenantApiKey>>, StatusCode> {
    user.require_manage_users()?;
    let keys = store.list_api_keys(&id).await;
    Ok(Json(keys))
}

#[utoipa::path(
    post,
    path = "/admin/tenants/{id}/api-keys",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, description = "API key created", body = CreateApiKeyResponse),
        (status = 404, description = "Tenant not found", body = ErrorResponse)
    )
)]
async fn create_tenant_api_key_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(store): Extension<Arc<TenantStore>>,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, StatusCode> {
    user.require_manage_users()?;
    let scopes = if body.scopes.is_empty() {
        vec!["read".to_string(), "query".to_string()]
    } else {
        body.scopes
    };
    let key = store.create_api_key(&id, &body.name, scopes).await?;
    Ok(Json(key))
}

#[utoipa::path(
    delete,
    path = "/admin/tenants/{id}/api-keys/{key_id}",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Tenant identifier"),
        ("key_id" = String, Path, description = "API Key UUID")
    ),
    responses(
        (status = 200, description = "API key revoked"),
        (status = 404, description = "Key not found", body = ErrorResponse)
    )
)]
async fn revoke_tenant_api_key_handler(
    user: AuthUser,
    Path((_id, key_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<TenantStore>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    store.revoke_api_key(&key_id).await?;
    Ok(Json(
        serde_json::json!({"status": "revoked", "key_id": key_id}),
    ))
}

#[utoipa::path(
    get,
    path = "/admin/tenants/{id}/audit-log",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    responses(
        (status = 200, description = "Tenant audit log entries")
    )
)]
async fn tenant_audit_log_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(audit): Extension<Arc<AuditLog>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    let entries = audit.export_entries(Some(&id)).await;
    Ok(Json(serde_json::to_value(entries).unwrap()))
}

#[utoipa::path(
    get,
    path = "/admin/tenants/{id}/audit-log/export",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    responses(
        (status = 200, description = "Tenant audit log export")
    )
)]
async fn tenant_audit_log_export_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(audit): Extension<Arc<AuditLog>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    let entries = audit.export_entries(Some(&id)).await;
    Ok(Json(serde_json::to_value(entries).unwrap()))
}

#[utoipa::path(
    post,
    path = "/admin/tenants/{id}/invite",
    tag = "admin",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Tenant identifier")),
    request_body = InviteUserRequest,
    responses(
        (status = 200, description = "User invited")
    )
)]
async fn tenant_invite_user_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(audit): Extension<Arc<AuditLog>>,
    Json(body): Json<InviteUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    user.require_manage_users()?;
    audit
        .log_request(
            &id,
            &user.claims.sub,
            "POST",
            "/admin/tenants/invite",
            200,
            1,
            "internal",
            "aegis-gateway",
        )
        .await;
    Ok(Json(serde_json::json!({
        "status": "invited",
        "tenant_id": id,
        "email": body.email,
        "role": body.role,
    })))
}

#[utoipa::path(
    get,
    path = "/scans",
    tag = "scans",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List scans", body = Vec<ScanJob>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
async fn list_scans_handler(
    user: AuthUser,
    Extension(scan_store): Extension<Arc<ScanStore>>,
) -> Result<Json<Vec<ScanJob>>, StatusCode> {
    user.require_query()?;
    let is_admin = user.claims.role == "admin";
    let scans = scan_store
        .list_scans(&user.claims.tenant_id, is_admin)
        .await;
    Ok(Json(scans))
}

#[utoipa::path(
    post,
    path = "/scans",
    tag = "scans",
    security(("bearerAuth" = [])),
    request_body = CreateScanRequest,
    responses(
        (status = 201, description = "Scan dispatched", body = ScanJob),
        (status = 400, description = "Invalid target or parameters", body = ErrorResponse),
        (status = 403, description = "Insufficient quota or permissions", body = ErrorResponse)
    )
)]
async fn start_scan_handler(
    user: AuthUser,
    Extension(scan_store): Extension<Arc<ScanStore>>,
    Extension(audit): Extension<Arc<AuditLog>>,
    Json(body): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<ScanJob>), StatusCode> {
    user.require_scan()?;
    let job = scan_store.create_scan(&user.claims.tenant_id, body).await?;
    audit
        .log_request(
            &user.claims.tenant_id,
            &user.claims.sub,
            "POST",
            "/scans",
            201,
            1,
            "internal",
            "aegis-gateway",
        )
        .await;
    Ok((StatusCode::CREATED, Json(job)))
}

#[utoipa::path(
    post,
    path = "/scans/{id}/cancel",
    tag = "scans",
    security(("bearerAuth" = [])),
    params(("id" = String, Path, description = "Scan identifier")),
    responses(
        (status = 200, description = "Scan cancelled", body = ScanJob),
        (status = 404, description = "Scan not found", body = ErrorResponse)
    )
)]
async fn cancel_scan_handler(
    user: AuthUser,
    Path(id): Path<String>,
    Extension(scan_store): Extension<Arc<ScanStore>>,
    Extension(audit): Extension<Arc<AuditLog>>,
) -> Result<Json<ScanJob>, StatusCode> {
    user.require_scan()?;
    let is_admin = user.claims.role == "admin";
    let job = scan_store
        .cancel_scan(&id, &user.claims.tenant_id, is_admin)
        .await?;
    audit
        .log_request(
            &user.claims.tenant_id,
            &user.claims.sub,
            "POST",
            &format!("/scans/{}/cancel", id),
            200,
            1,
            "internal",
            "aegis-gateway",
        )
        .await;
    Ok(Json(job))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aegis_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let _tracer_provider = init_tracing("aegis-gateway");

    let jwt_config = Arc::new(JwtConfig::from_env());
    let user_store = Arc::new(UserStore::new());
    let trino_config = TrinoConfig::default();
    let trino = Arc::new(TrinoRouter::new(trino_config));
    let rate_limit_config = RateLimitConfig::default();
    let rate_limiter = Arc::new(TokenBucketRateLimiter::new(rate_limit_config));
    let tenant_store = Arc::new(TenantStore::new());
    let audit_log = Arc::new(AuditLog::new("audit-logs"));
    let scan_store = Arc::new(ScanStore::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let grpc_service = Arc::new(GrpcQueryService::new(
        Arc::clone(&trino),
        Arc::clone(&jwt_config),
    ));
    let grpc_service_for_server = (*grpc_service).clone();

    let admin_routes = Router::new()
        .route("/admin/action", post(admin_action))
        .route("/admin/users", post(manage_users))
        .route(
            "/admin/tenants",
            get(list_tenants_handler).post(create_tenant_handler),
        )
        .route(
            "/admin/tenants/:id",
            put(update_tenant_handler).delete(delete_tenant_handler),
        )
        .route(
            "/admin/tenants/:id/api-keys",
            get(list_tenant_api_keys_handler).post(create_tenant_api_key_handler),
        )
        .route(
            "/admin/tenants/:id/api-keys/:key_id",
            delete(revoke_tenant_api_key_handler),
        )
        .route(
            "/admin/tenants/:id/audit-log",
            get(tenant_audit_log_handler),
        )
        .route(
            "/admin/tenants/:id/audit-log/export",
            get(tenant_audit_log_export_handler),
        )
        .route(
            "/admin/tenants/:id/invite",
            post(tenant_invite_user_handler),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            require_admin(jwt_config.clone()),
            authz_middleware,
        ));

    let scan_routes = Router::new()
        .route("/scans", get(list_scans_handler).post(start_scan_handler))
        .route("/scans/:id/cancel", post(cancel_scan_handler))
        .route("/scans/:id", delete(cancel_scan_handler))
        .route_layer(axum_middleware::from_fn_with_state(
            require_analyst(jwt_config.clone()),
            authz_middleware,
        ))
        .layer(axum::middleware::from_fn(scan_quota_middleware))
        .layer(Extension(Arc::clone(&scan_store)));

    let analyst_routes = Router::new()
        .route("/query", post(query_handler))
        .route_layer(axum_middleware::from_fn_with_state(
            require_analyst(jwt_config.clone()),
            authz_middleware,
        ))
        .layer(axum::middleware::from_fn(query_quota_middleware))
        .layer(Extension(Arc::clone(&trino)));

    let viewer_routes = Router::new()
        .route("/viewer/dashboard", get(|| async { "viewer dashboard" }))
        .route_layer(axum_middleware::from_fn_with_state(
            require_viewer(jwt_config.clone()),
            authz_middleware,
        ));

    let protected_routes = Router::new()
        .merge(admin_routes)
        .merge(analyst_routes)
        .merge(scan_routes)
        .merge(viewer_routes)
        .route_layer(axum_middleware::from_fn_with_state(
            Arc::clone(&rate_limiter),
            rate_limit_middleware,
        ));

    let login_router = Router::new().route("/auth/login", post(login));

    let app = Router::new()
        .route("/health", get(health))
        .route("/rate-limit/status", get(rate_limit_handler))
        .route("/ws/query", get(ws_handler))
        .route("/ws/stream", get(ws_stream_handler))
        .merge(login_router)
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(tenant_isolation_middleware))
        .layer(axum::middleware::from_fn(audit_middleware))
        .layer(axum::middleware::from_fn(trace_middleware))
        .layer(Extension(jwt_config))
        .layer(Extension(Arc::clone(&user_store)))
        .layer(Extension(Arc::clone(&trino)))
        .layer(Extension(grpc_service))
        .layer(Extension(Arc::clone(&rate_limiter)))
        .layer(Extension(Arc::clone(&tenant_store)))
        .layer(Extension(Arc::clone(&audit_log)))
        .layer(Extension(Arc::clone(&scan_store)))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let grpc_addr: std::net::SocketAddr = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".into())
        .parse()
        .map(|p: u16| format!("0.0.0.0:{}", p).parse().unwrap())
        .unwrap_or_else(|_| "0.0.0.0:50051".parse().unwrap());

    tracing::info!("gRPC server starting on {}", grpc_addr);

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(grpc_service_for_server.into_service())
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    let host = std::env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap_or(3000);

    let addr = format!("{}:{}", host, port);
    tracing::info!("AEGIS Gateway starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    user: AuthUser,
    Extension(trino): Extension<Arc<TrinoRouter>>,
) -> axum::response::Response {
    ws::ws_handler(ws, user, trino)
}
