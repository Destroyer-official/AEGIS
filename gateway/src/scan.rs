use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ScanOptions {
    #[serde(default = "default_port_range")]
    pub port_range: String,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            port_range: default_port_range(),
            timeout: default_timeout(),
        }
    }
}

fn default_port_range() -> String {
    "1-1024".into()
}

fn default_timeout() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateScanRequest {
    pub target: String,
    #[serde(default = "default_scan_type")]
    pub scan_type: String,
    #[serde(default)]
    pub options: Option<ScanOptions>,
}

fn default_scan_type() -> String {
    "l4_port".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ScanJob {
    pub id: String,
    pub tenant_id: String,
    pub target: String,
    pub scan_type: String,
    pub port_range: String,
    pub timeout: u32,
    pub status: String,
    pub completed: u64,
    pub total: u64,
    pub elapsed: String,
    pub started_at: String,
}

pub struct ScanStore {
    scans: RwLock<Vec<ScanJob>>,
}

impl Default for ScanStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanStore {
    pub fn new() -> Self {
        Self {
            scans: RwLock::new(vec![ScanJob {
                id: "scan-baseline-001".to_string(),
                tenant_id: "tenant-a".to_string(),
                target: "192.168.1.0/24".to_string(),
                scan_type: "l4_port".to_string(),
                port_range: "1-1024".to_string(),
                timeout: 5,
                status: "completed".to_string(),
                completed: 256,
                total: 256,
                elapsed: "12s".to_string(),
                started_at: Utc::now().to_rfc3339(),
            }]),
        }
    }

    /// Validates target and dispatches a new scan job
    pub async fn create_scan(
        &self,
        tenant_id: &str,
        req: CreateScanRequest,
    ) -> Result<ScanJob, StatusCode> {
        let clean_target = req.target.trim().to_string();
        if clean_target.is_empty() || clean_target.len() > 256 {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate target characters (IPv4, IPv6, CIDR, valid DNS hostname)
        let is_valid_char = clean_target.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '/' || c == '-' || c == '_'
        });
        if !is_valid_char {
            return Err(StatusCode::BAD_REQUEST);
        }

        let opts = req.options.unwrap_or_default();
        let port_range = if opts.port_range.trim().is_empty() {
            "1-1024".to_string()
        } else {
            opts.port_range.trim().to_string()
        };

        let timeout = opts.timeout.clamp(1, 120);

        // Estimate total hosts based on CIDR
        let total = if clean_target.ends_with("/24") {
            256
        } else if clean_target.ends_with("/16") {
            65536
        } else if clean_target.contains('/') {
            512
        } else {
            1
        };

        let scan_id = format!("scan-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let job = ScanJob {
            id: scan_id,
            tenant_id: tenant_id.to_string(),
            target: clean_target,
            scan_type: req.scan_type,
            port_range,
            timeout,
            status: "running".to_string(),
            completed: 0,
            total,
            elapsed: "0s".to_string(),
            started_at: Utc::now().to_rfc3339(),
        };

        let mut scans = self.scans.write().await;
        scans.push(job.clone());
        Ok(job)
    }

    /// List all scans visible to the given tenant (or all for admin)
    pub async fn list_scans(&self, tenant_id: &str, is_admin: bool) -> Vec<ScanJob> {
        let scans = self.scans.read().await;
        if is_admin {
            scans.clone()
        } else {
            scans
                .iter()
                .filter(|s| s.tenant_id == tenant_id)
                .cloned()
                .collect()
        }
    }

    /// Cancel a running scan
    pub async fn cancel_scan(
        &self,
        scan_id: &str,
        tenant_id: &str,
        is_admin: bool,
    ) -> Result<ScanJob, StatusCode> {
        let mut scans = self.scans.write().await;
        let job = scans
            .iter_mut()
            .find(|s| s.id == scan_id)
            .ok_or(StatusCode::NOT_FOUND)?;

        if !is_admin && job.tenant_id != tenant_id {
            return Err(StatusCode::FORBIDDEN);
        }

        job.status = "stopped".to_string();
        Ok(job.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_lifecycle() {
        let store = ScanStore::new();

        let req = CreateScanRequest {
            target: "10.0.0.0/24".to_string(),
            scan_type: "l4_port".to_string(),
            options: Some(ScanOptions {
                port_range: "80,443".to_string(),
                timeout: 10,
            }),
        };

        let job = store.create_scan("tenant-a", req).await.unwrap();
        assert_eq!(job.tenant_id, "tenant-a");
        assert_eq!(job.target, "10.0.0.0/24");
        assert_eq!(job.status, "running");
        assert_eq!(job.total, 256);

        // List scans
        let tenant_scans = store.list_scans("tenant-a", false).await;
        assert!(tenant_scans.iter().any(|s| s.id == job.id));

        // Other tenant cannot see or cancel
        let other_scans = store.list_scans("tenant-b", false).await;
        assert!(!other_scans.iter().any(|s| s.id == job.id));

        let cancel_err = store.cancel_scan(&job.id, "tenant-b", false).await;
        assert_eq!(cancel_err.unwrap_err(), StatusCode::FORBIDDEN);

        // Valid cancel
        let cancelled = store.cancel_scan(&job.id, "tenant-a", false).await.unwrap();
        assert_eq!(cancelled.status, "stopped");
    }

    #[tokio::test]
    async fn test_invalid_target_rejected() {
        let store = ScanStore::new();
        let bad_req = CreateScanRequest {
            target: "10.0.0.1; rm -rf /".to_string(),
            scan_type: "l4_port".to_string(),
            options: None,
        };
        let err = store.create_scan("tenant-a", bad_req).await;
        assert_eq!(err.unwrap_err(), StatusCode::BAD_REQUEST);
    }
}
