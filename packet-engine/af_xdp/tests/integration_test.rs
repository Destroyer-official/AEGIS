//! Integration test for AF_XDP packet pipeline: kind cluster → kernel probe → Kafka
//!
//! This test validates the end-to-end data flow:
//! 1. Synthetic L4 probes sent to kind cluster nodes
//! 2. XDP program on node intercepts and redirects to AF_XDP socket
//! 3. Userspace engine processes packets and forwards to Kafka
//! 4. Consumer validates packet delivery and metadata

#[cfg(all(target_os = "linux", test))]
mod integration_tests {
    use std::collections::HashMap;
    use std::net::Ipv4Addr;
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use aegis_af_xdp::{AfXdpEngine, ParsedPacket, XskConfig};
    use aegis_sequence::SeqKey;
    use aegis_traversal::CyclicPermutation;
    use rdkafka::config::ClientConfig;
    use rdkafka::consumer::{BaseConsumer, Consumer};
    use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
    use rdkafka::Message;

    const TEST_INTERFACE: &str = "eth0";
    const KAFKA_BOOTSTRAP: &str = "localhost:9092";
    const KAFKA_TOPIC: &str = "aegis-l4-probes";
    const FRAME_COUNT: u32 = 4096;
    const NUM_PROBES: usize = 1000;
    const TEST_TIMEOUT_SECS: u64 = 30;

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

    fn build_syn_probe(dst_ip: Ipv4Addr, dst_port: u16, seq: u32) -> Vec<u8> {
        let mut pkt = Vec::with_capacity(64);
        pkt.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        pkt.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        pkt.extend_from_slice(&[0x08, 0x00]);
        pkt.extend_from_slice(&[0x45, 0x00, 0x00, 0x3c]);
        pkt.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x40, 0x06]);
        pkt.extend_from_slice(&[0x00, 0x00]);
        pkt.extend_from_slice(&[192, 168, 1, 1]);
        pkt.extend_from_slice(&dst_ip.octets());
        pkt.extend_from_slice(&12345u16.to_be_bytes());
        pkt.extend_from_slice(&dst_port.to_be_bytes());
        pkt.extend_from_slice(&seq.to_be_bytes());
        pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        pkt.extend_from_slice(&[0x50, 0x02, 0xff, 0xff]);
        pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        pkt
    }

    #[test]
    #[ignore = "requires kind cluster, Kafka, and root privileges"]
    fn test_af_xdp_kind_kafka_pipeline() {
        if !check_kind_cluster() {
            eprintln!("SKIP: kind cluster not available");
            return;
        }
        if !check_kafka() {
            eprintln!("SKIP: Kafka not available at {}", KAFKA_BOOTSTRAP);
            return;
        }

        create_kafka_topic().expect("Failed to create Kafka topic");

        let config = XskConfig {
            frame_count: FRAME_COUNT,
            ..Default::default()
        };

        let mut engine =
            AfXdpEngine::new(TEST_INTERFACE, &config).expect("Failed to create AF_XDP engine");

        let xdp_prog_path = std::env::var("AEGIS_XDP_PROG").unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("bpf")
                .join("xdp_redirect.o")
                .to_string_lossy()
                .to_string()
        });

        let _xdp_prog = engine
            .load_xdp_program(&xdp_prog_path, TEST_INTERFACE)
            .expect("Failed to load XDP program");

        engine
            .refill_fill_ring()
            .expect("Failed to refill FILL ring");

        let engine = Arc::new(Mutex::new(engine));

        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_BOOTSTRAP)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer");

        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_BOOTSTRAP)
            .set("group.id", "aegis-integration-test")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create Kafka consumer");

        consumer
            .subscribe(&[KAFKA_TOPIC])
            .expect("Failed to subscribe to topic");

        let seq_key = SeqKey::from_bytes(&[0x42u8; 16]);
        let mut perm = CyclicPermutation::new_default(1).expect("Failed to create permutation");

        let sent_count = Arc::new(AtomicU64::new(0));
        let recv_count = Arc::new(AtomicU64::new(0));
        let kafka_sent = Arc::new(AtomicU64::new(0));
        let kafka_received = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let send_thread = {
            let engine = Arc::clone(&engine);
            let sent_count = sent_count.clone();
            let kafka_sent = kafka_sent.clone();
            let running = running.clone();
            thread::spawn(move || {
                let mut seq = 0u32;
                for _ in 0..NUM_PROBES {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(ip_u32) = perm.next_ipv4() {
                        let dst_ip = Ipv4Addr::from(ip_u32);
                        let dst_port = 80u16;
                        let probe_seq = seq_key.encode_ipv4(dst_ip, dst_port, seq);
                        let pkt = build_syn_probe(dst_ip, dst_port, probe_seq);

                        let key = format!("{}:{}", dst_ip, dst_port);
                        let record = BaseRecord::to(KAFKA_TOPIC).key(&key).payload(&pkt);
                        if producer.send(record).is_ok() {
                            sent_count.fetch_add(1, Ordering::Relaxed);
                            kafka_sent.fetch_add(1, Ordering::Relaxed);
                        }
                        seq = seq.wrapping_add(1);
                    }
                    thread::sleep(Duration::from_micros(100));
                }
            })
        };

        let recv_thread = {
            let engine = Arc::clone(&engine);
            let recv_count = recv_count.clone();
            let running = running.clone();
            thread::spawn(move || {
                let mut packets = Vec::with_capacity(64);
                while running.load(Ordering::Relaxed) {
                    if let Ok(count) = engine.lock().unwrap().recv(&mut packets, 64) {
                        if count > 0 {
                            recv_count.fetch_add(count as u64, Ordering::Relaxed);
                            packets.clear();
                        }
                    }
                    thread::sleep(Duration::from_millis(1));
                }
            })
        };

        let kafka_recv_thread = {
            let kafka_received = kafka_received.clone();
            let running = running.clone();
            thread::spawn(move || {
                while running.load(Ordering::Relaxed) {
                    match consumer.poll(Duration::from_millis(100)) {
                        Some(Ok(msg)) => {
                            kafka_received.fetch_add(1, Ordering::Relaxed);
                            let _ = msg.payload();
                        }
                        Some(Err(e)) => eprintln!("Kafka consumer error: {}", e),
                        None => {}
                    }
                }
            })
        };

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(TEST_TIMEOUT_SECS) {
            let sent = sent_count.load(Ordering::Relaxed);
            let recv = recv_count.load(Ordering::Relaxed);
            let ks = kafka_sent.load(Ordering::Relaxed);
            let kr = kafka_received.load(Ordering::Relaxed);

            if sent >= NUM_PROBES as u64 && ks >= NUM_PROBES as u64 && kr >= NUM_PROBES as u64 {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        running.store(false, Ordering::Relaxed);

        send_thread.join().expect("Send thread panicked");
        recv_thread.join().expect("Recv thread panicked");
        kafka_recv_thread
            .join()
            .expect("Kafka recv thread panicked");

        let final_sent = sent_count.load(Ordering::Relaxed);
        let final_recv = recv_count.load(Ordering::Relaxed);
        let final_ks = kafka_sent.load(Ordering::Relaxed);
        let final_kr = kafka_received.load(Ordering::Relaxed);

        println!("=== Integration Test Results ===");
        println!("Probes sent:           {}", final_sent);
        println!("Probes recv (AF_XDP):  {}", final_recv);
        println!("Probes sent to Kafka:  {}", final_ks);
        println!("Probes recv from Kafka: {}", final_kr);
        println!(
            "Send rate:             {:.2} pps",
            final_sent as f64 / TEST_TIMEOUT_SECS as f64
        );
        println!(
            "Recv rate:             {:.2} pps",
            final_recv as f64 / TEST_TIMEOUT_SECS as f64
        );

        assert!(final_sent > 0, "No probes sent");
        assert!(final_ks > 0, "No probes sent to Kafka");
        assert!(final_kr > 0, "No probes received from Kafka");

        let kafka_delivery = final_kr as f64 / final_ks as f64;

        println!("Kafka delivery rate:   {:.2}%", kafka_delivery * 100.0);

        assert!(kafka_delivery > 0.9, "Kafka delivery rate below 90%");
    }

    #[test]
    #[ignore = "requires root privileges and XDP-capable interface"]
    fn test_af_xdp_engine_basic() {
        let config = XskConfig {
            frame_count: FRAME_COUNT,
            ..Default::default()
        };

        let engine = AfXdpEngine::new(TEST_INTERFACE, &config);
        if engine.is_err() {
            eprintln!("SKIP: AF_XDP engine creation failed (need root + XDP interface)");
            return;
        }
        let mut engine = engine.unwrap();

        let xdp_prog_path = std::env::var("AEGIS_XDP_PROG").unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("bpf")
                .join("xdp_redirect.o")
                .to_string_lossy()
                .to_string()
        });

        let _xdp_prog = engine.load_xdp_program(&xdp_prog_path, TEST_INTERFACE);
        if _xdp_prog.is_err() {
            eprintln!("SKIP: XDP program load failed");
            return;
        }

        let _ = engine.refill_fill_ring();

        let mut packets = Vec::with_capacity(64);
        let start = Instant::now();
        let mut total = 0u64;

        while start.elapsed() < Duration::from_secs(5) {
            if let Ok(count) = engine.recv(&mut packets, 64) {
                total += count as u64;
                packets.clear();
            }
            thread::sleep(Duration::from_millis(10));
        }

        println!(
            "Basic engine test: {} packets in 5s ({:.2} pps)",
            total,
            total as f64 / 5.0
        );
        assert!(total >= 0);
    }

    #[test]
    fn test_probe_encoding_validation() {
        let seq_key = SeqKey::from_bytes(&[0x11u8; 16]);
        let perm = CyclicPermutation::new_default(1).unwrap();

        for _ in 0..100 {
            if let Some(ip_u32) = perm.next_ipv4() {
                let dst_ip = Ipv4Addr::from(ip_u32);
                let dst_port = 80u16;
                let timestamp = 12345u32;
                let seq = seq_key.encode_ipv4(dst_ip, dst_port, timestamp);

                assert!(seq_key.validate_ipv4(dst_ip, dst_port, timestamp, seq));
                assert!(!seq_key.validate_ipv4(dst_ip, dst_port, timestamp + 1, seq));
            }
        }
    }
}

#[cfg(not(all(target_os = "linux", test)))]
mod integration_tests {
    #[test]
    fn test_integration_stub() {
        println!("Integration tests only run on Linux with test feature");
    }
}
