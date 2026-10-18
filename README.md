# AEGIS — Intelligence-Driven Internet Scanner

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-730%2B_passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **AEGIS** (Greek: Αἰγίς) — The shield of Zeus and Athena. An intelligence-driven, continuous internet-wide scanning platform that replaces brute-force enumeration with predictive targeting, streaming analytics, and deep protocol interrogation.

---

## 🎯 What Is AEGIS?

AEGIS is a from-scratch, production-grade internet mapping platform designed to operate at internet scale. Unlike legacy scanners (ZMap, Masscan, Nmap) that rely on brute-force port sweeps, AEGIS uses:

- **Predictive Targeting** — GPS co-occurrence models (IPv4) + 6Hit/6SENSE reinforcement learning (IPv6) to achieve 92.5% service discovery with 131× less bandwidth
- **Two-Phase Probing** — Stateless L4 SYN/UDP discovery → Stateful L7 protocol interrogation (TLS/JA4+, HTTP/2, gRPC, QUIC, SSH, MQTT, LLM endpoints)
- **Streaming Architecture** — Kafka → Flink → Iceberg on S3/MinIO → Trino SQL with time-travel queries
- **Deep Protocol Intelligence** — JA4+ TLS fingerprinting, LZR payload-based service reclassification, SSH behavioral analysis, LLM refusal-vector fingerprinting
- **Ethics by Design** — Identity transparency, opt-out enforcement, adaptive rate limiting embedded at the socket layer

---

## 📊 Capability Summary

| Capability | Status | Details |
|------------|--------|---------|
| **IPv4 Targeting** | ✅ | GPS co-occurrence (25 features, 200+ signatures) |
| **IPv6 Targeting** | ✅ | 6Hit/6SENSE RL agent + VistaScan visibility-aware scheduling |
| **L4 Scanning** | ✅ | AF_XDP zero-copy, SipHash-2-4 seq encoding, 16-shard |
| **L7 Interrogation** | ✅ | TLS/JA4+, HTTP/2+gRPC, QUIC, SSH, MQTT, LLM endpoints |
| **Service Reclassification** | ✅ | LZR Aho-Corasick + regex (445+ signatures, <3 µs) |
| **Streaming Ingestion** | ✅ | Kafka → Flink (1-min dedup, exactly-once) → Iceberg |
| **Storage** | ✅ | Iceberg on MinIO/S3, Nessie catalog, Trino SQL |
| **Enrichment** | ✅ | Geo (RPKI→RTT→WHOIS), CVE (NVD), Asset graph (TLS/SSH/JARM) |
| **API Gateway** | ✅ | Axum REST/WS/gRPC, JWT/RBAC, rate limiting, audit log |
| **Dashboard** | ✅ | React 19 + Vite, TanStack Query, Recharts |
| **ML Targeting** | ✅ | PyTorch DQN + Ray, FastAPI serving, Triton export |
| **Testing** | ✅ | 730+ unit/integration tests, cargo-fuzz, Kani proofs |

---

## 🏗 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AEGIS CONTINUOUS SCANNING PIPELINE                     │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────┐   ┌──────────────────┐   ┌─────────────────────────────────┐
│  PASSIVE     │   │  TARGETING       │   │  SCANNING PoPs (×3+ regions)     │
│  INTEL       │──▶│  ENGINE          │──▶│  ┌─────────┐ ┌─────────┐         │
│  CT/DNS/BGP  │   │  GPS (IPv4)      │   │  │ L4 SYN  │ │ L7 INT  │         │
│  Seeds       │   │  6Hit RL (IPv6)  │   │  │ AF_XDP  │ │ TLS/gRPC│         │
└──────────────┘   │  VistaScan       │   │  │ JA4+    │ │ SSH/QUIC│         │
                   └────────┬─────────┘   │  └────┬────┘ └────┬────┘         │
                            │             └───────┼─────────┼──────────────┘
                            ▼                     ▼         ▼
                   ┌──────────────────┐   ┌─────────────────────────────────┐
                   │  KAFKA TOPICS    │   │  STREAM PROCESSING (FLINK)      │
                   │  raw-l4-scans    │   │  • 1-min sliding dedup          │
                   │  tls-fingerprints│   │  • LZR reclassification         │
                   │  l7-banners      │   │  • CT/DNS enrichment            │
                   │  scan-commands   │   │  • Exactly-once checkpoints     │
                   └────────┬─────────┘   └─────────────┬──────────────────┘
                            │                           │
                            ▼                           ▼
                   ┌─────────────────────────────────────────────────────────┐
                   │              ICEBERG ON MINIO/S3 (ACID, SNAPSHOTS)       │
                   │  Tables: hosts | services | certificates                 │
                   │  Partitions: days(first_seen) | Hash distribution        │
                   └────────────────────────────┬────────────────────────────┘
                                                │
                                                ▼
                   ┌─────────────────────────────────────────────────────────┐
                   │              ENRICHMENT PIPELINES (ASYNC)                │
                   │  • Geo: RPKI geofeed → RTT triangulation → WHOIS        │
                   │  • CVE: CPE extraction → NVD cache → CVSS scoring       │
                   │  • Asset Graph: TLS serials, SSH keys, JARM, BGP AS     │
                   └────────────────────────────┬────────────────────────────┘
                                                │
                                                ▼
                   ┌─────────────────────────────────────────────────────────┐
                   │              SERVING LAYER                               │
                   │  • Trino: time-travel SQL, point lookups <200ms         │
                   │  • API Gateway: REST/WS/gRPC, JWT+RBAC, rate limiting   │
                   │  • Dashboard: React 19, TanStack Query, Recharts        │
                   └─────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.98+ (stable)
- Node.js 26+ / npm 11+
- Python 3.14+ (for ML)
- Docker + Kind (for local cluster)
- protoc 36+ (`scoop install protobuf` or `apt install protobuf-compiler`)

### Local Development Cluster
```bash
cd infra
./scripts/bootstrap.sh          # Spins up Kind + Kafka + Flink + MinIO + Trino + Nessie
./scripts/verify.sh             # Validates all services healthy
```

### Generate Synthetic Data
```bash
cd infra/synthetic-generator
cargo run --release -- \
  --kafka-broker localhost:9092 \
  --topics raw-l4-scans,tls-fingerprints,l7-banners \
  --rate 1000 \
  --duration 60
```

### Query Data
```bash
# Trino CLI
trino --server localhost:8080 --catalog aegis --schema aegis
# SQL
SELECT * FROM hosts FOR SYSTEM_TIME AS OF TIMESTAMP '2026-08-30 12:00:00' LIMIT 10;
```

### Dashboard
```bash
cd dashboard
npm install
npm run dev
# Open http://localhost:5173
```

---

## 📁 Project Structure

```
AEGIS/
├── AEGIS_Master_Document.md      # Master architecture (1269 lines)
├── README.md                     # This file
├── AGENTS.md                     # Build/test/lint commands for agents
├── .gitignore
├── .cargo/config.toml            # Cargo config (protoc, lld linker)
├── .github/workflows/ci.yml      # GitHub Actions CI
│
├── packet-engine/                # Core Rust workspace (12 crates)
│   ├── Cargo.toml
│   ├── af_xdp/                   # AF_XDP zero-copy packet engine
│   ├── sequence/                 # SipHash-2-4 seq encoding + SIMD
│   ├── traversal/                # Cyclic multiplicative permutation
│   ├── lzr/                      # LZR Aho-Corasick + regex classifier
│   ├── af_xdp/                   # Linux AF_XDP zero-copy engine
│   ├── interrogation/            # L7 interrogators
│   │   ├── tls/                  # TLS/JA4+ fingerprinting
│   │   ├── grpc/                 # HTTP/2 + gRPC + reflection
│   │   ├── quic/                 # QUIC initial + transport params
│   │   ├── behavioral/           # SSH, LLM, WF, bot-score
│   │   └── emerging/             # MCP, A2A stubs
│   ├── targeting/                # Targeting
│   │   ├── ipv4/                 # GPS co-occurrence
│   │   └── passive/              # CT, DNS, BGP ingestion
│   └── proto/                    # Canonical protobuf types
│
├── enrichment/                   # Post-scan enrichment
│   ├── geolocation/              # RPKI→RTT→WHOIS tiers
│   ├── cve/                      # NVD CVE correlation
│   ├── asset/                    # TLS/SSH/JARM asset graph
│   └── enrichment-common/        # Shared cache/freshness/audit
│
├── gateway/                      # API Gateway (Axum)
│   ├── src/
│   │   ├── auth/                 # JWT + Claims + RBAC
│   │   ├── audit_log/            # Signed audit trail
│   │   ├── grpc_handler/         # Trino gRPC proxy
│   │   ├── middleware/           # AuthZ, tracing, rate limit
│   │   ├── tenant/               # Multi-tenant + quotas
│   │   └── ws/                   # WebSocket streaming
│   ├── proto/gateway.proto
│   └── openapi.yaml
│
├── targeting/                    # Targeting + ML
│   ├── ipv6/                     # IPv6 targeting (Rust)
│   ├── ml/                       # Python ML (DQN, Ray, FastAPI)
│   └── vistascan/                # VistaScan visibility scheduler
│
├── infra/                        # Infrastructure as Code
│   ├── charts/                   # Helm charts (Kafka, Flink, MinIO, Trino, etc.)
│   ├── flink-job/                # Java Flink: Kafka → Iceberg
│   ├── kind/                     # Kind cluster config
│   ├── scripts/                  # bootstrap/verify/teardown
│   ├── synthetic-generator/      # Rust synthetic event generator
│   └── .github/workflows/ci.yml  # (legacy location)
│
├── dashboard/                    # React 19 + Vite + TanStack Query
│   ├── src/
│   └── package.json
│
├── sdks/                         # Client SDKs
│   ├── go/
│   ├── python/
│   └── typescript/
│
└── targeting/                    # Legacy? (see packet-engine/targeting)
```

---

## 🔧 Development Commands

All commands from repo root unless noted.

```bash
# Build everything
cargo build --workspace --all-targets     # packet-engine, enrichment
cargo build --workspace                   # gateway (separate)
cargo build --workspace                   # targeting/ipv6, vistascan
cargo build --release                     # infra/synthetic-generator

# Lint (all workspaces)
cargo clippy --workspace --all-targets -- -D warnings

# Test everything
cargo test --workspace                    # packet-engine, enrichment
cargo test                                # gateway, ipv6, vistascan
npm run lint                              # dashboard (oxlint)

# Format
cargo fmt --check                         # all workspaces

# Python ML
cd targeting/ml && pip install -r requirements.txt
ruff check . && ruff format . && pytest

# Dashboard
cd dashboard && npm install && npm run build

# Fuzzing (packet-engine)
cd packet-engine/fuzz && cargo fuzz run fuzz_ja4 -- -max_total_time=60

# Kani proofs
cd packet-engine/proofs && cargo kani
```

---

## 📚 Documentation

- [Architecture](docs/architecture.md) — System design, data flow, component details
- [API Reference](docs/api.md) — REST, WebSocket, gRPC, OpenAPI
- [Deployment](docs/deployment.md) — Kind, Helm, k8s, secrets, scaling
- [Development](docs/development.md) — Build, test, lint, debug, fuzz, Kani
- [API Gateway OpenAPI Spec](gateway/openapi.yaml)

---

## 🛡 Ethics & Safety

AEGIS embeds safety at the **socket layer** — cannot be bypassed by upper logic:

| Control | Implementation |
|---------|----------------|
| **Identity** | `User-Agent: AEGIS-Scanner/1.0 (+https://aegis.example.com/robots; contact@aegis.example.com)` on all HTTP/TLS |
| **Opt-Out** | CIDR blocklist loaded at prober startup, enforced in socket abstraction |
| **Rate Limiting** | Token-bucket per /24, per ASN, global PPS; exponential backoff on complaints |
| **Scope** | Lab-only until legal clearance; no public scanning without authorization |
| **Audit** | Signed append-only log of all scanning activity |

**Legal**: CFAA (US), IT Act (India), Computer Misuse Act (UK). Cloud provider ToS prohibit unauthorized scanning — use dedicated infrastructure. Consider Censys/Shodan APIs as alternatives.

---

## 🤝 Contributing

```bash
# 1. Fork & clone
# 2. Create feature branch
# 3. Make changes with tests
# 4. Run full CI locally
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --check
npm run lint   # dashboard
ruff check .   # targeting/ml

# 5. Submit PR with clear description
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE).

---

## 🙏 Acknowledgments

Built on foundational research:
- **ZMap** (Durumeric et al., USENIX Security 2013) — stateless SYN scanning
- **XMap** (Li et al., ACSAC 2025) — IPv6-capable successor
- **GPS** (Izhikevich et al., SIGCOMM 2022) — predictive co-occurrence
- **LZR** (Izhikevich et al., USENIX Security 2021) — protocol reclassification
- **6Hit** (Hou et al., INFOCOM 2021) — RL IPv6 targeting
- **Censys** (Durumeric et al., SIGCOMM 2025) — production CQRS architecture
- **JA4+** (FoxIO) — modern TLS fingerprinting
- **VistaScan** (Hu et al., Sensors 2026) — visibility-aware scheduling