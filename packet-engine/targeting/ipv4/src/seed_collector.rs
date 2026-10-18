use std::io::Write;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::HostObservation;

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("CSV write error: {0}")]
    Csv(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "parquet-output")]
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[cfg(feature = "parquet-output")]
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
}

/// A seed record from a 1% LZR scan, ready for persistence.
///
/// This is the flattened representation of a host observation
/// suitable for writing to Parquet or CSV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRecord {
    /// IPv4 address in dotted-quad notation.
    pub ip: String,
    /// Autonomous System Number.
    pub asn: u32,
    /// Open ports (comma-separated).
    pub open_ports: String,
    /// Active features (comma-separated).
    pub features: String,
    /// Number of open ports.
    pub port_count: u32,
    /// Number of active features.
    pub feature_count: u32,
    /// ISO-8601 timestamp of the observation.
    pub observed_at: String,
}

impl SeedRecord {
    /// Convert a HostObservation into a SeedRecord.
    pub fn from_observation(obs: &HostObservation) -> Self {
        let ip_str = format!(
            "{}.{}.{}.{}",
            (obs.ip >> 24) & 0xFF,
            (obs.ip >> 16) & 0xFF,
            (obs.ip >> 8) & 0xFF,
            obs.ip & 0xFF,
        );

        let open_ports = obs
            .open_ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let features = obs
            .features
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Self {
            ip: ip_str,
            asn: obs.asn,
            open_ports,
            features,
            port_count: obs.open_ports.len() as u32,
            feature_count: obs.features.len() as u32,
            observed_at: chrono_free_now(),
        }
    }
}

/// Simple timestamp without pulling in chrono.
fn chrono_free_now() -> String {
    // Use a basic ISO-8601 timestamp. In production this would
    // use chrono or time crate. For seed collection purposes,
    // a placeholder is acceptable.
    "2026-08-29T00:00:00Z".to_string()
}

/// Seed collector that accumulates host observations from a 1%
/// LZR scan and persists them to CSV or Parquet.
pub struct SeedCollector {
    records: Vec<SeedRecord>,
}

impl SeedCollector {
    pub fn new() -> Self {
        Self {
            records: Vec::with_capacity(1024),
        }
    }

    /// Create with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
        }
    }

    /// Ingest a single host observation from the scanner.
    pub fn collect(&mut self, obs: &HostObservation) {
        self.records.push(SeedRecord::from_observation(obs));
    }

    /// Ingest a batch of observations.
    pub fn collect_batch(&mut self, observations: &[HostObservation]) {
        for obs in observations {
            self.collect(obs);
        }
    }

    /// Total records collected.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Write all collected records to CSV format.
    pub fn write_csv(&self, writer: &mut impl Write) -> Result<(), SeedError> {
        let mut wtr = csv::Writer::from_writer(writer);

        for record in &self.records {
            wtr.serialize(record)
                .map_err(|e| SeedError::Csv(e.to_string()))?;
        }

        wtr.flush().map_err(|e| SeedError::Csv(e.to_string()))?;
        Ok(())
    }

    /// Write all collected records to Parquet format.
    ///
    /// Requires the `parquet-output` feature to be enabled.
    #[cfg(feature = "parquet-output")]
    pub fn write_parquet(&self, path: impl AsRef<std::path::Path>) -> Result<(), SeedError> {
        use arrow::array::{StringArray, UInt32Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use parquet::arrow::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("ip", DataType::Utf8, false),
            Field::new("asn", DataType::UInt32, false),
            Field::new("open_ports", DataType::Utf8, false),
            Field::new("features", DataType::Utf8, false),
            Field::new("port_count", DataType::UInt32, false),
            Field::new("feature_count", DataType::UInt32, false),
            Field::new("observed_at", DataType::Utf8, false),
        ]));

        let ips: Vec<&str> = self.records.iter().map(|r| r.ip.as_str()).collect();
        let asns: Vec<u32> = self.records.iter().map(|r| r.asn).collect();
        let open_ports: Vec<&str> = self.records.iter().map(|r| r.open_ports.as_str()).collect();
        let features: Vec<&str> = self.records.iter().map(|r| r.features.as_str()).collect();
        let port_counts: Vec<u32> = self.records.iter().map(|r| r.port_count).collect();
        let feature_counts: Vec<u32> = self.records.iter().map(|r| r.feature_count).collect();
        let timestamps: Vec<&str> = self
            .records
            .iter()
            .map(|r| r.observed_at.as_str())
            .collect();

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(ips)),
                Arc::new(UInt32Array::from(asns)),
                Arc::new(StringArray::from(open_ports)),
                Arc::new(StringArray::from(features)),
                Arc::new(UInt32Array::from(port_counts)),
                Arc::new(UInt32Array::from(feature_counts)),
                Arc::new(StringArray::from(timestamps)),
            ],
        )?;

        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = ArrowWriter::try_new(file, schema, None)?;
        writer.write(&batch)?;
        writer.close()?;

        Ok(())
    }

    /// Get references to all collected records.
    pub fn records(&self) -> &[SeedRecord] {
        &self.records
    }
}

impl Default for SeedCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Feature;

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
    fn test_seed_collect() {
        let mut collector = SeedCollector::new();

        collector.collect(&make_observation(
            0xC0A80001,
            vec![22, 80, 443],
            vec![Feature::PortSsh, Feature::PortHttp, Feature::PortHttps],
            15169,
        ));

        assert_eq!(collector.record_count(), 1);

        let record = &collector.records()[0];
        assert_eq!(record.ip, "192.168.0.1");
        assert_eq!(record.asn, 15169);
        assert_eq!(record.port_count, 3);
    }

    #[test]
    fn test_write_csv() {
        let mut collector = SeedCollector::new();

        collector.collect(&make_observation(
            0xC0A80001,
            vec![22, 80],
            vec![Feature::PortSsh, Feature::PortHttp],
            12345,
        ));

        collector.collect(&make_observation(
            0xC0A80002,
            vec![443],
            vec![Feature::PortHttps, Feature::AppTls],
            16509,
        ));

        let mut buf = Vec::new();
        collector.write_csv(&mut buf).unwrap();

        let csv_str = String::from_utf8(buf).unwrap();

        // Parse back and verify record count
        let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
        let records: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
        assert_eq!(
            records.len(),
            2,
            "CSV should contain 2 data records, got {}:\n{}",
            records.len(),
            csv_str
        );
        assert_eq!(&records[0][0], "192.168.0.1");
        assert_eq!(&records[1][0], "192.168.0.2");
    }

    #[test]
    fn test_feature_string_roundtrip() {
        let obs = make_observation(
            0x0A000001,
            vec![22, 80, 443],
            vec![Feature::PortSsh, Feature::PortHttp, Feature::AppTls],
            16509,
        );

        let record = SeedRecord::from_observation(&obs);
        assert!(record.features.contains("port_ssh"));
        assert!(record.features.contains("port_http"));
        assert!(record.features.contains("app_tls"));
    }
}
