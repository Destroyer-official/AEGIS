use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Per-signature statistics counters.
///
/// **Stability**: Stable since v1.3.0.
#[derive(Debug, Default)]
pub struct SignatureStats {
    pub true_positives: AtomicU64,
    pub false_positives: AtomicU64,
    pub false_negatives: AtomicU64,
}

impl SignatureStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_tp(&self) {
        self.true_positives.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_fp(&self) {
        self.false_positives.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_fn(&self) {
        self.false_negatives.fetch_add(1, Ordering::Relaxed);
    }

    pub fn to_json(&self) -> String {
        let tp = self.true_positives.load(Ordering::Relaxed);
        let fp = self.false_positives.load(Ordering::Relaxed);
        let fn_ = self.false_negatives.load(Ordering::Relaxed);
        let total = tp + fp;
        let precision = if total > 0 {
            tp as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        let recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64 * 100.0
        } else {
            0.0
        };
        format!(
            r#"{{"true_positives":{},"false_positives":{},"false_negatives":{},"precision":{:.2},"recall":{:.2}}}"#,
            tp, fp, fn_, precision, recall
        )
    }
}

/// Snapshot of stats for JSON serialization.
///
/// **Stability**: Stable since v1.3.0.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatsSnapshot {
    pub timestamp: String,
    pub engine_version: u64,
    pub total_queries: u64,
    pub per_signature: Vec<SignatureEntry>,
    pub totals: SignatureEntry,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureEntry {
    pub protocol: String,
    pub category: String,
    pub true_positives: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub precision: f64,
    pub recall: f64,
}

/// Embedded HTTP server for the FP/FN dashboard.
///
/// Serves JSON stats on `GET /classify/stats`. Zero new dependencies —
/// uses only `std::net::TcpListener` and `serde_json` (already in deps).
///
/// # Endpoints
///
/// | Method | Path | Description |
/// |--------|------|-------------|
/// | GET | `/classify/stats` | Full per-signature stats JSON |
/// | GET | `/health` | Health check |
///
/// **Stability**: Stable since v1.3.0.
pub struct StatsServer {
    listener: TcpListener,
    stats: Arc<StatsRegistry>,
    engine_version: Arc<AtomicU64>,
}

/// Thread-safe registry of per-signature stats.
///
/// **Stability**: Stable since v1.3.0.
pub struct StatsRegistry {
    inner: Mutex<StatsRegistryInner>,
}

impl Default for StatsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

struct StatsRegistryInner {
    signatures: Vec<SignatureEntry>,
    total_queries: u64,
}

impl StatsRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(StatsRegistryInner {
                signatures: Vec::new(),
                total_queries: 0,
            }),
        }
    }

    pub fn register_signature(&self, protocol: &str, category: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.signatures.push(SignatureEntry {
            protocol: protocol.to_string(),
            category: category.to_string(),
            true_positives: 0,
            false_positives: 0,
            false_negatives: 0,
            precision: 0.0,
            recall: 0.0,
        });
    }

    pub fn record_tp(&self, protocol: &str) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.signatures.iter_mut().find(|s| s.protocol == protocol) {
            entry.true_positives += 1;
            update_precision_recall(entry);
        }
        inner.total_queries += 1;
    }

    pub fn record_fp(&self, protocol: &str) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.signatures.iter_mut().find(|s| s.protocol == protocol) {
            entry.false_positives += 1;
            update_precision_recall(entry);
        }
    }

    pub fn record_fn(&self, protocol: &str) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.signatures.iter_mut().find(|s| s.protocol == protocol) {
            entry.false_negatives += 1;
            update_precision_recall(entry);
        }
    }

    pub fn snapshot(&self, engine_version: u64) -> StatsSnapshot {
        let inner = self.inner.lock().unwrap();

        let totals = inner.signatures.iter().fold(
            SignatureEntry {
                protocol: "ALL".to_string(),
                category: "ALL".to_string(),
                true_positives: 0,
                false_positives: 0,
                false_negatives: 0,
                precision: 0.0,
                recall: 0.0,
            },
            |mut acc, s| {
                acc.true_positives += s.true_positives;
                acc.false_positives += s.false_positives;
                acc.false_negatives += s.false_negatives;
                acc
            },
        );

        let tp = totals.true_positives;
        let fp = totals.false_positives;
        let fn_ = totals.false_negatives;
        let mut totals = totals;
        totals.precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64 * 100.0
        } else {
            0.0
        };
        totals.recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64 * 100.0
        } else {
            0.0
        };

        StatsSnapshot {
            timestamp: chrono_now(),
            engine_version,
            total_queries: inner.total_queries,
            per_signature: inner.signatures.clone(),
            totals,
        }
    }
}

fn update_precision_recall(entry: &mut SignatureEntry) {
    let total = entry.true_positives + entry.false_positives;
    entry.precision = if total > 0 {
        entry.true_positives as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    let denom = entry.true_positives + entry.false_negatives;
    entry.recall = if denom > 0 {
        entry.true_positives as f64 / denom as f64 * 100.0
    } else {
        0.0
    };
}

impl StatsServer {
    /// Create a new stats server bound to the given address.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self {
            listener,
            stats: Arc::new(StatsRegistry::new()),
            engine_version: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Create from an existing listener (for testing).
    ///
    /// **Stability**: Unstable — test helper.
    pub fn from_listener(listener: TcpListener) -> Self {
        Self {
            listener,
            stats: Arc::new(StatsRegistry::new()),
            engine_version: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Get a reference to the stats registry.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn registry(&self) -> &Arc<StatsRegistry> {
        &self.stats
    }

    /// Set the engine version for the stats snapshot.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn set_engine_version(&self, version: u64) {
        self.engine_version.store(version, Ordering::Relaxed);
    }

    /// Get the bound address.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn addr(&self) -> std::net::SocketAddr {
        self.listener.local_addr().unwrap()
    }

    /// Run the stats server in the current thread (blocking).
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn run(&self) -> Result<(), std::io::Error> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let stats = Arc::clone(&self.stats);
                    let version = self.engine_version.load(Ordering::Relaxed);
                    thread::spawn(move || {
                        handle_connection(stream, &stats, version);
                    });
                }
                Err(e) => {
                    eprintln!("[stats-server] connection error: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Run the stats server in a background thread.
    ///
    /// Returns the thread handle and the bound address.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn spawn(self) -> Result<(thread::JoinHandle<()>, std::net::SocketAddr), std::io::Error> {
        let addr = self.listener.local_addr()?;
        let handle = thread::spawn(move || {
            self.run().ok();
        });
        Ok((handle, addr))
    }
}

fn handle_connection(mut stream: TcpStream, stats: &Arc<StatsRegistry>, engine_version: u64) {
    let reader = BufReader::new(stream.try_clone().unwrap());

    let mut method = String::new();
    let mut path = String::new();

    let mut lines = reader.lines();
    if let Some(Ok(request_line)) = lines.next() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            method = parts[0].to_string();
            path = parts[1].to_string();
        }
    }

    // Skip headers
    for line in lines {
        match line {
            Ok(line) if line.is_empty() => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }

    let (status, body) = match (method.as_str(), path.as_str()) {
        ("GET", "/classify/stats") => {
            let snapshot = stats.snapshot(engine_version);
            match serde_json::to_string_pretty(&snapshot) {
                Ok(json) => ("200 OK", json),
                Err(_) => (
                    "500 Internal Server Error",
                    r#"{"error":"serialization failed"}"#.to_string(),
                ),
            }
        }
        ("GET", "/health") => ("200 OK", r#"{"status":"ok"}"#.to_string()),
        _ => ("404 Not Found", r#"{"error":"not found"}"#.to_string()),
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).ok();
}

/// Minimal chrono-like timestamp (zero deps).
fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple epoch seconds format
    format!("epoch:{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpStream;

    fn http_get_once(addr: std::net::SocketAddr, path: &str) -> std::io::Result<(String, String)> {
        let mut stream = TcpStream::connect(addr)?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            path
        );
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        let parts: Vec<&str> = response.split("\r\n\r\n").collect();
        let status_line = parts.first().unwrap_or(&"").to_string();
        let body = if parts.len() > 1 { parts[1] } else { "" };

        // Parse status code
        let status = status_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("000")
            .to_string();

        Ok((status, body.to_string()))
    }

    /// GET with retry — the server thread may not have entered its accept loop
    /// yet, so a single attempt can race under parallel test load.
    fn http_get(addr: std::net::SocketAddr, path: &str) -> (String, String) {
        let mut last_err = None;
        for _ in 0..100 {
            match http_get_once(addr, path) {
                Ok(v) => return v,
                Err(e) => {
                    last_err = Some(e);
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        panic!("http_get to {} failed: {:?}", path, last_err);
    }

    #[test]
    fn test_health_endpoint() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = StatsServer::from_listener(listener);

        thread::spawn(move || server.run().ok());
        thread::sleep(std::time::Duration::from_millis(50));

        let (status, body) = http_get(addr, "/health");
        assert_eq!(status, "200");
        assert!(body.contains("ok"));
    }

    #[test]
    fn test_stats_endpoint_empty() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = StatsServer::from_listener(listener);

        thread::spawn(move || server.run().ok());
        thread::sleep(std::time::Duration::from_millis(50));

        let (status, body) = http_get(addr, "/classify/stats");
        assert_eq!(status, "200");
        assert!(body.contains("per_signature"));
        assert!(body.contains("totals"));
        assert!(body.contains("epoch:"));
    }

    #[test]
    fn test_stats_with_data() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = StatsServer::from_listener(listener);

        server.registry().register_signature("HTTP/1.1", "web");
        server.registry().register_signature("SSH", "remote");
        server.registry().record_tp("HTTP/1.1");
        server.registry().record_tp("HTTP/1.1");
        server.registry().record_fp("HTTP/1.1");
        server.registry().record_fn("SSH");

        let _handle = thread::spawn(move || server.run().ok());
        thread::sleep(std::time::Duration::from_millis(50));

        let (status, body) = http_get(addr, "/classify/stats");
        assert_eq!(status, "200");

        let snapshot: StatsSnapshot = serde_json::from_str(&body).unwrap();
        assert_eq!(snapshot.per_signature.len(), 2);
        assert_eq!(snapshot.totals.true_positives, 2);
        assert_eq!(snapshot.totals.false_positives, 1);
        assert_eq!(snapshot.totals.false_negatives, 1);

        // precision = 2/(2+1) = 66.67%
        assert!((snapshot.totals.precision - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_404() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = StatsServer::from_listener(listener);

        thread::spawn(move || server.run().ok());
        thread::sleep(std::time::Duration::from_millis(50));

        let (status, body) = http_get(addr, "/nonexistent");
        assert_eq!(status, "404");
        assert!(body.contains("not found"));
    }

    #[test]
    fn test_registry_counters() {
        let registry = StatsRegistry::new();
        registry.register_signature("HTTP/1.1", "web");

        registry.record_tp("HTTP/1.1");
        registry.record_tp("HTTP/1.1");
        registry.record_fp("HTTP/1.1");
        registry.record_fn("HTTP/1.1");

        let snapshot = registry.snapshot(1);
        let http = &snapshot.per_signature[0];
        assert_eq!(http.true_positives, 2);
        assert_eq!(http.false_positives, 1);
        assert_eq!(http.false_negatives, 1);
        assert!((http.precision - 66.67).abs() < 0.1);
        assert!((http.recall - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_engine_version_in_snapshot() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = StatsServer::from_listener(listener);
        server.set_engine_version(42);

        let _handle = thread::spawn(move || server.run().ok());
        thread::sleep(std::time::Duration::from_millis(50));

        let (_status, body) = http_get(addr, "/classify/stats");
        let snapshot: StatsSnapshot = serde_json::from_str(&body).unwrap();
        assert_eq!(snapshot.engine_version, 42);
    }
}
