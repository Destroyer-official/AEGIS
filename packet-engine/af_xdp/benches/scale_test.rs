//! Synthetic throughput test against Kafka (raw-l4-scans topic).
//! Targets 1M / 5M / 10M / 20M pps sustained, measures packet loss and CPU.
//!
//! Linux-only: requires AF_XDP-capable NIC + Kafka at `AEGIS_KAFKA_BROKER`.
//!
//! Run:
//!   cargo bench --bench scale_test --release
//!
//! Environment variables:
//!   AEGIS_IFACE       — NIC name           (default: eth0)
//!   AEGIS_KAFKA_BROKER — Kafka bootstrap   (default: localhost:9092)
//!   AEGIS_XDP_PROG    — path to xdp_redirect.o
//!
//! Outputs results to `benches/scale_results.csv`.

#[cfg(target_os = "linux")]
const BATCH_SIZE: u32 = 64;
#[cfg(target_os = "linux")]
const WARMUP_SECS: u64 = 2;
#[cfg(target_os = "linux")]
const MEASURE_SECS: u64 = 5;
#[cfg(target_os = "linux")]
const COOLDOWN_SECS: u64 = 1;

#[cfg(target_os = "linux")]
struct ScaleResult {
    target_label: String,
    target_pps: u64,
    actual_pps: f64,
    packets_sent: u64,
    packets_lost: u64,
    loss_pct: f64,
    cpu_pct: f64,
    duration_ms: u64,
    passed: bool,
}

#[cfg(target_os = "linux")]
impl ScaleResult {
    fn csv_header() -> String {
        "target_label,target_pps,actual_pps,packets_sent,packets_lost,loss_pct,cpu_pct,duration_ms,passed".into()
    }

    fn to_csv(&self) -> String {
        format!(
            "{},{},{:.0},{},{},{:.4},{:.1},{},{}",
            self.target_label,
            self.target_pps,
            self.actual_pps,
            self.packets_sent,
            self.packets_lost,
            self.loss_pct,
            self.cpu_pct,
            self.duration_ms,
            self.passed
        )
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_usage() -> f64 {
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        let line = stat.lines().next().unwrap_or("");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let user: u64 = parts[1].parse().unwrap_or(0);
            let nice: u64 = parts[2].parse().unwrap_or(0);
            let system: u64 = parts[3].parse().unwrap_or(0);
            let idle: u64 = parts[4].parse().unwrap_or(0);
            let total = user + nice + system + idle;
            let busy = user + nice + system;
            if total > 0 {
                return busy as f64 / total as f64 * 100.0;
            }
        }
    }
    0.0
}

#[cfg(target_os = "linux")]
fn run_scale_test() {
    use aegis_af_xdp::build_syn_packet;
    use aegis_sequence::SeqKey;
    use aegis_traversal::CyclicPermutation;
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let broker = std::env::var("AEGIS_KAFKA_BROKER").unwrap_or_else(|_| "localhost:9092".into());

    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &broker)
        .set("linger.ms", "0")
        .set("batch.num.messages", "1")
        .set("queue.buffering.max.messages", "1000000")
        .set("message.send.max.retries", "0")
        .create()
        .expect("Failed to create Kafka producer");

    let topic = "raw-l4-scans";

    let seq_key = SeqKey::from_bytes(&[
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08,
    ]);
    let mut perm = CyclicPermutation::new_default(1).expect("perm failed");

    let total_sent = Arc::new(AtomicU64::new(0));
    let total_loss = Arc::new(AtomicU64::new(0));

    let target_rates: Vec<(u64, &str)> = vec![
        (1_000_000, "1M"),
        (5_000_000, "5M"),
        (10_000_000, "10M"),
        (20_000_000, "20M"),
    ];

    let mut results: Vec<ScaleResult> = Vec::new();

    for (target_pps, label) in target_rates {
        println!("\n=== Scale test: {} pps target ===", label);

        total_sent.store(0, Ordering::Relaxed);
        total_loss.store(0, Ordering::Relaxed);

        let inter_pkt_ns = 1_000_000_000u64 / target_pps;

        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            if elapsed > Duration::from_secs(WARMUP_SECS + MEASURE_SECS + COOLDOWN_SECS) {
                break;
            }

            if elapsed < Duration::from_secs(WARMUP_SECS) {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }

            let batch_deadline =
                Instant::now() + Duration::from_nanos(inter_pkt_ns * BATCH_SIZE as u64);

            for _ in 0..BATCH_SIZE {
                if let Some(ip_u32) = perm.next_ipv4() {
                    let dst_ip = std::net::Ipv4Addr::from(ip_u32);
                    let dst_port = 80u16;
                    let ts = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        & 0xFFFFFFFF) as u32;
                    let seq = seq_key.encode_ipv4(dst_ip, dst_port, ts);
                    let pkt = build_syn_packet(
                        dst_ip,
                        dst_port,
                        std::net::Ipv4Addr::new(192, 168, 1, 1),
                        12345,
                        seq,
                    );

                    let key = format!("{}:{}", dst_ip, dst_port);
                    let record = BaseRecord::to(topic).key(&key).payload(&pkt);
                    if producer.send(record).is_ok() {
                        total_sent.fetch_add(1, Ordering::Relaxed);
                    } else {
                        total_loss.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            producer.poll(0);

            let remaining = batch_deadline.saturating_duration_since(Instant::now());
            if !remaining.is_zero() {
                std::thread::sleep(remaining);
            }
        }

        producer.flush(Duration::from_secs(2));

        let measure_start = start + Duration::from_secs(WARMUP_SECS);
        let measure_end = measure_start + Duration::from_secs(MEASURE_SECS);
        let actual_duration = measure_end.duration_since(measure_start).as_secs_f64();

        let sent = total_sent.load(Ordering::Relaxed);
        let lost = total_loss.load(Ordering::Relaxed);
        let actual_pps = sent as f64 / actual_duration;
        let loss_pct = if sent + lost > 0 {
            lost as f64 / (sent + lost) as f64 * 100.0
        } else {
            0.0
        };

        let cpu_pct = read_cpu_usage();
        let passed = loss_pct < 2.0 && cpu_pct < 80.0;

        println!(
            "  target={} pps | actual={:.0} pps | sent={} | lost={} | loss={:.2}% | cpu={:.1}% | {}",
            label, actual_pps, sent, lost, loss_pct, cpu_pct,
            if passed { "PASS" } else { "FAIL" }
        );

        results.push(ScaleResult {
            target_label: label.to_string(),
            target_pps,
            actual_pps,
            packets_sent: sent,
            packets_lost: lost,
            loss_pct,
            cpu_pct,
            duration_ms: MEASURE_SECS * 1000,
            passed,
        });

        std::thread::sleep(Duration::from_secs(2));
    }

    let csv_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("scale_results.csv");

    let mut f = File::create(&csv_path).expect("Cannot create CSV");
    writeln!(f, "{}", ScaleResult::csv_header()).unwrap();
    for r in &results {
        writeln!(f, "{}", r.to_csv()).unwrap();
    }
    println!("\nResults written to {}", csv_path.display());

    let all_passed = results.iter().all(|r| r.passed);
    println!(
        "\n=== Overall: {} ===",
        if all_passed {
            "ALL PASSED"
        } else {
            "SOME FAILED"
        }
    );

    assert!(
        results.iter().any(|r| r.passed),
        "At least one scale test target must pass"
    );
}

#[cfg(not(target_os = "linux"))]
fn run_scale_test() {
    eprintln!("Scale tests require Linux with AF_XDP support.");
    eprintln!("Build and run on the scanning cluster node.");
}

fn main() {
    run_scale_test();
}
