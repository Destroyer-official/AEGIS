use aegis_proto::*;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tracing::info;

pub type KafkaProducer = FutureProducer;

pub async fn create_producer(brokers: &str) -> Result<KafkaProducer, Box<dyn std::error::Error>> {
    let producer: KafkaProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "30000")
        .set("acks", "all")
        .set("linger.ms", "5")
        .set("batch.size", "65536")
        .create()?;

    info!("Kafka producer created for brokers: {}", brokers);
    Ok(producer)
}

pub async fn produce_l4_response(
    producer: &KafkaProducer,
    topic: &str,
    event: &L4Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("{:?}", event.target_ip);
    let mut payload = Vec::new();
    event.encode(&mut payload)?;

    let record = FutureRecord::to(topic)
        .key(key.as_bytes())
        .payload(&payload);

    match producer.send(record, Duration::from_secs(10)).await {
        Ok(_) => Ok(()),
        Err((err, _)) => {
            tracing::error!("Failed to produce L4Response: {}", err);
            Err(err.into())
        }
    }
}

pub async fn produce_tls_fingerprint(
    producer: &KafkaProducer,
    topic: &str,
    event: &TLSFingerprint,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("{:?}", event.target_ip);
    let mut payload = Vec::new();
    event.encode(&mut payload)?;

    let record = FutureRecord::to(topic)
        .key(key.as_bytes())
        .payload(&payload);

    match producer.send(record, Duration::from_secs(10)).await {
        Ok(_) => Ok(()),
        Err((err, _)) => {
            tracing::error!("Failed to produce TLSFingerprint: {}", err);
            Err(err.into())
        }
    }
}

pub async fn produce_l7_banner(
    producer: &KafkaProducer,
    topic: &str,
    event: &L7Banner,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("{:?}", event.target_ip);
    let mut payload = Vec::new();
    event.encode(&mut payload)?;

    let record = FutureRecord::to(topic)
        .key(key.as_bytes())
        .payload(&payload);

    match producer.send(record, Duration::from_secs(10)).await {
        Ok(_) => Ok(()),
        Err((err, _)) => {
            tracing::error!("Failed to produce L7Banner: {}", err);
            Err(err.into())
        }
    }
}
