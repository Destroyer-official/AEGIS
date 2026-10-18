use axum::http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

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
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, config }
    }

    pub async fn execute_query(
        &self,
        sql: &str,
        auth: &AuthUser,
    ) -> Result<QueryResult, StatusCode> {
        auth.require_query()?;

        let full_sql = self.wrap_query(sql);
        let url = format!("{}/v1/statement", self.config.coordinator_url);

        info!(
            "Executing Trino query: {}",
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

    fn wrap_query(&self, sql: &str) -> String {
        let trimmed = sql.trim().to_uppercase();
        if trimmed.starts_with("SELECT")
            || trimmed.starts_with("WITH")
            || trimmed.starts_with("SHOW")
            || trimmed.starts_with("DESCRIBE")
        {
            sql.to_string()
        } else {
            format!("SELECT * FROM ({}) LIMIT 1000", sql)
        }
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
    let sql = trino_query.query;
    let limit = 1000;
    let sql = if sql.to_uppercase().contains("LIMIT") {
        sql
    } else {
        format!("{} LIMIT {}", sql.trim(), limit)
    };

    let result = trino.execute_query(&sql, &user).await?;
    Ok(axum::Json(result))
}
