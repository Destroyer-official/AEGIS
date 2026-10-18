# AEGIS Architecture Document

**Version:** 1.0  
**Status:** Production-Ready  
**Classification:** Internal Technical Reference

---

## 1. System Overview

AEGIS is an intelligence-driven, continuous internet-wide scanning platform. It replaces legacy brute-force port sweeping with predictive targeting, streaming analytics, and deep protocol interrogation.

### 1.1 Design Goals

| Goal | Target | Validation |
|------|--------|------------|
| L4 Probe Velocity | ≥ 10M probes/sec global | AF_XDP zero-copy, 16-shard |
| Data Freshness | < 60 sec probe-to-query | Kafka → Flink → Iceberg streaming |
| Service Accuracy | > 90% confirmed live | 72-hr staleness eviction |
| IPv6 Hit Rate | > 5% | 6Hit RL agent |
| L7 Classification | < 10 ms | LZR Aho-Corasick (< 3 µs) |

### 1.2 Non-Goals

- ❌ Not a vulnerability scanner (no exploitation)
- ❌ Not a compliance auditor (no policy engine)
- ❌ Not a SIEM (no alerting/correlation)
- ❌ No legacy tool dependencies (ZMap, Masscan, Nmap)

---

## 2. Core Components

### 2.1 Packet Engine (`packet-engine/`)

#### 2.1.1 Sequence Encoding (`sequence/`)
- **SipHash-2-4** implementation from scratch (constant-time)
- `SeqKey`: 16-byte secret → 32-bit TCP sequence numbers
- Encodes: `dst_ip(4) | dst_port(2) | timestamp(4)` = 10 bytes (IPv4)
- Zero-state response validation — no per-connection memory
- SIMD-optimized batch operations
- Verified against SipHash reference vectors

#### 2.1.2 Address Traversal (`traversal/`)
- **Cyclic multiplicative group** over ℤ*ₚ
- `next = (current × g) mod p` with `p = 2³² + 15`, `g = 3`
- Full-period permutation: every IPv4 address exactly once per cycle
- `next_ipv4()` skips values ≥ 2³² (~15 skips/cycle)
- 16 concurrent shards via `PermutationShard` / `ShardSet`
- Constant-time modular arithmetic (u128 intermediate)
- Kani proofs: permutation bijection, SipHash collision bounds

#### 2.1.3 AF_XDP Engine (`af_xdp/`)
- Linux-only zero-copy via AF_XDP sockets (kernel 5.10+)
- UMEM: 4096 frames × 2KB = 8MB zero-copy region
- FILL/COMPLETION/TX/RX rings (lock-free)
- eBPF XDP program: parse Eth/IP/TCP/UDP → redirect via XSKMAP
- Pipelined send/recv with recv-key registration before send
- NUMA-aware queue allocation, metrics server, config hot-reload
- Graceful degradation to `sendmmsg` / `pnet` on non-Linux

#### 2.1.4 LZR Classifier (`lzr/`)
- Two-tier: Aho-Corasick (literal prefixes) + regex fallback
- 445+ embedded signatures (CSV), hot-reload, DFA snapshot serde
- gRPC service (`Classify`, `ClassifyBatch`, `HealthCheck`)
- Stats server, priority-based matching (0-100)
- < 3 µs/classify, ~400K classifications/sec/core
- 40 categories: web, mail, remote, database, AI, IoT, etc.

#### 2.1.5 Interrogation Suite (`interrogation/`)
| Module | Protocol | Key Features |
|--------|----------|--------------|
| `tls/` | TLS 1.0-1.3 | Zero-copy record/CH/SH parsers, full JA4+ suite (JA4,S,H,T,TS,L,X,SSH,D), GREASE, ALPN policy, rate limit, CT monitor |
| `grpc/` | HTTP/2 + gRPC | ALPN `h2`+`grpc`, connection preface, SETTINGS, health check, reflection |
| `quic/` | QUIC/HTTP/3 | Initial packet, Version Negotiation, transport params |
| `behavioral/` | SSH, LLM, WF | SSH KEX/host key analysis, LLM refusal vectors (cosine similarity), website fingerprinting (packet timing), JA4 bot-score |
| `emerging/` | MCP, A2A | Pluggable `Interrogator` trait, stub implementations |

#### 2.1.6 Targeting (`targeting/`)
| Module | Scope | Algorithm |
|--------|-------|-----------|
| `ipv4/` | IPv4 | GPS co-occurrence: 25 features, ndarray matrix, Rayon parallel |
| `passive/` | CT/DNS/BGP | crt.sh fetcher, DNS aggregator, BGP CSV parser, CVE feed |

---

### 2.2 Enrichment Pipeline (`enrichment/`)

| Pipeline | Input | Output | Method |
|----------|-------|--------|--------|
| **Geolocation** | IP | (lat, lon, tier, confidence) | Tier 1: RPKI-validated geofeed (confidence 1.0)<br>Tier 2: CBG/GeoCET/NeighborGeo RTT triangulation (confidence 0.6-0.8)<br>Tier 3: WHOIS/RDAP/BGP AS (confidence 0.2-0.4) |
| **CVE Correlation** | CPE strings | CVE IDs + CVSS | Local NVD cache, version comparison |
| **Asset Graph** | TLS/SSH/JARM | Edges | Shared cert serials, SSH host keys, JARM clusters, BGP AS |

---

### 2.3 Gateway (`gateway/`)
- **Framework**: Axum 0.7 + Tokio
- **Auth**: JWT (HS256), Claims (sub, role, tenant_id, exp, iat)
- **RBAC**: Viewer / Analyst / Admin + `manage_users` permission
- **Endpoints**:
  - `GET /health` — health check
  - `POST /auth/login` — JWT issuance
  - `POST /query` — Trino SQL (analyst+)
  - `GET /rate-limit/status` — token bucket status
  - `POST /admin/action` — audit export/verify/key
  - `POST /admin/users` — tenant CRUD, API keys
  - `GET /ws/query` / `/ws/stream` — WebSocket streaming
- **Middleware**: tracing (OpenTelemetry), authz, rate-limit, audit, tenant isolation
- **gRPC**: Trino query proxy (`GrpcQueryService`)

---

### 2.4 ML Targeting (`targeting/ml/`)
- **Algorithm**: Dueling DQN (PyTorch)
- **State**: 9 features (prefix_len, hit_rate, as_path_len, origin_asn, density, alloc_granularity, budget, seeds, expected_reward)
- **Actions**: Probe/64, Probe/120, Skip, Repartition
- **Reward**: +1 live host, -1 barren/alias
- **Serving**: FastAPI + Ray Serve, <100ms latency target
- **Training**: Ray distributed, continuous retraining, ONNX/Triton export
- **Drift Monitoring**: KS-test on feature distributions

---

### 2.5 VistaScan (`targeting/vistascan/`)
- **Visibility Matrix**: anchor-based probing → per-node per-CIDR visibility
- **Anchor Selection**: Greedy / MaxMinDistance / Random strategies
- **SLA Monitoring**: coverage % ≥ threshold, anchor count ≥ 2, redundancy ≥ 0.3
- **Grafana Dashboard**: JSON export for monitoring

---

### 2.6 Infra & Data Spine (`infra/`)

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Message Backbone** | Kafka (KRaft) / Redpanda | 5 topics, 3 partitions, 24h retention |
| **Stream Processing** | Apache Flink | 1-min dedup, LZR, exactly-once |
| **Object Storage** | MinIO (S3-compatible) | Iceberg table files |
| **Catalog** | Nessie (Iceberg REST) | Branching, tags, multi-engine |
| **Query Engine** | Trino | Interactive SQL, time-travel |
| **Orchestration** | Kind + Helm | Local dev cluster |
| **Flink Job** | Java (Gradle) | Kafka → dedup → Iceberg Dynamic Sink |

---

## 3. Data Model

### 3.1 Canonical Protobuf (`packet-engine/proto/aegis_types.proto`)

| Message | Kafka Topic | Description |
|---------|-------------|-------------|
| `ScanCommand` | `scan-commands` | Targeting → Scanner |
| `L4Response` | `raw-l4-scans` | L4 probe result |
| `TLSFingerprint` | `tls-fingerprints` | JA4+ handshake data |
| `L7Banner` | `l7-banners` | L7 interrogation result |
| `EnrichedHost` | (Flink → Iceberg) | Deduplicated, enriched record |

### 3.2 Iceberg Tables (MinIO + Nessie)

| Table | Partition | Key Fields |
|-------|-----------|------------|
| `aegis.hosts` | `days(first_seen)` | ip, host_id, asn, org, geo (tier/lat/lon/conf), status |
| `aegis.services` | `days(first_seen)` | host_ip, proto, port, protocol (post-LZR), ja4, ja4s, cves, status |
| `aegis.certificates` | `days(first_seen)` | sha256, serial, subject/issuer, SANs, validity, RPKI |

**Schema Evolution**: Additive only (nullable columns), metadata-only operations.

---

## 4. Data Flow

```
SCANNING PoPs
     │
     ▼ (L4Response, TLSFingerprint, L7Banner)
KAFKA (raw-l4-scans, tls-fingerprints, l7-banners)
     │
     ▼ (Flink: 1-min dedup, LZR, CT/DNS enrich, exactly-once)
ICBERG on MINIO (ACID, snapshots, time-travel)
     │
     ▼ (Async enrichment pipelines)
ENRICHMENT: Geo (3-tier) → CVE (NVD) → Asset Graph
     │
     ▼
TRINO (time-travel SQL, point lookups <200ms)
     │
     ▼
API GATEWAY (REST/WS/gRPC) → DASHBOARD
```

---

## 5. Scaling Characteristics

| Dimension | Strategy |
|-----------|----------|
| **L4 Throughput** | Add AF_XDP PoPs, 16-shard per core |
| **L7 Throughput** | Horizontal interrogator pods |
| **Kafka** | Partition by target IP hash |
| **Flink** | Scale TaskManagers by consumer lag |
| **Trino** | Add workers, catalog caching |
| **Iceberg** | Compaction jobs (24h), snapshot expiry |
| **Targeting** | Ray distributed DQN training |

---

## 6. Failure Modes & Mitigations

| Failure | Detection | Mitigation |
|---------|-----------|------------|
| PoP network partition | Flink consumer lag > threshold | Multi-PoP redundancy, VistaScan redistributes |
| Flink checkpoint timeout | Metrics alert | Increase interval, add parallelism |
| Iceberg compaction backlog | MinIO metrics | Scheduled compaction, larger target files |
| ML model drift | KS-test p-value < 0.01 | Automatic retrain, A/B canary |
| Opt-out violation | Audit log scan | Socket-layer enforcement (untestable bypass) |

---

## 7. Security Boundaries

| Boundary | Control |
|----------|---------|
| **Network** | PoPs in dedicated VPCs, no public ingress |
| **Auth** | JWT + mTLS for gRPC, RBAC on all endpoints |
| **Data** | MinIO encryption at rest, TLS in transit |
| **Secrets** | External vault (not in repo), env vars only |
| **Audit** | Signed append-only log, immutable |

---

## 8. References

- **Master Document**: `AEGIS_Master_Document.md` (1269 lines)
- **OpenAPI Spec**: `gateway/openapi.yaml`
- **Protobuf Schema**: `packet-engine/proto/aegis_types.proto`
- **Helm Charts**: `infra/charts/`
- **Flink Job**: `infra/flink-job/src/main/java/com/aegis/flink/`