#[cfg(all(test, feature = "kafka"))]
mod integration_tests {
    use super::*;
    use crate::{Feature, HostObservation, OptimizedMatrix, PredictionMethod, TargetPrediction};

    #[tokio::test]
    async fn test_scan_command_generation() {
        // Build a test matrix with realistic data
        let mut matrix = OptimizedMatrix::new();

        for i in 0..1000 {
            matrix
                .observe_batch_parallel(&[HostObservation {
                    ip: i,
                    open_ports: vec![80, 443, 22],
                    features: vec![
                        Feature::PortHttp,
                        Feature::PortHttps,
                        Feature::PortSsh,
                        Feature::AppHttpServer,
                        Feature::AppTls,
                        Feature::AppSshBanner,
                        Feature::NetCloudProvider,
                        Feature::NetAsn,
                    ],
                    asn: 16509,
                }])
                .unwrap();
        }
        matrix.build().unwrap();

        // Generate predictions
        let target_ips: Vec<u32> = (1000..2000).collect();
        let network_features = vec![Feature::NetCloudProvider, Feature::NetAsn];

        let mut all_commands = Vec::new();
        let source = aegis_proto::TargetSource::TargetSourceGpsCoOccurrence;
        let now = chrono::Utc::now().to_rfc3339();

        for ip in target_ips {
            if let Ok(Some((port, prob))) = matrix.predict_first_service(&network_features) {
                let pred = TargetPrediction {
                    ip,
                    asn: 16509,
                    predicted_ports: vec![(port, prob)],
                    source_features: network_features.clone(),
                    prediction_method: PredictionMethod::FirstService,
                };

                if let Ok(remaining) = matrix.predict_remaining_services(&network_features, 0.1) {
                    let mut all_ports = vec![(port, prob)];
                    all_ports.extend(remaining);

                    let pred = TargetPrediction {
                        ip,
                        asn: 16509,
                        predicted_ports: all_ports,
                        source_features: network_features.clone(),
                        prediction_method: PredictionMethod::RemainingService,
                    };

                    let mut feature_tuple: Vec<u32> = pred
                        .source_features
                        .iter()
                        .map(|f| f.index() as u32)
                        .collect();
                    for (p, _) in &pred.predicted_ports {
                        feature_tuple.push(*p as u32);
                    }

                    let cmd = aegis_proto::ScanCommand {
                        command_id: (chrono::Utc::now().timestamp_millis() as u64) << 16
                            | (ip as u64),
                        target_ip: pred.ip.to_be_bytes().to_vec(),
                        port: pred
                            .predicted_ports
                            .first()
                            .map(|(p, _)| *p as u32)
                            .unwrap_or(0),
                        transport: aegis_proto::Transport::TransportTcp as i32,
                        priority: pred
                            .predicted_ports
                            .first()
                            .map(|(_, p)| *p as f64)
                            .unwrap_or(0.5),
                        source: aegis_proto::TargetSource::TargetSourceGpsCoOccurrence as i32,
                        conditional_probability: pred
                            .predicted_ports
                            .first()
                            .map(|(_, p)| *p as f64)
                            .unwrap_or(0.0),
                        asn: pred.asn,
                        feature_tuple,
                        generated_at: now.clone(),
                        cidr_block: String::new(),
                        asn_name: String::new(),
                    };
                    all_commands.push(cmd);
                }
            }
        }

        // Verify we generated commands
        assert!(!all_commands.is_empty(), "Should generate scan commands");

        // Verify command structure
        for cmd in &all_commands {
            assert_eq!(cmd.target_ip.len(), 4, "IP should be 4 bytes");
            assert!(cmd.port > 0, "Port should be set");
            assert!(cmd.priority > 0.0, "Priority should be positive");
            assert!(
                !cmd.feature_tuple.is_empty(),
                "Feature tuple should not be empty"
            );
            assert!(!cmd.generated_at.is_empty(), "Timestamp should be set");
        }

        println!("Generated {} scan commands", all_commands.len());
    }

    #[tokio::test]
    async fn test_kafka_producer_creation() {
        // Test that producer can be created with valid config
        let result =
            crate::kafka_integration::ScanCommandProducer::new("localhost:9092", "scan-commands");

        // Producer creation should not panic; result depends on rdkafka init
        let _ = result;
    }

    #[tokio::test]
    async fn test_prediction_to_command_conversion() {
        let preds = vec![TargetPrediction {
            ip: 0xC0A80001,
            asn: 16509,
            predicted_ports: vec![(443, 0.9), (80, 0.8), (22, 0.3)],
            source_features: vec![
                Feature::PortHttp,
                Feature::AppTls,
                Feature::NetCloudProvider,
            ],
            prediction_method: PredictionMethod::RemainingService,
        }];

        let cmds = crate::kafka_integration::predictions_to_commands(
            &preds,
            aegis_proto::TargetSource::TargetSourceGpsCoOccurrence,
        );

        assert_eq!(cmds.len(), 1);
        let cmd = &cmds[0];
        assert_eq!(cmd.target_ip, vec![192, 168, 0, 1]);
        assert_eq!(cmd.port, 443);
        assert!((cmd.priority - 0.9).abs() < f64::EPSILON);
        assert_eq!(
            cmd.source,
            aegis_proto::TargetSource::TargetSourceGpsCoOccurrence as i32
        );
        assert!((cmd.conditional_probability - 0.9).abs() < f64::EPSILON);
        assert!(!cmd.feature_tuple.is_empty());
    }

    #[tokio::test]
    async fn test_full_pipeline() {
        // Full integration test: matrix -> predictions -> commands
        let mut matrix = OptimizedMatrix::new();

        // Seed with 500 hosts
        for i in 0..500 {
            matrix
                .observe_batch_parallel(&[HostObservation {
                    ip: i,
                    open_ports: vec![80, 443],
                    features: vec![
                        Feature::PortHttp,
                        Feature::PortHttps,
                        Feature::AppHttpServer,
                        Feature::AppTls,
                        Feature::NetCloudProvider,
                        Feature::NetAsn,
                    ],
                    asn: 16509,
                }])
                .unwrap();
        }
        matrix.build().unwrap();

        let target_ips: Vec<u32> = (500..600).collect();
        let network_features = vec![Feature::NetCloudProvider, Feature::NetAsn];

        let source = aegis_proto::TargetSource::TargetSourceGpsCoOccurrence;
        let now = chrono::Utc::now().to_rfc3339();
        let mut total_commands = 0;

        for ip in target_ips {
            if let Ok(Some((port, prob))) = matrix.predict_first_service(&network_features) {
                let mut all_ports = vec![(port, prob)];
                if let Ok(remaining) = matrix.predict_remaining_services(&network_features, 0.1) {
                    all_ports.extend(remaining);
                }

                let pred = TargetPrediction {
                    ip,
                    asn: 16509,
                    predicted_ports: all_ports,
                    source_features: network_features.clone(),
                    prediction_method: PredictionMethod::RemainingService,
                };

                let mut feature_tuple: Vec<u32> = pred
                    .source_features
                    .iter()
                    .map(|f| f.index() as u32)
                    .collect();
                for (p, _) in &pred.predicted_ports {
                    feature_tuple.push(*p as u32);
                }

                let cmd = aegis_proto::ScanCommand {
                    command_id: (chrono::Utc::now().timestamp_millis() as u64) << 16 | (ip as u64),
                    target_ip: pred.ip.to_be_bytes().to_vec(),
                    port: pred
                        .predicted_ports
                        .first()
                        .map(|(p, _)| *p as u32)
                        .unwrap_or(0),
                    transport: aegis_proto::Transport::TransportTcp as i32,
                    priority: pred
                        .predicted_ports
                        .first()
                        .map(|(_, p)| *p as f64)
                        .unwrap_or(0.5),
                    source: source as i32,
                    conditional_probability: pred
                        .predicted_ports
                        .first()
                        .map(|(_, p)| *p as f64)
                        .unwrap_or(0.0),
                    asn: pred.asn,
                    feature_tuple,
                    generated_at: now.clone(),
                    cidr_block: String::new(),
                    asn_name: String::new(),
                };

                // Verify proto serialization works
                let bytes = prost::Message::encode_to_vec(cmd);
                assert!(
                    !bytes.is_empty(),
                    "Command should serialize to non-empty bytes"
                );

                let decoded = aegis_proto::ScanCommand::decode(bytes.as_slice()).unwrap();
                assert_eq!(decoded.target_ip, cmd.target_ip);
                assert_eq!(decoded.port, cmd.port);

                total_commands += 1;
            }
        }

        assert!(total_commands > 0, "Should generate at least one command");
        println!("Full pipeline test: generated {} commands", total_commands);
    }
}
