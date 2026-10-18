# AEGIS Infrastructure

Running data spine for the AEGIS intelligence-driven internet scanner platform.

## Quick Start

```bash
# One-command cluster bootstrap (kind + all components)
./scripts/bootstrap.sh

# Verify all services are running
./scripts/verify.sh

# Tear down when done
./scripts/teardown.sh
```

## Cluster URLs

| Service    | URL                          | Purpose                                |
|------------|------------------------------|----------------------------------------|
| Kafka      | `localhost:9092`             | Streaming data backbone                |
| Flink UI   | `http://localhost:8081`       | Stream processing dashboard            |
| Trino      | `http://localhost:8080`       | SQL query engine (Iceberg)             |
| MinIO API  | `http://localhost:9000`       | S3-compatible object storage           |
| MinIO UI   | `http://localhost:9001`       | MinIO web console                      |
| Nessie     | `http://localhost:19120`      | Iceberg REST catalog + version store   |

## Kafka Topics (§3.3 Topic Architecture)

| Topic                | Partitions | Retention | Description                                |
|----------------------|------------|-----------|--------------------------------------------|
| `raw-l4-scans`       | 3          | 24h       | Raw L4 discovery responses (IP, port, proto)|
| `tls-fingerprints`   | 3          | 24h       | JA4/JA4+ fingerprints and TLS handshake    |
| `l7-banners`         | 3          | 24h       | Protocol-specific banner data              |
| `scan-commands`      | 3          | 24h       | Target generation commands                 |
| `enrichment-events`  | 3          | 24h       | Downstream enrichment pipeline events      |

## Schema Registry

| Property             | Value                                        |
|----------------------|----------------------------------------------|
| Catalog Type         | Nessie (Iceberg catalog, `iceberg.catalog.type=nessie`, API v1) |
| Nessie API           | `http://localhost:19120/api/v1` (Trino 433) / `api/v2` (new clients) |
| Default Branch       | `main`                                       |
| Namespace            | `aegis`                                      |
| Tables               | `hosts`, `services`, `certificates`          |
| Table Format         | Apache Iceberg v2                            |
| File Format          | Parquet                                      |
| Partitioning         | `days(first_seen)`                           |
| Distribution Mode    | Hash                                         |

### Iceberg Table Schemas

**aegis.hosts** (§7.1 Host Record)
```
ip_address (string), host_id (string), asn (long), organization (string),
bgp_prefix (string), bgp_as_path (string), reverse_dns (string),
forward_dns (string), geo_tier_used (string), geo_latitude (double),
geo_longitude (double), geo_city (string), geo_country (string),
geo_confidence (float), first_seen (long), last_seen (long),
scan_count (int), status (string)
```

**aegis.services** (§7.2 Service Record)
```
host_ip (string), transport_protocol (string), port_number (long),
reclassified_protocol (string), detection_method (string),
raw_banner (string), sanitized_banner (string), banner_hash (string),
ja4_fingerprint (string), ja4s_fingerprint (string), tls_version (string),
cipher_suite (string), certificate_chain (string), http_method (string),
http_status (long), ssh_version (string), ssh_kex_algorithms (string),
cves (array<string>), cvss_scores (array<float>), first_seen (long),
last_seen (long), status (string)
```

**aegis.certificates** (§7.3 Certificate Record)
```
sha256_fingerprint (string), serial_number (string), cert_pem (string),
subject (string), issuer (string), subject_cn (string), issuer_cn (string),
organization (string), sans (array<string>), not_before (long),
not_after (long), is_expired (boolean), certificate_chain (array<string>),
rpki_validation_status (string), roa_prefix (string),
first_seen (long), last_seen (long)
```

## Data Flow

```
Scanning Agents (Phase 1 L4 + Phase 2 L7)
         |
         v
    Kafka (raw-l4-scans, tls-fingerprints, l7-banners)
         |
         v
    Flink (1-min dedup window, LZR reclassification, exactly-once checkpointing)
         |
         v
    Iceberg Tables on MinIO (ACID, snapshot isolation, time-travel)
         |
         v
    Trino (interactive SQL, point-in-time queries)
         |
         v
    API Gateway / Web Dashboard
```

## Synthetic Generator

Rust binary that generates synthetic scan events for development/testing.

```bash
cd synthetic-generator

# Build
cargo build --release

# Run (default: 100 events/sec for 5 minutes)
cargo run --release -- \
    --kafka-broker localhost:9092 \
    --topic raw-l4-scans \
    --rate 100 \
    --duration 300 \
    --source-pop aegis-poc

# Verify events in Kafka
kubectl exec aegis-kafka-0 -- /opt/kafka/bin/kafka-console-consumer.sh \
    --bootstrap-server localhost:9092 \
    --topic raw-l4-scans \
    --from-beginning \
    --max-messages 5
```

### Event Schema (§7 Protobuf)

The generator produces `L4Response` protobuf messages from the canonical `aegis_types.proto` (§7 Protobuf):
- `L4Response`: IP, port, transport protocol, detection method, sanitized banner, banner hash, JA4 fingerprint

## Flink Job

Java Flink job: Kafka → dedup (1-min window) → Iceberg Dynamic Sink.

```bash
cd flink-job

# Build the fat JAR
./gradlew shadowJar

# Submit to Flink cluster
kubectl cp build/libs/aegis-flink-job-0.1.0.jar \
    aegis-flink-jobmanager:/tmp/aegis-flink-job.jar

kubectl exec aegis-flink-jobmanager -- \
    flink run -d /tmp/aegis-flink-job.jar
```

## Trino Time-Travel Queries

```sql
-- Connect to Trino
trino --server localhost:8080 --catalog aegis --schema aegis

-- Query current state
SELECT * FROM hosts LIMIT 10;

-- Time-travel query (§3.4, §7.4)
SELECT * FROM hosts
FOR SYSTEM_TIME AS OF TIMESTAMP '2026-08-30 12:00:00';

-- Services with JA4 fingerprints
SELECT host_ip, port_number, reclassified_protocol,
       ja4_fingerprint, status
FROM services
WHERE status = 'active'
LIMIT 20;

-- Certificate analysis
SELECT sha256_fingerprint, subject_cn, issuer_cn,
       not_before, not_after, is_expired
FROM certificates
WHERE is_expired = false
LIMIT 20;
```

## Architecture Reference

This infrastructure implements the data spine (Phase 1) per the AEGIS Master Document:

- **§3.3 Streaming Data Backbone**: Kafka with 5 topics, Flink stream processing
- **§3.4 Storage & Query Layer**: Iceberg on MinIO with Nessie catalog, Trino queries
- **§3.8 Orchestration**: Kubernetes via kind, Helm charts for each component
- **§7 Data Model**: Host, Service, Certificate records with additive schema evolution
- **§9 Performance**: 1-min Flink checkpointing, exactly-once semantics

## Directory Structure

```
infra/
├── README.md                              # This file
├── .github/workflows/ci.yml               # CI: cargo, clippy, cargo-audit, cargo-fuzz, k6
├── charts/
│   ├── aegis-kafka/                       # Kafka KRaft (no ZooKeeper)
│   ├── aegis-flink/                       # Flink JobManager + TaskManagers
│   ├── aegis-minio/                       # MinIO S3 storage
│   ├── aegis-polaris/                     # Nessie catalog (Iceberg REST backend)
│   ├── aegis-trino/                       # Trino coordinator + workers
│   └── aegis-iceberg/                     # Iceberg table init job
├── kind/
│   └── kind-config.yaml                   # kind cluster with port mappings
├── scripts/
│   ├── bootstrap.sh                       # One-command cluster setup
│   ├── create-topics.sh                   # Create the 5 Kafka topics
│   ├── verify.ps1                         # Full infrastructure verification (PowerShell)
│   ├── teardown.sh                        # Cluster cleanup
│   └── k6-trino-health.js                 # k6 load test for Trino
├── synthetic-generator/                   # Rust: synthetic scan events
│   ├── Cargo.toml
│   ├── build.rs
│   ├── proto/scan_events.proto            # §7 data model as protobuf
│   └── src/
│       ├── main.rs
│       ├── generator.rs
│       └── kafka_producer.rs
└── flink-job/                             # Java: Kafka → dedup → Iceberg
    ├── build.gradle.kts
    ├── settings.gradle.kts
    └── src/main/java/com/aegis/flink/
        └── DeduplicationJob.java
```

## CI/CD Pipeline

GitHub Actions workflow runs on every push/PR:

| Job               | What it does                                          |
|-------------------|-------------------------------------------------------|
| cargo-build       | Build both workspaces (packet-engine, enrichment)     |
| build-standalone  | Build standalone crates (gateway, synthetic-generator) |
| clippy            | Lint all Rust code with `-D warnings`                 |
| cargo-audit       | Security audit all Cargo.lock files                   |
| cargo-fuzz        | Fuzz test JA4 and TLS parsers                         |
| k6-load-test      | Load test Trino endpoint (main branch only)           |
| fmt               | Check rustfmt formatting                              |
