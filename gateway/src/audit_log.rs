use axum::{extract::Request, middleware::Next, response::Response};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::auth::{verify_token, JwtConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub tenant_id: String,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub client_ip: String,
    pub user_agent: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntryPayload {
    pub timestamp: String,
    pub tenant_id: String,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub client_ip: String,
    pub user_agent: String,
}

impl AuditEntryPayload {
    pub fn to_signing_bytes(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.timestamp,
            self.tenant_id,
            self.user_id,
            self.method,
            self.path,
            self.status,
            self.response_time_ms,
            self.client_ip,
            self.user_agent
        )
        .into_bytes()
    }
}

#[allow(dead_code)]
pub struct AuditLog {
    log_path: PathBuf,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    writer: Arc<Mutex<()>>,
}

impl AuditLog {
    pub fn new(log_dir: &str) -> Self {
        let log_path = PathBuf::from(log_dir).join("audit.log");
        fs::create_dir_all(log_dir).expect("Failed to create audit log directory");

        let signing_key = Self::load_or_generate_key(log_dir);
        let verifying_key = signing_key.verifying_key();

        info!("Audit log initialized at {:?}", log_path);
        info!("Verifying key: {:?}", verifying_key.to_bytes());

        Self {
            log_path,
            signing_key,
            verifying_key,
            writer: Arc::new(Mutex::new(())),
        }
    }

    fn load_or_generate_key(log_dir: &str) -> SigningKey {
        let key_path = PathBuf::from(log_dir).join("audit_key.pem");

        if key_path.exists() {
            let key_bytes = fs::read(&key_path).expect("Failed to read audit key");
            let key_hex = String::from_utf8(key_bytes).expect("Invalid key format");
            let decoded = hex::decode(key_hex.trim()).expect("Invalid hex key");
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&decoded);
            return SigningKey::from_bytes(&bytes);
        }

        let mut bytes = [0u8; 32];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);

        let key_hex = hex::encode(signing_key.to_bytes());
        fs::write(&key_path, key_hex).expect("Failed to write audit key");

        info!("Generated new audit signing key at {:?}", key_path);
        signing_key
    }

    pub fn get_verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    pub fn verifying_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn log_request(
        &self,
        tenant_id: &str,
        user_id: &str,
        method: &str,
        path: &str,
        status: u16,
        response_time_ms: u64,
        client_ip: &str,
        user_agent: &str,
    ) -> AuditEntry {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let payload = AuditEntryPayload {
            timestamp: timestamp.clone(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            status,
            response_time_ms,
            client_ip: client_ip.to_string(),
            user_agent: user_agent.to_string(),
        };

        let signing_bytes = payload.to_signing_bytes();
        let signature = self.signing_key.sign(&signing_bytes);

        let entry = AuditEntry {
            timestamp: payload.timestamp.clone(),
            tenant_id: payload.tenant_id.clone(),
            user_id: payload.user_id.clone(),
            method: payload.method.clone(),
            path: payload.path.clone(),
            status: payload.status,
            response_time_ms: payload.response_time_ms,
            client_ip: payload.client_ip.clone(),
            user_agent: payload.user_agent.clone(),
            signature: hex::encode(signature.to_bytes()),
        };

        let _lock = self.writer.lock().await;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .expect("Failed to open audit log");

        let line = serde_json::to_string(&entry).expect("Failed to serialize audit entry");
        writeln!(file, "{}", line).expect("Failed to write audit entry");

        entry
    }

    pub async fn verify_log(&self) -> Result<Vec<AuditEntry>, Vec<String>> {
        let file = fs::File::open(&self.log_path)
            .map_err(|e| vec![format!("Failed to open log: {}", e)])?;

        let reader = BufReader::new(file);
        let mut verified = Vec::new();
        let mut errors = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    errors.push(format!("Line {}: read error: {}", line_num + 1, e));
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("Line {}: parse error: {}", line_num + 1, e));
                    continue;
                }
            };

            let payload = AuditEntryPayload {
                timestamp: entry.timestamp.clone(),
                tenant_id: entry.tenant_id.clone(),
                user_id: entry.user_id.clone(),
                method: entry.method.clone(),
                path: entry.path.clone(),
                status: entry.status,
                response_time_ms: entry.response_time_ms,
                client_ip: entry.client_ip.clone(),
                user_agent: entry.user_agent.clone(),
            };

            let signing_bytes = payload.to_signing_bytes();
            let sig_bytes = match hex::decode(&entry.signature) {
                Ok(b) => b,
                Err(e) => {
                    errors.push(format!(
                        "Line {}: invalid signature hex: {}",
                        line_num + 1,
                        e
                    ));
                    continue;
                }
            };

            let signature = match Signature::from_slice(&sig_bytes) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(format!("Line {}: invalid signature: {}", line_num + 1, e));
                    continue;
                }
            };

            match self.verifying_key.verify(&signing_bytes, &signature) {
                Ok(()) => verified.push(entry),
                Err(e) => {
                    errors.push(format!(
                        "Line {}: signature verification failed: {}",
                        line_num + 1,
                        e
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(verified)
        } else {
            Err(errors)
        }
    }

    pub async fn export_entries(&self, tenant_id: Option<&str>) -> Vec<AuditEntry> {
        let file = match fs::File::open(&self.log_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
                if tenant_id.is_none_or(|tid| entry.tenant_id == tid) {
                    entries.push(entry);
                }
            }
        }

        entries
    }
}

pub async fn audit_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let client_ip = request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|a| a.to_string())
        .unwrap_or_else(|| {
            request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string()
        });
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let audit_log = request.extensions().get::<Arc<AuditLog>>().cloned();

    let jwt_config = request.extensions().get::<Arc<JwtConfig>>().cloned();

    let mut user_id = "anonymous".to_string();
    let mut tenant_id = "anonymous".to_string();

    if let (Some(config), Some(_audit)) = (&jwt_config, &audit_log) {
        if let Some(auth_header) = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Ok(claims) = verify_token(token, config) {
                    user_id = claims.sub.clone();
                    tenant_id = claims.tenant_id.clone();
                }
            }
        }
    }

    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let elapsed = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();

    if let Some(audit) = audit_log {
        audit
            .log_request(
                &tenant_id,
                &user_id,
                &method,
                &uri,
                status,
                elapsed,
                &client_ip,
                &user_agent,
            )
            .await;
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_log_write_and_verify() {
        let tmp = TempDir::new().unwrap();
        let audit = AuditLog::new(tmp.path().to_str().unwrap());

        audit
            .log_request(
                "tenant-a",
                "user1",
                "GET",
                "/health",
                200,
                15,
                "127.0.0.1",
                "test-agent",
            )
            .await;
        audit
            .log_request(
                "tenant-b", "user2", "POST", "/query", 200, 120, "10.0.0.1", "curl/7.0",
            )
            .await;

        let result = audit.verify_log().await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tenant_id, "tenant-a");
        assert_eq!(entries[1].tenant_id, "tenant-b");
    }

    #[tokio::test]
    async fn test_tenant_filter() {
        let tmp = TempDir::new().unwrap();
        let audit = AuditLog::new(tmp.path().to_str().unwrap());

        audit
            .log_request(
                "tenant-a",
                "user1",
                "GET",
                "/health",
                200,
                15,
                "127.0.0.1",
                "test",
            )
            .await;
        audit
            .log_request(
                "tenant-b",
                "user2",
                "GET",
                "/query",
                200,
                30,
                "127.0.0.1",
                "test",
            )
            .await;

        let all = audit.export_entries(None).await;
        assert_eq!(all.len(), 2);

        let tenant_a = audit.export_entries(Some("tenant-a")).await;
        assert_eq!(tenant_a.len(), 1);
        assert_eq!(tenant_a[0].tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn test_tamper_detection() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        let audit = AuditLog::new(tmp.path().to_str().unwrap());
        audit
            .log_request(
                "tenant-a",
                "user1",
                "GET",
                "/health",
                200,
                15,
                "127.0.0.1",
                "test",
            )
            .await;

        drop(audit);

        let mut content = fs::read_to_string(&log_path).unwrap();
        content = content.replace("200", "404");
        fs::write(&log_path, &content).unwrap();

        let audit2 = AuditLog::new(tmp.path().to_str().unwrap());
        let result = audit2.verify_log().await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("signature verification failed"));
    }
}
