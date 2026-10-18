#![allow(clippy::result_large_err)]

mod audit_log;
mod auth;
mod grpc_handler;
mod middleware;
mod query;
mod rate_limit;
mod tenant;
mod ws;

use axum::{
    extract::Extension,
    http::StatusCode,
    middleware as axum_middleware,
    routing::{get, post},
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
use auth::{create_token, AuthUser, Claims, JwtConfig, LoginRequest, TokenResponse};
use middleware::{
    authz_middleware, init_tracing, require_admin, require_analyst, require_viewer,
    trace_middleware,
};
use query::{
    execute_trino_query, ColumnDef, QueryResult, TrinoConfig, TrinoQuery, TrinoRouter,
    TrinoStatementStats,
};
use rate_limit::{rate_limit_status, RateLimitConfig, RateLimitStatus, TokenBucketRateLimiter};
use tenant::{
    query_quota_middleware, scan_quota_middleware, tenant_isolation_middleware, TenantConfig,
    TenantStore,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AEGIS Gateway API",
        version = "0.1.0",
        description = "AEGIS Platform REST/WebSocket/gRPC Gateway"
    ),
    paths(health, login, query_handler, rate_limit_handler, admin_action, manage_users),
    components(schemas(
        HealthResponse, TokenResponse, LoginRequest, QueryResult, RateLimitStatus,
        TrinoQuery, TrinoStatementStats, ColumnDef, Claims, ErrorResponse
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "query", description = "Trino query endpoints"),
        (name = "admin", description = "Admin-only endpoints"),
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

    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let payload: LoginRequest =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let role = if payload.password.starts_with("admin") {
        "admin"
    } else if payload.password.starts_with("analyst") {
        "analyst"
    } else if payload.password.starts_with("viewer") {
        "viewer"
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: payload.username.clone(),
        role: role.to_string(),
        tenant_id: "tenant-a".to_string(),
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
    Extension(limiter): Extension<Arc<TokenBucketRateLimiter>>,
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
    let trino_config = TrinoConfig::default();
    let trino = Arc::new(TrinoRouter::new(trino_config));
    let rate_limit_config = RateLimitConfig::default();
    let rate_limiter = Arc::new(TokenBucketRateLimiter::new(rate_limit_config));
    let tenant_store = Arc::new(TenantStore::new());
    let audit_log = Arc::new(AuditLog::new("audit-logs"));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let grpc_service = Arc::new(GrpcQueryService::new(Arc::clone(&trino)));
    let grpc_service_for_server = (*grpc_service).clone();

    let admin_routes = Router::new()
        .route("/admin/action", post(admin_action))
        .route("/admin/users", post(manage_users))
        .route_layer(axum_middleware::from_fn_with_state(
            require_admin(jwt_config.clone()),
            authz_middleware,
        ))
        .layer(axum::middleware::from_fn(scan_quota_middleware));

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
        .layer(Extension(jwt_config))
        .layer(Extension(Arc::clone(&trino)))
        .layer(Extension(grpc_service))
        .layer(Extension(Arc::clone(&rate_limiter)))
        .layer(Extension(Arc::clone(&tenant_store)))
        .layer(Extension(Arc::clone(&audit_log)))
        .layer(cors)
        .layer(axum::middleware::from_fn(trace_middleware))
        .layer(axum::middleware::from_fn(audit_middleware))
        .layer(axum::middleware::from_fn(tenant_isolation_middleware))
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
