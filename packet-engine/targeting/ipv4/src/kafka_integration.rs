use aegis_proto::ScanCommand;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use crate::{Feature, HostObservation, OptimizedMatrix, PredictionMethod, TargetPrediction};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Kafka config error: {0}")]
    Config(String),
    #[error("Kafka produce error: {0}")]
    Produce(#[from] rdkafka::error::KafkaError),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone)]
pub struct ScanCommandProducer {
    producer: FutureProducer,
    topic: String,
}

impl ScanCommandProducer {
    pub fn new(brokers: &str, topic: &str) -> Result<Self, KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("request.timeout.ms", "3000")
            .create()
            .map_err(|e| KafkaError::Config(e.to_string()))?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }

    /// Send a ScanCommand to the Kafka topic
    pub async fn send_command(&self, cmd: ScanCommand) -> Result<(), KafkaError> {
        let bytes = prost::Message::encode_to_vec(&cmd);
        let record = FutureRecord::to(&self.topic)
            .payload(&bytes)
            .key(&cmd.command_id.to_string());

        self.producer
            .send(record, Timeout::After(std::time::Duration::from_secs(5)))
            .await
            .map_err(|(e, _)| KafkaError::Produce(e))?;

        Ok(())
    }

    /// Send multiple commands in batch
    pub async fn send_batch(&self, commands: Vec<ScanCommand>) -> Result<(), KafkaError> {
        for cmd in commands {
            self.send_command(cmd).await?;
        }
        Ok(())
    }

    /// Flush pending messages
    pub async fn flush(&self) -> Result<(), KafkaError> {
        self.producer
            .flush(Timeout::After(std::time::Duration::from_secs(10)))
            .map_err(|e| KafkaError::Produce(e))
    }
}

/// Convert targeting predictions to ScanCommand protos
pub fn predictions_to_commands(
    predictions: &[TargetPrediction],
    source: aegis_proto::TargetSource,
) -> Vec<ScanCommand> {
    let now = chrono::Utc::now().to_rfc3339();

    predictions
        .iter()
        .enumerate()
        .map(|(i, pred)| {
            let mut feature_tuple: Vec<u32> = pred
                .source_features
                .iter()
                .map(|f| f.index() as u32)
                .collect();

            // Add predicted ports as feature tuple
            for (port, _) in &pred.predicted_ports {
                feature_tuple.push(*port as u32);
            }

            ScanCommand {
                command_id: (chrono::Utc::now().timestamp_millis() as u64) << 16 | (i as u64),
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
            }
        })
        .collect()
}

/// Generate scan commands from GPS matrix predictions for a set of target IPs
pub async fn generate_scan_commands_from_matrix(
    matrix: &OptimizedMatrix,
    target_ips: &[u32],
    producer: &ScanCommandProducer,
    network_features: &[Feature],
) -> Result<usize, KafkaError> {
    let mut all_commands = Vec::new();
    let source = if network_features.is_empty() {
        aegis_proto::TargetSource::TargetSourceGpsFirstService
    } else {
        aegis_proto::TargetSource::TargetSourceGpsCoOccurrence
    };

    for ip in target_ips {
        // Get first service prediction
        if let Ok(Some((port, prob))) = matrix.predict_first_service(network_features) {
            let pred = TargetPrediction {
                ip: *ip,
                asn: 0,
                predicted_ports: vec![(port, prob)],
                source_features: network_features.to_vec(),
                prediction_method: PredictionMethod::FirstService,
            };

            // Get remaining services
            if let Ok(remaining) = matrix.predict_remaining_services(network_features, 0.1) {
                let mut all_ports = vec![(port, prob)];
                all_ports.extend(remaining);

                let pred = TargetPrediction {
                    ip: *ip,
                    asn: 0,
                    predicted_ports: all_ports,
                    source_features: network_features.to_vec(),
                    prediction_method: PredictionMethod::RemainingService,
                };
                all_commands.extend(predictions_to_commands(&[pred], source));
            }
        }
    }

    producer.send_batch(all_commands.clone()).await?;
    Ok(all_commands.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Feature, HostObservation, OptimizedMatrix};

    fn make_observation(
        ip: u32,
        ports: Vec<u16>,
        features: Vec<Feature>,
        asn: u32,
    ) -> HostObservation {
        HostObservation {
            ip,
            open_ports: ports,
            features,
            asn,
        }
    }

    #[test]
    fn test_predictions_to_commands() {
        let preds = vec![TargetPrediction {
            ip: 0xC0A80001,
            asn: 16509,
            predicted_ports: vec![(443, 0.9), (80, 0.8)],
            source_features: vec![Feature::PortHttp, Feature::AppTls],
            prediction_method: PredictionMethod::RemainingService,
        }];

        let cmds = predictions_to_commands(
            &preds,
            aegis_proto::TargetSource::TargetSourceGpsCoOccurrence,
        );
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].target_ip, vec![192, 168, 0, 1]);
        assert_eq!(cmds[0].port, 443);
        assert!(cmds[0].priority > 0.0);
    }
}
