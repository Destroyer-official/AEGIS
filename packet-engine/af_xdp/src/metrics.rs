//! Embedded Prometheus metrics server for AF_XDP probe engine.
//!
//! Serves `/metrics` on `:9090` with zero external dependencies.
//! Exposes: pps, drop_rate, ring_utilization, bpf_map_usage.
//!
//! Usage:
//!   let server = MetricsServer::start("0.0.0.0:9090");
//!   server.inc_rx_packets(100);
//!   server.set_ring_utilization(0.75);

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const RESPONSE_TEMPLATE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nConnection: close\r\n\r\n";

/// Atomic counters for Prometheus exposition.
#[derive(Debug, Default)]
pub struct ProbeMetrics {
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
    pub rx_bytes: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub rx_drops: AtomicU64,
    pub tx_drops: AtomicU64,
    pub parse_errors: AtomicU64,
    pub kafka_send_ok: AtomicU64,
    pub kafka_send_err: AtomicU64,
    pub bpf_map_inserts: AtomicU64,
    pub bpf_map_lookups: AtomicU64,
    pub fill_ring_util: AtomicU64,
    pub rx_ring_util: AtomicU64,
    pub tx_ring_util: AtomicU64,
    pub completion_ring_util: AtomicU64,
    pub uptime_start: AtomicU64,
}

impl ProbeMetrics {
    pub fn new() -> Self {
        let m = Self::default();
        m.uptime_start
            .store(elapsed_secs_since_epoch(), Ordering::Relaxed);
        m
    }

    pub fn inc_rx_packets(&self, n: u64) {
        self.rx_packets.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_tx_packets(&self, n: u64) {
        self.tx_packets.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_rx_bytes(&self, n: u64) {
        self.rx_bytes.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_tx_bytes(&self, n: u64) {
        self.tx_bytes.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_rx_drops(&self, n: u64) {
        self.rx_drops.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_tx_drops(&self, n: u64) {
        self.tx_drops.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_parse_errors(&self, n: u64) {
        self.parse_errors.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_kafka_send_ok(&self, n: u64) {
        self.kafka_send_ok.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_kafka_send_err(&self, n: u64) {
        self.kafka_send_err.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_bpf_map_inserts(&self, n: u64) {
        self.bpf_map_inserts.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_bpf_map_lookups(&self, n: u64) {
        self.bpf_map_lookups.fetch_add(n, Ordering::Relaxed);
    }

    pub fn set_ring_util(&self, fill: u64, rx: u64, tx: u64, completion: u64) {
        self.fill_ring_util.store(fill, Ordering::Relaxed);
        self.rx_ring_util.store(rx, Ordering::Relaxed);
        self.tx_ring_util.store(tx, Ordering::Relaxed);
        self.completion_ring_util
            .store(completion, Ordering::Relaxed);
    }

    fn render(&self) -> String {
        let mut out = String::with_capacity(2048);

        out.push_str("# HELP aegis_rx_packets Total received packets.\n");
        out.push_str("# TYPE aegis_rx_packets counter\n");
        out.push_str(&format!(
            "aegis_rx_packets {}\n",
            self.rx_packets.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_tx_packets Total transmitted packets.\n");
        out.push_str("# TYPE aegis_tx_packets counter\n");
        out.push_str(&format!(
            "aegis_tx_packets {}\n",
            self.tx_packets.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_rx_bytes Total received bytes.\n");
        out.push_str("# TYPE aegis_rx_bytes counter\n");
        out.push_str(&format!(
            "aegis_rx_bytes {}\n",
            self.rx_bytes.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_tx_bytes Total transmitted bytes.\n");
        out.push_str("# TYPE aegis_tx_bytes counter\n");
        out.push_str(&format!(
            "aegis_tx_bytes {}\n",
            self.tx_bytes.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_rx_drops Total dropped receive packets.\n");
        out.push_str("# TYPE aegis_rx_drops counter\n");
        out.push_str(&format!(
            "aegis_rx_drops {}\n",
            self.rx_drops.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_tx_drops Total dropped transmit packets.\n");
        out.push_str("# TYPE aegis_tx_drops counter\n");
        out.push_str(&format!(
            "aegis_tx_drops {}\n",
            self.tx_drops.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_parse_errors Total packet parse errors.\n");
        out.push_str("# TYPE aegis_parse_errors counter\n");
        out.push_str(&format!(
            "aegis_parse_errors {}\n",
            self.parse_errors.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_kafka_send_ok Total successful Kafka sends.\n");
        out.push_str("# TYPE aegis_kafka_send_ok counter\n");
        out.push_str(&format!(
            "aegis_kafka_send_ok {}\n",
            self.kafka_send_ok.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_kafka_send_err Total failed Kafka sends.\n");
        out.push_str("# TYPE aegis_kafka_send_err counter\n");
        out.push_str(&format!(
            "aegis_kafka_send_err {}\n",
            self.kafka_send_err.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_bpf_map_inserts Total BPF map inserts.\n");
        out.push_str("# TYPE aegis_bpf_map_inserts counter\n");
        out.push_str(&format!(
            "aegis_bpf_map_inserts {}\n",
            self.bpf_map_inserts.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_bpf_map_lookups Total BPF map lookups.\n");
        out.push_str("# TYPE aegis_bpf_map_lookups counter\n");
        out.push_str(&format!(
            "aegis_bpf_map_lookups {}\n",
            self.bpf_map_lookups.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_fill_ring_util Fill ring utilization (0-100).\n");
        out.push_str("# TYPE aegis_fill_ring_util gauge\n");
        out.push_str(&format!(
            "aegis_fill_ring_util {}\n",
            self.fill_ring_util.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_rx_ring_util RX ring utilization (0-100).\n");
        out.push_str("# TYPE aegis_rx_ring_util gauge\n");
        out.push_str(&format!(
            "aegis_rx_ring_util {}\n",
            self.rx_ring_util.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_tx_ring_util TX ring utilization (0-100).\n");
        out.push_str("# TYPE aegis_tx_ring_util gauge\n");
        out.push_str(&format!(
            "aegis_tx_ring_util {}\n",
            self.tx_ring_util.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_completion_ring_util Completion ring utilization (0-100).\n");
        out.push_str("# TYPE aegis_completion_ring_util gauge\n");
        out.push_str(&format!(
            "aegis_completion_ring_util {}\n",
            self.completion_ring_util.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP aegis_uptime_seconds Process uptime in seconds.\n");
        out.push_str("# TYPE aegis_uptime_seconds gauge\n");
        let start = self.uptime_start.load(Ordering::Relaxed);
        let now = elapsed_secs_since_epoch();
        let uptime = now.saturating_sub(start);
        out.push_str(&format!("aegis_uptime_seconds {}\n", uptime));

        out
    }
}

fn elapsed_secs_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Lightweight HTTP metrics server (zero external deps).
pub struct MetricsServer {
    metrics: Arc<ProbeMetrics>,
    handle: Option<thread::JoinHandle<()>>,
    running: Option<Arc<AtomicBool>>,
}

impl MetricsServer {
    pub fn start(bind_addr: &str) -> Self {
        let metrics = Arc::new(ProbeMetrics::new());
        let metrics_clone = Arc::clone(&metrics);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let listener = TcpListener::bind(bind_addr)
            .unwrap_or_else(|e| panic!("Failed to bind metrics server on {}: {}", bind_addr, e));

        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        let handle = thread::spawn(move || {
            listener_loop(listener, metrics_clone, running_clone);
        });

        Self {
            metrics,
            handle: Some(handle),
            running: Some(running),
        }
    }

    pub fn start_with_metrics(bind_addr: &str, metrics: Arc<ProbeMetrics>) -> Self {
        let metrics_clone = Arc::clone(&metrics);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let listener = TcpListener::bind(bind_addr)
            .unwrap_or_else(|e| panic!("Failed to bind metrics server on {}: {}", bind_addr, e));

        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        let handle = thread::spawn(move || {
            listener_loop(listener, metrics_clone, running_clone);
        });

        Self {
            metrics,
            handle: Some(handle),
            running: Some(running),
        }
    }

    pub fn metrics(&self) -> &Arc<ProbeMetrics> {
        &self.metrics
    }

    pub fn shutdown(mut self) {
        if let Some(running) = self.running.take() {
            running.store(false, Ordering::Relaxed);
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for MetricsServer {
    fn drop(&mut self) {
        if let Some(running) = self.running.take() {
            running.store(false, Ordering::Relaxed);
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn listener_loop(listener: TcpListener, metrics: Arc<ProbeMetrics>, running: Arc<AtomicBool>) {
    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let metrics = Arc::clone(&metrics);
                thread::spawn(move || {
                    handle_connection(stream, &metrics);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, metrics: &ProbeMetrics) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line);

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let body = match path {
        "/metrics" => metrics.render(),
        "/health" => "OK\n".to_string(),
        "/ready" => "READY\n".to_string(),
        _ => {
            let _ =
                stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found");
            return;
        }
    };

    let response = format!(
        "{}Content-Length: {}\r\n\r\n{}",
        RESPONSE_TEMPLATE,
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpStream;

    #[test]
    fn test_metrics_render() {
        let m = ProbeMetrics::new();
        m.inc_rx_packets(100);
        m.inc_tx_packets(50);
        m.inc_rx_drops(5);

        let rendered = m.render();
        assert!(rendered.contains("aegis_rx_packets 100"));
        assert!(rendered.contains("aegis_tx_packets 50"));
        assert!(rendered.contains("aegis_rx_drops 5"));
        assert!(rendered.contains("# TYPE aegis_rx_packets counter"));
    }

    #[test]
    fn test_metrics_server_start_stop() {
        let server = MetricsServer::start("127.0.0.1:19090");
        std::thread::sleep(Duration::from_millis(100));

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:19090") {
            let _ = stream.write_all(
                b"GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            );
            let mut response = vec![0u8; 4096];
            let n = stream.read(&mut response).unwrap_or(0);
            response.truncate(n);
            let body = String::from_utf8_lossy(&response);
            assert!(body.contains("aegis_rx_packets"));
        }

        server.shutdown();
    }

    #[test]
    fn test_health_endpoint() {
        let server = MetricsServer::start("127.0.0.1:19091");
        std::thread::sleep(Duration::from_millis(100));

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:19091") {
            let _ = stream
                .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
            let mut response = vec![0u8; 4096];
            let n = stream.read(&mut response).unwrap_or(0);
            response.truncate(n);
            let body = String::from_utf8_lossy(&response);
            assert!(body.contains("OK"));
        }

        server.shutdown();
    }

    #[test]
    fn test_uptime_counter() {
        let m = ProbeMetrics::new();
        let rendered = m.render();
        assert!(rendered.contains("aegis_uptime_seconds"));
        assert!(rendered.contains("# TYPE aegis_uptime_seconds gauge"));
    }
}
