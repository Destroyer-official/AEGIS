use clap::Parser;
use tracing::info;

mod generator;
mod kafka_producer;

#[derive(Parser)]
#[command(name = "aegis-synthetic-generator")]
#[command(about = "Generates synthetic AEGIS scan events using canonical proto types")]
struct Cli {
    #[arg(long, default_value = "localhost:9092")]
    kafka_broker: String,

    /// Comma-separated list of topics to produce to. One of:
    /// raw-l4-scans, tls-fingerprints, l7-banners.
    #[arg(long, default_value = "raw-l4-scans", value_delimiter = ',')]
    topics: Vec<String>,

    #[arg(long, default_value = "100")]
    rate: u64,

    #[arg(long, default_value = "300")]
    duration: u64,

    #[arg(long, default_value = "aegis-poc")]
    source_pop: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    info!("AEGIS Synthetic Generator (using canonical proto types)");
    info!("  Kafka broker: {}", cli.kafka_broker);
    info!("  Topics: {}", cli.topics.join(", "));
    info!("  Rate: {} events/sec", cli.rate);
    info!("  Duration: {} seconds", cli.duration);
    info!("  Source PoP: {}", cli.source_pop);

    let producer = kafka_producer::create_producer(&cli.kafka_broker).await?;
    let mut gen = generator::Generator::new(cli.source_pop.clone());

    let total_events = cli.rate * cli.duration;
    let mut sent = 0u64;

    info!("Generating {} total events", total_events);

    let start = std::time::Instant::now();

    while sent < total_events {
        let batch_size = std::cmp::min(cli.rate, total_events - sent);

        for _ in 0..batch_size {
            for topic in &cli.topics {
                match topic.as_str() {
                    "raw-l4-scans" => {
                        let event = gen.generate_l4_response();
                        kafka_producer::produce_l4_response(&producer, topic, &event).await?;
                    }
                    "tls-fingerprints" => {
                        let event = gen.generate_tls_fingerprint();
                        kafka_producer::produce_tls_fingerprint(&producer, topic, &event).await?;
                    }
                    "l7-banners" => {
                        let event = gen.generate_l7_banner();
                        kafka_producer::produce_l7_banner(&producer, topic, &event).await?;
                    }
                    other => {
                        return Err(format!("unknown topic: {}", other).into());
                    }
                }
            }
        }

        sent += batch_size;
        let elapsed = start.elapsed().as_secs_f64();
        let actual_rate = sent as f64 / elapsed;

        if sent.is_multiple_of(1000) || sent == total_events {
            info!(
                "Progress: {}/{} ({:.1} events/sec)",
                sent, total_events, actual_rate
            );
        }

        let expected_elapsed = sent as f64 / cli.rate as f64;
        let sleep_time = (expected_elapsed - elapsed).max(0.0);
        if sleep_time > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(sleep_time)).await;
        }
    }

    let elapsed = start.elapsed();
    info!(
        "Generation complete: {} events in {:.1}s ({:.1} events/sec)",
        total_events,
        elapsed.as_secs_f64(),
        total_events as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
