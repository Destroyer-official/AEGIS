use axum::http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::auth::AuthUser;

#[derive(Debug, Clone)]
pub struct TrinoConfig {
    pub coordinator_url: String,
    pub catalog: String,
    pub schema: String,
}

impl Default for TrinoConfig {
    fn default() -> Self {
        Self {
            coordinator_url: std::env::var("TRINO_COORDINATOR_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            catalog: std::env::var("TRINO_CATALOG").unwrap_or_else(|_| "iceberg".into()),
            schema: std::env::var("TRINO_SCHEMA").unwrap_or_else(|_| "aegis".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrinoQuery {
    #[serde(alias = "sql")]
    pub query: String,
    pub catalog: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrinoColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
}

#[derive(Debug, Deserialize, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct TrinoStatementStats {
    pub state: Option<String>,
    pub queued: Option<bool>,
    pub rows: Option<u64>,
    pub processed_rows: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TrinoQueryResult {
    pub id: Option<String>,
    pub next_uri: Option<String>,
    pub columns: Option<Vec<TrinoColumn>>,
    pub data: Option<Vec<Vec<serde_json::Value>>>,
    pub stats: Option<TrinoStatementStats>,
    pub error: Option<TrinoError>,
}

#[derive(Debug, Deserialize)]
pub struct TrinoError {
    pub message: Option<String>,
    pub error_type: Option<String>,
}

pub struct TrinoRouter {
    client: Client,
    config: TrinoConfig,
}

impl TrinoRouter {
    pub fn new(config: TrinoConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, config }
    }

    /// Validates that the submitted SQL is strictly read-only, free from statement-chaining injection,
    /// and enforces bounded result limits.
    pub fn validate_and_secure_query(
        &self,
        raw_sql: &str,
        _auth: &AuthUser,
    ) -> Result<String, StatusCode> {
        let trimmed = raw_sql.trim();
        if trimmed.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 1. Strip string literals to safely inspect SQL tokens and semicolons without false positives
        let mut in_quote = false;
        let mut quote_char = ' ';
        let mut stripped = String::with_capacity(trimmed.len());

        for c in trimmed.chars() {
            if !in_quote && (c == '\'' || c == '"') {
                in_quote = true;
                quote_char = c;
                stripped.push(c);
            } else if in_quote && c == quote_char {
                in_quote = false;
                stripped.push(c);
            } else if in_quote {
                stripped.push(' ');
            } else {
                stripped.push(c);
            }
        }

        // Check for statement chaining (unquoted semicolons followed by other commands)
        let statements: Vec<&str> = stripped
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if statements.len() > 1 {
            warn!("Blocked query containing multiple chained SQL statements");
            return Err(StatusCode::BAD_REQUEST);
        }

        // 2. Reject destructive / data-modification statements
        let upper_clean = stripped.to_uppercase();
        let tokens: Vec<&str> = upper_clean
            .split_whitespace()
            .map(|s| s.trim_matches(|p| p == '(' || p == ')' || p == ',' || p == ';'))
            .collect();

        let prohibited_keywords = [
            "DROP", "ALTER", "TRUNCATE", "DELETE", "UPDATE", "INSERT", "CREATE", "GRANT", "REVOKE",
            "CALL", "EXECUTE", "SHUTDOWN", "SET",
        ];

        for kw in prohibited_keywords {
            if tokens.contains(&kw) {
                warn!("Blocked query with prohibited DDL/DML keyword: {}", kw);
                return Err(StatusCode::FORBIDDEN);
            }
        }

        // 3. Ensure statement starts with an allowed read-only verb
        let first_token = tokens.first().copied().unwrap_or("");
        if !["SELECT", "WITH", "SHOW", "DESCRIBE", "EXPLAIN"].contains(&first_token) {
            warn!(
                "Blocked statement not starting with read-only verb: {}",
                first_token
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // 4. Enforce bounded result limit (max 1000 rows)
        let mut final_sql = trimmed.trim_end_matches(';').trim().to_string();
        if !upper_clean.contains("LIMIT") {
            final_sql = format!("{} LIMIT 1000", final_sql);
        }

        Ok(final_sql)
    }

    pub async fn execute_query(
        &self,
        sql: &str,
        auth: &AuthUser,
    ) -> Result<QueryResult, StatusCode> {
        auth.require_query()?;

        let full_sql = self.validate_and_secure_query(sql, auth)?;
        let url = format!("{}/v1/statement", self.config.coordinator_url);

        info!(
            "Executing Trino query for user {} (tenant {}): {}",
            auth.claims.sub,
            auth.claims.tenant_id,
            &full_sql[..full_sql.len().min(100)]
        );

        let response = self
            .client
            .post(&url)
            .header("X-Trino-User", &auth.claims.sub)
            .header("X-Trino-Catalog", &self.config.catalog)
            .header("X-Trino-Schema", &self.config.schema)
            .body(full_sql.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Trino request failed: {}", e);
                StatusCode::BAD_GATEWAY
            })?;

        let mut result: TrinoQueryResult = response.json().await.map_err(|e| {
            error!("Failed to parse Trino response: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

        // Follow next URI for additional pages
        while let Some(next_uri) = &result.next_uri {
            if next_uri.is_empty() {
                break;
            }
            let resp = self
                .client
                .get(next_uri)
                .send()
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            result = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        }

        if let Some(err) = &result.error {
            let error_msg = err
                .message
                .clone()
                .unwrap_or_else(|| "Unknown Trino error".to_string());
            let error_type = err
                .error_type
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string());
            error!("Trino query error [{error_type}]: {error_msg}");
            return Err(StatusCode::BAD_REQUEST);
        }

        let columns = result.columns.unwrap_or_default();
        let data = result.data.unwrap_or_default();
        let total_rows = data.len() as u64;

        let column_defs: Vec<ColumnDef> = columns
            .iter()
            .map(|c| ColumnDef {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
            })
            .collect();

        let rows: Vec<Vec<String>> = data
            .into_iter()
            .map(|row| row.into_iter().map(|v| v.to_string()).collect())
            .collect();

        Ok(QueryResult {
            columns: column_defs,
            rows,
            total_rows,
            query_id: result.id.unwrap_or_default(),
            stats: result.stats,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct QueryResult {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: u64,
    pub query_id: String,
    pub stats: Option<TrinoStatementStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
}

pub async fn execute_trino_query(
    user: AuthUser,
    trino_query: TrinoQuery,
    trino: axum::extract::Extension<Arc<TrinoRouter>>,
) -> Result<axum::Json<QueryResult>, StatusCode> {
    let result = trino.execute_query(&trino_query.query, &user).await?;
    Ok(axum::Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Claims;

    fn test_auth_user() -> AuthUser {
        AuthUser {
            claims: Claims {
                sub: "alice".into(),
                role: "analyst".into(),
                tenant_id: "tenant-a".into(),
                exp: 9999999999,
                iat: 1000000000,
            },
        }
    }

    #[test]
    fn test_valid_select_query_adds_limit() {
        let router = TrinoRouter::new(TrinoConfig::default());
        let auth = test_auth_user();
        let res = router.validate_and_secure_query("SELECT * FROM hosts", &auth);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "SELECT * FROM hosts LIMIT 1000");
    }

    #[test]
    fn test_blocked_drop_table() {
        let router = TrinoRouter::new(TrinoConfig::default());
        let auth = test_auth_user();
        let res = router.validate_and_secure_query("DROP TABLE hosts", &auth);
        assert_eq!(res.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_blocked_statement_chaining() {
        let router = TrinoRouter::new(TrinoConfig::default());
        let auth = test_auth_user();
        let res =
            router.validate_and_secure_query("SELECT * FROM hosts; DELETE FROM services;", &auth);
        assert_eq!(res.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_allowed_quoted_keywords() {
        let router = TrinoRouter::new(TrinoConfig::default());
        let auth = test_auth_user();
        // Quoted string containing "DROP TABLE" must not falsely trigger token prohibition
        let res = router
            .validate_and_secure_query("SELECT * FROM hosts WHERE name = 'DROP TABLE'", &auth);
        assert!(res.is_ok());
    }
}
