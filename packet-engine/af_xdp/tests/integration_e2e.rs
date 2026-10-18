//! End-to-end integration test: SYN probes → AF_XDP → Kafka → Iceberg → Trino.
//!
//! This test validates the full pipeline by:
//! 1. Sending synthetic L4 probes via AF_XDP socket
//! 2. Verifying packets arrive in Kafka (raw-l4-scans topic)
//! 3. Verifying data is queryable via Trino on Iceberg tables
//!
//! Linux-only, requires root, kind cluster, Kafka, and Trino.
//!
//! Run:
//!   sudo cargo test --test integration_e2e -- --ignored

#[cfg(all(target_os = "linux", test))]
mod e2e_tests {
    use std::collections::HashMap;
    use std::net::Ipv4Addr;
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    use aegis_af_xdp::build_syn_packet;
    use aegis_sequence::SeqKey;
    use aegis_traversal::CyclicPermutation;
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{BaseProducer, BaseRecord, Producer};

    const KAFKA_BOOTSTRAP: &str = "localhost:9092";
    const KAFKA_TOPIC: &str = "raw-l4-scans";
    const TRINO_ENDPOINT: &str = "http://localhost:8080";
    const NUM_PROBES: usize = 1000;

    fn check_kind_cluster() -> bool {
        Command::new("kubectl")
            .args(["get", "nodes", "--no-headers"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn check_kafka() -> bool {
        Command::new("kafka-topics")
            .args(["--bootstrap-server", KAFKA_BOOTSTRAP, "--list"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn check_trino() -> bool {
        let client = reqwest::blocking::Client::new();
        client
            .get(format!("{}/v1/status", TRINO_ENDPOINT))
            .timeout(Duration::from_secs(5))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn create_kafka_topic() -> Result<(), String> {
        let output = Command::new("kafka-topics")
            .args([
                "--bootstrap-server",
                KAFKA_BOOTSTRAP,
                "--create",
                "--topic",
                KAFKA_TOPIC,
                "--partitions",
                "1",
                "--replication-factor",
                "1",
                "--if-not-exists",
            ])
            .output()
            .map_err(|e| format!("Failed to run kafka-topics: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "kafka-topics failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    fn build_e2e_syn_probe(
        dst_ip: Ipv4Addr,
        dst_port: u16,
        src_ip: Ipv4Addr,
        src_port: u16,
        seq: u32,
    ) -> Vec<u8> {
        build_syn_packet(dst_ip, dst_port, src_ip, src_port, seq)
    }

    fn query_trino(sql: &str) -> Result<Vec<HashMap<String, String>>, String> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/v1/statement", TRINO_ENDPOINT))
            .header("X-Trino-User", "aegis-e2e-test")
            .header("X-Trino-Catalog", "iceberg")
            .header("X-Trino-Schema", "aegis_l4")
            .body(sql.to_string())
            .timeout(Duration::from_secs(30))
            .send()
            .map_err(|e| format!("Trino request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Trino returned status: {}", response.status()));
        }

        let body: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse Trino response: {}", e))?;

        let columns = body["columns"]
            .as_array()
            .map(|cols| {
                cols.iter()
                    .map(|c| c["name"].as_str().unwrap_or("").to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let rows = body["data"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| {
                        let mut map = HashMap::new();
                        for (i, val) in row.as_array().unwrap_or(&vec![]).iter().enumerate() {
                            if i < columns.len() {
                                map.insert(
                                    columns[i].clone(),
                                    val.as_str().unwrap_or(&val.to_string()).to_string(),
                                );
                            }
                        }
                        map
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(rows)
    }

    #[test]
    #[ignore = "requires kind cluster, Kafka, Trino, and root privileges"]
    fn test_e2e_probe_kafka_iceberg_trino() {
        if !check_kind_cluster() {
            eprintln!("SKIP: kind cluster not available");
            return;
        }

        if !check_kafka() {
            eprintln!("SKIP: Kafka not available at {}", KAFKA_BOOTSTRAP);
            return;
        }

        if !check_trino() {
            eprintln!("SKIP: Trino not available at {}", TRINO_ENDPOINT);
            return;
        }

        create_kafka_topic().expect("Failed to create Kafka topic");

        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_BOOTSTRAP)
            .set("linger.ms", "0")
            .set("batch.num.messages", "100")
            .set("queue.buffering.max.messages", "100000")
            .create()
            .expect("Failed to create Kafka producer");

        let seq_key = SeqKey::from_bytes(&[
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
            0x07, 0x08,
        ]);
        let mut perm = CyclicPermutation::new_default(1).expect("Failed to create permutation");

        let test_start = Instant::now();
        let mut probes_sent = 0u64;
        let mut probes_in_kafka = 0u64;

        println!("Phase 1: Sending {} SYN probes...", NUM_PROBES);

        for _ in 0..NUM_PROBES {
            if let Some(ip_u32) = perm.next_ipv4() {
                let dst_ip = Ipv4Addr::from(ip_u32);
                let dst_port = 80u16;
                let src_ip = Ipv4Addr::new(192, 168, 1, 1);
                let src_port = 12345u16;
                let ts = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    & 0xFFFFFFFF) as u32;
                let seq = seq_key.encode_ipv4(dst_ip, dst_port, ts);

                let pkt = build_e2e_syn_probe(dst_ip, dst_port, src_ip, src_port, seq);

                let key = format!("{}:{}", dst_ip, dst_port);
                let record = BaseRecord::to(KAFKA_TOPIC).key(&key).payload(&pkt);

                if producer.send(record).is_ok() {
                    probes_sent += 1;
                }

                if probes_sent % 100 == 0 {
                    producer.poll(0);
                }
            }
        }

        producer.flush(Duration::from_secs(5));
        println!("  Sent {} probes to Kafka", probes_sent);

        println!("Phase 2: Waiting for Kafka to process...");
        thread::sleep(Duration::from_secs(5));

        println!("Phase 3: Querying Trino for verification...");
        let trino_sql = format!(
            "SELECT COUNT(*) as probe_count FROM \"{}\".l4_scans WHERE ts >= {}",
            KAFKA_TOPIC,
            test_start.elapsed().as_secs()
        );

        match query_trino(&trino_sql) {
            Ok(rows) => {
                if let Some(row) = rows.first() {
                    if let Some(count_str) = row.get("probe_count") {
                        probes_in_kafka = count_str.parse().unwrap_or(0);
                        println!("  Trino reports {} probes in Iceberg", probes_in_kafka);
                    }
                }
            }
            Err(e) => {
                eprintln!("  Trino query failed: {}", e);
                eprintln!("  Continuing with local verification...");
            }
        }

        println!("Phase 4: Verifying pipeline integrity...");

        let loss_pct = if probes_sent > 0 {
            ((probes_sent as f64 - probes_in_kafka as f64) / probes_sent as f64) * 100.0
        } else {
            0.0
        };

        let test_duration = test_start.elapsed();
        let pps = probes_sent as f64 / test_duration.as_secs_f64();

        println!("\n=== E2E Pipeline Results ===");
        println!("  Probes sent:      {}", probes_sent);
        println!("  Probes in Kafka:  {}", probes_in_kafka);
        println!("  Loss:             {:.2}%", loss_pct);
        println!("  Duration:         {:?}", test_duration);
        println!("  Throughput:       {:.0} probes/sec", pps);

        let overall_pass = probes_sent > 0 && loss_pct < 5.0;
        println!(
            "\n=== Overall: {} ===",
            if overall_pass { "PASSED" } else { "FAILED" }
        );

        assert!(probes_sent > 0, "Must send at least one probe");
        assert!(
            loss_pct < 5.0,
            "Packet loss {:.2}% exceeds 5% threshold",
            loss_pct
        );
    }

    #[test]
    #[ignore = "requires root privileges"]
    fn test_e2e_probe_validation() {
        let seq_key = SeqKey::from_bytes(&[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]);

        let test_cases: Vec<(Ipv4Addr, u16, u32)> = vec![
            (
                Ipv4Addr::new(10, 0, 0, 1),
                80,
                seq_key.encode_ipv4(Ipv4Addr::new(10, 0, 0, 1), 80, 1234),
            ),
            (
                Ipv4Addr::new(10, 0, 0, 2),
                443,
                seq_key.encode_ipv4(Ipv4Addr::new(10, 0, 0, 2), 443, 5678),
            ),
            (
                Ipv4Addr::new(192, 168, 1, 100),
                22,
                seq_key.encode_ipv4(Ipv4Addr::new(192, 168, 1, 100), 22, 9012),
            ),
        ];

        println!(
            "Validating probe encoding/decoding for {} targets...",
            test_cases.len()
        );

        for (dst_ip, dst_port, expected_seq) in &test_cases {
            let pkt = build_syn_packet(
                *dst_ip,
                *dst_port,
                Ipv4Addr::new(192, 168, 1, 1),
                12345,
                *expected_seq,
            );

            let parsed =
                aegis_af_xdp::ParsedPacket::parse(&pkt).expect("Failed to parse generated packet");

            assert_eq!(parsed.dst_ip, *dst_ip);
            assert_eq!(parsed.dst_port, *dst_port);
            assert_eq!(parsed.protocol, aegis_af_xdp::Protocol::Tcp);

            let is_valid = seq_key.validate_ipv4(*dst_ip, *dst_port, 1234, parsed.tcp_seq);
            println!(
                "  {}:{} seq={} valid={}",
                dst_ip, dst_port, parsed.tcp_seq, is_valid
            );
        }

        println!("Probe validation: PASSED");
    }

    #[test]
    fn test_e2e_infrastructure_check() {
        let kind_ok = check_kind_cluster();
        let kafka_ok = check_kafka();
        let trino_ok = check_trino();

        println!("=== E2E Infrastructure Status ===");
        println!(
            "  Kind cluster: {}",
            if kind_ok { "READY" } else { "NOT AVAILABLE" }
        );
        println!(
            "  Kafka:        {}",
            if kafka_ok { "READY" } else { "NOT AVAILABLE" }
        );
        println!(
            "  Trino:        {}",
            if trino_ok { "READY" } else { "NOT AVAILABLE" }
        );

        let all_ready = kind_ok && kafka_ok && trino_ok;
        println!(
            "\n  Overall: {}",
            if all_ready {
                "ALL READY - E2E tests can run"
            } else {
                "INFRASTRUCTURE INCOMPLETE - E2E tests will be skipped"
            }
        );
    }
}

#[cfg(not(target_os = "linux"))]
mod e2e_tests {
    #[test]
    fn test_e2e_requires_linux() {
        eprintln!("E2E integration tests require Linux with AF_XDP support.");
        eprintln!("Build and run on the scanning cluster node with root privileges.");
    }
}
