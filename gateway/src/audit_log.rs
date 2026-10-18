use axum::{extract::Request, middleware::Next, response::Response};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::auth::{verify_token, JwtConfig};

const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub sequence_number: u64,
    pub timestamp: String,
    pub tenant_id: String,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub client_ip: String,
    pub user_agent: String,
    pub prev_entry_hash: String,
    pub entry_hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntryPayload {
    pub sequence_number: u64,
    pub timestamp: String,
    pub tenant_id: String,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub client_ip: String,
    pub user_agent: String,
    pub prev_entry_hash: String,
}

impl AuditEntryPayload {
    /// Canonical structured JSON encoding for cryptographic hashing and signing
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn compute_hash(&self) -> String {
        format!("{:x}", Sha256::digest(self.to_canonical_bytes()))
    }
}

pub struct AuditLog {
    log_path: PathBuf,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    /// Thread-safe writer state: protects (sequence_number, last_entry_hash) and file writes
    state: Arc<Mutex<(u64, String)>>,
}

impl AuditLog {
    pub fn new(log_dir: &str) -> Self {
        let log_path = PathBuf::from(log_dir).join("audit.log");
        fs::create_dir_all(log_dir).expect("Failed to create audit log directory");

        let signing_key = Self::load_or_generate_key(log_dir);
        let verifying_key = signing_key.verifying_key();

        // Restore sequence number and last hash from existing log file if present
        let (initial_seq, initial_hash) = Self::restore_chain_state(&log_path);

        info!("Audit log initialized at {:?}", log_path);
        info!(
            "Audit chain state restored: seq={}, last_hash={}",
            initial_seq, initial_hash
        );

        Self {
            log_path,
            signing_key,
            verifying_key,
            state: Arc::new(Mutex::new((initial_seq, initial_hash))),
        }
    }

    pub fn get_verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    pub fn verifying_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    fn restore_chain_state(log_path: &PathBuf) -> (u64, String) {
        if !log_path.exists() {
            return (0, GENESIS_HASH.to_string());
        }

        let file = match fs::File::open(log_path) {
            Ok(f) => f,
            Err(_) => return (0, GENESIS_HASH.to_string()),
        };

        let reader = BufReader::new(file);
        let mut last_seq = 0;
        let mut last_hash = GENESIS_HASH.to_string();

        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
                last_seq = entry.sequence_number;
                last_hash = entry.entry_hash;
            }
        }

        (last_seq, last_hash)
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

        let key_hex = hex::encode(bytes);
        fs::write(&key_path, key_hex).expect("Failed to write audit key");

        signing_key
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
        let mut state = self.state.lock().await;
        let (ref mut last_seq, ref mut last_hash) = *state;

        let seq = *last_seq + 1;
        let prev_hash = last_hash.clone();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let payload = AuditEntryPayload {
            sequence_number: seq,
            timestamp: timestamp.clone(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            status,
            response_time_ms,
            client_ip: client_ip.to_string(),
            user_agent: user_agent.to_string(),
            prev_entry_hash: prev_hash.clone(),
        };

        let entry_hash = payload.compute_hash();
        let signature = self.signing_key.sign(entry_hash.as_bytes());

        let entry = AuditEntry {
            sequence_number: seq,
            timestamp,
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            status,
            response_time_ms,
            client_ip: client_ip.to_string(),
            user_agent: user_agent.to_string(),
            prev_entry_hash: prev_hash,
            entry_hash: entry_hash.clone(),
            signature: hex::encode(signature.to_bytes()),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .expect("Failed to open audit log");

        let line = serde_json::to_string(&entry).expect("Failed to serialize audit entry");
        writeln!(file, "{}", line).expect("Failed to write audit entry");

        *last_seq = seq;
        *last_hash = entry_hash;

        entry
    }

    /// Verifies the cryptographic integrity of the entire audit log.
    /// Checks:
    /// 1. Sequential unbroken sequence numbers (guards against deletion/skipping)
    /// 2. Previous entry hash link (guards against truncation/reordering)
    /// 3. Canonical payload SHA-256 match (guards against field tampering)
    /// 4. Ed25519 signature on entry hash (guards against unauthorized forging)
    pub async fn verify_log(&self) -> Result<Vec<AuditEntry>, Vec<String>> {
        let file = match fs::File::open(&self.log_path) {
            Ok(f) => f,
            Err(e) => return Err(vec![format!("Failed to open log: {}", e)]),
        };

        let reader = BufReader::new(file);
        let mut verified = Vec::new();
        let mut errors = Vec::new();

        let mut expected_seq = 1u64;
        let mut expected_prev_hash = GENESIS_HASH.to_string();

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

            // 1. Sequence number continuity
            if entry.sequence_number != expected_seq {
                errors.push(format!(
                    "Line {}: sequence break: expected {}, got {} (possible deleted or inserted record)",
                    line_num + 1,
                    expected_seq,
                    entry.sequence_number
                ));
            }

            // 2. Hash chain link
            if entry.prev_entry_hash != expected_prev_hash {
                errors.push(format!(
                    "Line {}: broken hash chain at seq {}: prev_entry_hash does not match preceding entry",
                    line_num + 1,
                    entry.sequence_number
                ));
            }

            // 3. Payload integrity check
            let payload = AuditEntryPayload {
                sequence_number: entry.sequence_number,
                timestamp: entry.timestamp.clone(),
                tenant_id: entry.tenant_id.clone(),
                user_id: entry.user_id.clone(),
                method: entry.method.clone(),
                path: entry.path.clone(),
                status: entry.status,
                response_time_ms: entry.response_time_ms,
                client_ip: entry.client_ip.clone(),
                user_agent: entry.user_agent.clone(),
                prev_entry_hash: entry.prev_entry_hash.clone(),
            };

            let computed_hash = payload.compute_hash();
            if computed_hash != entry.entry_hash {
                errors.push(format!(
                    "Line {}: payload hash mismatch at seq {}: data has been tampered with",
                    line_num + 1,
                    entry.sequence_number
                ));
            }

            // 4. Ed25519 signature check
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
                    errors.push(format!(
                        "Line {}: invalid signature format: {}",
                        line_num + 1,
                        e
                    ));
                    continue;
                }
            };

            if let Err(e) = self
                .verifying_key
                .verify(entry.entry_hash.as_bytes(), &signature)
            {
                errors.push(format!(
                    "Line {}: signature verification failed at seq {}: {}",
                    line_num + 1,
                    entry.sequence_number,
                    e
                ));
            }

            expected_seq = entry.sequence_number + 1;
            expected_prev_hash = entry.entry_hash.clone();
            verified.push(entry);
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

    // Secure IP extraction: only accept X-Forwarded-For if explicitly configured via TRUSTED_PROXIES
    let client_ip = {
        let direct_peer = request
            .extensions()
            .get::<std::net::SocketAddr>()
            .map(|a| a.to_string());

        let trust_proxy = std::env::var("TRUST_PROXY")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        if trust_proxy {
            request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|ip| ip.trim().to_string())
                .or(direct_peer)
                .unwrap_or_else(|| "127.0.0.1".to_string())
        } else {
            direct_peer.unwrap_or_else(|| "127.0.0.1".to_string())
        }
    };

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
    async fn test_audit_log_write_and_verify_chain() {
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
        assert_eq!(entries[0].sequence_number, 1);
        assert_eq!(entries[0].prev_entry_hash, GENESIS_HASH);
        assert_eq!(entries[1].sequence_number, 2);
        assert_eq!(entries[1].prev_entry_hash, entries[0].entry_hash);
    }

    #[tokio::test]
    async fn test_tamper_detection_on_field_modification() {
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

        // Attacker modifies status 200 -> 404 in log file
        let mut content = fs::read_to_string(&log_path).unwrap();
        content = content.replace("\"status\":200", "\"status\":404");
        fs::write(&log_path, &content).unwrap();

        let audit2 = AuditLog::new(tmp.path().to_str().unwrap());
        let result = audit2.verify_log().await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("data has been tampered with"));
    }

    #[tokio::test]
    async fn test_tamper_detection_on_record_deletion() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        let audit = AuditLog::new(tmp.path().to_str().unwrap());
        audit
            .log_request("t1", "u1", "GET", "/1", 200, 10, "127.0.0.1", "agent")
            .await;
        audit
            .log_request("t1", "u1", "GET", "/2", 200, 10, "127.0.0.1", "agent")
            .await;
        audit
            .log_request("t1", "u1", "GET", "/3", 200, 10, "127.0.0.1", "agent")
            .await;

        drop(audit);

        // Delete line 2 (middle entry)
        let lines: Vec<String> = fs::read_to_string(&log_path)
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        let modified = format!("{}\n{}\n", lines[0], lines[2]);
        fs::write(&log_path, modified).unwrap();

        let audit2 = AuditLog::new(tmp.path().to_str().unwrap());
        let result = audit2.verify_log().await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("sequence break") || e.contains("broken hash chain")));
    }
}
