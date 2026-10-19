# AEGIS — Intelligence-Driven Internet Scanner

[![Build Status](https://github.com/Destroyer-official/AEGIS/actions/workflows/ci.yml/badge.svg)](https://github.com/Destroyer-official/AEGIS/actions/workflows/ci.yml)
[![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-verified_passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.98%2B-orange.svg)]()
[![Node Version](https://img.shields.io/badge/node-26%2B-green.svg)]()
[![Python Version](https://img.shields.io/badge/python-3.14%2B-blue.svg)]()

> **AEGIS** (Greek: Αἰγίς) — The shield of Zeus and Athena. An intelligence-driven, continuous internet-wide scanning platform that replaces brute-force enumeration with predictive targeting, streaming analytics, deep protocol interrogation, and zero-trust cryptographic security.

---

## 🎯 What Is AEGIS?

AEGIS is an open-source, production-ready internet mapping platform designed to operate at internet scale. Unlike legacy scanners (ZMap, Masscan, Nmap) that rely on brute-force port sweeps, AEGIS uses:

- **Predictive Targeting** — GPS co-occurrence models (IPv4) + 6Hit/6SENSE reinforcement learning (IPv6) to achieve high service discovery with minimized bandwidth overhead.
- **Two-Phase Probing** — Stateless L4 SYN/UDP discovery → Stateful L7 protocol interrogation (TLS/JA4+, HTTP/2, gRPC, QUIC, SSH, MQTT, LLM endpoints).
- **Streaming Architecture** — Kafka → Flink → Iceberg on S3/MinIO → Trino SQL with time-travel queries.
- **Deep Protocol Intelligence** — JA4+ TLS fingerprinting, LZR payload-based service reclassification, SSH behavioral analysis, LLM refusal-vector fingerprinting.
- **Enterprise Security Invariants** — Argon2id password verification, tamper-evident Ed25519 hash-chained audit trails, lexical SQL injection prevention, CSPRNG API keys, and bounded non-blocking rate limiting.

---

## 🔒 Security Architecture & Cryptographic Invariants

AEGIS enforces strict zero-trust cryptographic boundaries across every communication path:

| Defense Vector | Implementation | Guarantees |
|---|---|---|
| **Authentication & RBAC** | `argon2 = "0.5"`, `jsonwebtoken = "9"` | Salted **Argon2id** password verification against server-side user stores. Client-controlled role derivations are strictly rejected. Roles (`Admin`, `Analyst`, `Viewer`) are cryptographically verified per request. |
| **Fail-Closed Secrets** | `gateway/src/auth/mod.rs` | In production environments, `JWT_SECRET` must be provisioned externally with $\ge 32$ characters. Startup terminates immediately if secrets are absent or weak. |
| **SQL Security Boundary** | `gateway/src/query/mod.rs` | Lexical grammar inspection strips quoted string literals to detect and reject statement chaining (unquoted semicolons), blocks destructive DDL/DML mutations (`DROP`, `ALTER`, `TRUNCATE`, `INSERT`, `UPDATE`, `DELETE`, etc.), and enforces read-only query verbs with hard result row caps. |
| **Tenant Isolation** | `gateway/src/tenant.rs`, `ws/`, `grpc_handler/` | Authenticated principal identities propagate across REST, WebSocket, and gRPC execution pipelines. Synthetic administrative identities on default tenants are eliminated. |
| **API-Key Lifecycle** | `gateway/src/tenant.rs` | Keys generated via 32-byte CSPRNG (`rand::rngs::OsRng`), formatted as `aegis_<tenant>_<entropy>`. Stored verifiers use SHA-256 (MD5 eliminated). Keys return once at creation; verifiers enforce tenant status, scopes, and expiration timestamps. |
| **Audit Log Hash Chain** | `gateway/src/audit_log.rs` | Every audit entry contains a strictly monotonic sequence number and a `prev_entry_hash` linkage. Payloads are canonicalized to JSON, hashed with SHA-256, and signed via Ed25519 (`ed25519-dalek`). Deletions, reordering, truncation, or field edits are detected by `verify_log()`. |
| **Non-Blocking Rate Limiting** | `gateway/src/rate_limit/mod.rs` | Limiter cache bounded to 10,000 entries with automatic LRU-style pruning upon saturation. Over-limit traffic immediately receives `429 Too Many Requests` without thread starvation. |
| **Model Provenance** | `targeting/ml/export_onnx.py` | Exporter strictly fails closed (`FileNotFoundError`) if checkpoints are absent, preventing untrained weights from being deployed to production inference pipelines. |

---

## ⚡ Quick Start: Hands-on Usage & Real Demos

You can run individual probes, test the full API gateway, or execute the automated live system test locally without requiring a distributed cluster.

### 🧪 Mode 0: Automated End-to-End Live Verification (Single Command)

Execute the complete live end-to-end runtime test harness. This launches the Gateway binary, verifies live socket communication, exercises authentication, tests attack defenses, provisions tenants, creates CSPRNG API keys, tests SQL grammar security, and verifies the Ed25519 cryptographic hash chain:

```bash
python scripts/e2e_live_test.py
```

**Real Test Output:**
```text
======================================================================
AEGIS PLATFORM — LIVE END-TO-END SYSTEM INTEGRATION TEST
======================================================================
[INFO] ℹ️ Found Gateway binary: gateway/target/debug/aegis-gateway.exe
[INFO] ℹ️ Launching Gateway process on port 3005...
[SUCCESS] ✅ Gateway is LIVE! Status: 200 OK, Version: 0.1.0
[INFO] ℹ️ Testing Finding 1 Defense: Attacker bypass attempt with 'admin' prefix...
[SUCCESS] ✅ Finding 1 bypass attempt successfully REJECTED (401 Unauthorized)
[SUCCESS] ✅ Invalid password for admin user rejected (401 Unauthorized)
[INFO] ℹ️ Authenticating as 'admin' with valid Argon2id credentials...
[SUCCESS] ✅ Admin authentication SUCCESS. JWT token received (197 bytes)
[INFO] ℹ️ Authenticating as 'analyst'...
[SUCCESS] ✅ Analyst authentication SUCCESS
[INFO] ℹ️ Testing RBAC boundary: Analyst requesting /admin/users...
[SUCCESS] ✅ RBAC boundary enforced: Analyst blocked from admin endpoints (403 Forbidden)
[INFO] ℹ️ Admin creating tenant 'tenant-live-cyber'...
[SUCCESS] ✅ Tenant 'tenant-live-cyber' created successfully
[INFO] ℹ️ Testing Finding 4 Defense: Generating high-entropy API key...
[SUCCESS] ✅ High-entropy CSPRNG API key created: aegis_tenant-live-cy... (Single-use secret delivered, hash kept private)
[INFO] ℹ️ Testing Finding 2 Defense: SQL Injection Statement Chaining Prevention...
[SUCCESS] ✅ SQL injection chaining attack BLOCKED by query grammar defense (400 Bad Request)
[SUCCESS] ✅ Destructive DDL query BLOCKED by query grammar defense (400 Bad Request)
[INFO] ℹ️ Testing Finding 5 Defense: Verifying tamper-evident audit log hash chain...
[SUCCESS] ✅ Audit log verified: 10 records in valid Ed25519 cryptographic hash chain
[INFO] ℹ️ Checking Rate Limiter status...
[SUCCESS] ✅ Rate Limiter Status: 200 OK (Remaining tokens: 1)
======================================================================
🎉 ALL LIVE RUNTIME & INTEGRATION TESTS PASSED WITH 100% SUCCESS!
======================================================================
```

---

### 🌐 Mode 1: Live Internet TLS Probing (No Cluster Required)

Directly interrogate public internet endpoints using the zero-copy TLS parser and JA4S engine over live TCP:

```bash
# Probe Cloudflare DNS over real WAN (computes live JA4S fingerprint)
cd packet-engine/interrogation/tls
cargo run --example live_tls_probe -- 1.1.1.1:443

# Probe GitHub or Google DNS
cargo run --example live_tls_probe -- github.com:443
cargo run --example live_tls_probe -- 8.8.8.8:443
```

---

### 🚀 Mode 2: Start the AEGIS Gateway & Interactive Swagger UI

Launch the REST/WebSocket/gRPC Gateway locally:

```bash
cd gateway
cargo run
```
The gateway binds to port `3000` (HTTP/WS) and `50051` (gRPC).

1. **Verify Health:**
   ```bash
   curl http://localhost:3000/health
   # Returns: {"status":"ok","version":"0.1.0","timestamp":"..."}
   ```
2. **Interactive Swagger UI:**
   Open your browser to:
   ```
   http://localhost:3000/swagger-ui
   ```
3. **OpenAPI 3.0 Specification:**
   ```bash
   curl http://localhost:3000/api-docs/openapi.json
   ```

---

### 🖥️ Mode 3: Interactive Web Dashboard

Launch the React 19 + Vite dashboard:

```bash
cd dashboard
npm install
npm run dev
```
Open **`http://localhost:5173`** to access the dashboard. Mock prototype data and verified live telemetry streams are clearly attributed with real-time provenance badges.

To run the automated behavioral test suite in dashboard:
```bash
npm test        # Runs Vitest behavioral & security tests
npm run lint    # Runs Oxlint code verification
```

---

### 🧠 Mode 4: Python ML Targeting Pipeline

Test the reinforcement learning target predictor and drift monitor:

```bash
cd targeting/ml
python -m venv .venv
source .venv/bin/activate       # On Windows: .venv\Scripts\Activate.ps1
pip install -r requirements.txt
pytest tests/test_ml_pipeline.py -v
```

---

## 🛠️ Architecture Overview

```
                      ┌──────────────────────────────────────────────┐
                      │              INTERNET TARGETS                │
                      └──────────────────────▲───────────────────────┘
                                             │ L4 SYN Probing (AF_XDP)
                                             │ L7 Interrogation (TLS/SSH/HTTP)
                      ┌──────────────────────┴───────────────────────┐
                      │        AEGIS PACKET ENGINE WORKSPACE         │
                      │  • af_xdp        • sequence     • traversal  │
                      │  • tls / ja4     • lzr          • behavioral │
                      └──────────────────────┬───────────────────────┘
                                             │ Raw Telemetry
                                             ▼
                      ┌──────────────────────────────────────────────┐
                      │              KAFKA EVENT BUS                 │
                      └──────────────────────┬───────────────────────┘
                                             │ Stream Ingestion
                                             ▼
                      ┌──────────────────────────────────────────────┐
                      │             APACHE FLINK PIPELINE            │
                      │   Stateful Deduplication & Protocol Fusion   │
                      └──────────────────────┬───────────────────────┘
                                             │ Parquet
                                             ▼
                      ┌──────────────────────────────────────────────┐
                      │             APACHE ICEBERG / S3              │
                      └──────────────────────▲───────────────────────┘
                                             │ Query Acceleration
                                             ▼
                      ┌──────────────────────────────────────────────┐
                      │             TRINO SQL QUERY ENGINE           │
                      └──────────────────────▲───────────────────────┘
                                             │ Protected SQL
                                             ▼
  ┌──────────────────────────────────────────────────────────────────────────┐
  │                         AEGIS SECURE API GATEWAY                         │
  │  • Argon2id Auth & RBAC          • Bounded Rate Limiting (Token Bucket)  │
  │  • AST SQL Grammar Filter        • Ed25519 Hash-Chained Audit Trail      │
  │  • CSPRNG API Keys               • gRPC & WebSocket Identity Propagation │
  └──────────────────────────┬────────────────────────────▲──────────────────┘
                             │                            │
                             ▼                            ▼
                 ┌──────────────────────┐     ┌──────────────────────┐
                 │    REACT DASHBOARD   │     │   PYTHON / GO SDKs   │
                 └──────────────────────┘     └──────────────────────┘
```

---

## 🧪 Testing & Verification Matrix

Every check in AEGIS is verified locally and through strict CI gates (no failure suppression, no masked exit codes):

| Component | Test Runner | Scope | Results |
|---|---|---|---|
| **Gateway** | `cargo test` | Argon2id auth, audit hash-chain, SQL defense, rate limiter, tenant CRUD | **19 passed; 0 failed** |
| **Enrichment** | `cargo test --workspace` | CVE correlation, geolocation inference, multi-tier cache, asset graphs | **90 passed; 0 failed** |
| **Packet Engine** | `cargo test --workspace` | JA4+ fingerprinting, LZR reclassification, cyclic traversal, AF_XDP rings | **154 passed; 0 failed** |
| **Targeting IPv6** | `cargo test --workspace` | Space-tree prefix partitioning, 6Hit reward computation | **11 passed; 0 failed** |
| **VistaScan** | `cargo test --workspace` | Visibility matrix, anchor selection, SLA coverage monitors | **19 passed; 0 failed** |
| **Python ML** | `pytest -v` | Dueling DQN, drift monitor, fail-closed ONNX export | **7 passed; 0 failed** |
| **Dashboard** | `vitest run` | SQL injection sanitization, chart data provenance indicators | **6 passed; 0 failed** |
| **Java Flink** | `./gradlew test` | Stateful event deduplication, Iceberg sink | **Passed** |
| **Live E2E** | `python scripts/e2e_live_test.py` | Live HTTP socket test across all security and data layers | **12 passed; 0 failed** |

---

## 📁 Repository Structure

```
AEGIS/
├── packet-engine/          # Core scanning workspace (AF_XDP, LZR, JA4, traversal)
├── enrichment/             # Enrichment workspace (CVE matching, asset graphs, geo)
├── gateway/                # Axum REST, WebSocket, and gRPC Gateway
├── targeting/              # Target selection (IPv6 space-trees, VistaScan, Python ML)
├── infra/                  # Infrastructure (Flink stream job, Helm charts, Dockerfiles)
├── dashboard/              # React 19 + Vite analytics frontend
├── sdks/                   # Official Python, Go, and TypeScript SDK clients
├── docs/                   # Architecture, API, and Deployment documentation
└── scripts/                # End-to-end integration and verification scripts
```

---

## 📄 License

AEGIS is licensed under the MIT License — see [LICENSE](LICENSE).

---

## 📞 Support & Security

- **Issues**: [GitHub Issues](https://github.com/Destroyer-official/AEGIS/issues)
- **Security Vulnerabilities**: security@aegis.example.com
- **API Documentation**: [docs/api.md](docs/api.md)