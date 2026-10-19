# Changelog

All notable changes to AEGIS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.2] - 2026-10-19

### Security & Hardening
- **Argon2id Authentication & RBAC**: Implemented salted **Argon2id** password hashing (`argon2 = "0.5"`) with constant-time verification against server-side user accounts, eliminating client-side role derivation.
- **Zero-Trust JWT Model**: Enforced strict fail-closed validation for `JWT_SECRET` in production ($\ge 32$ characters required).
- **SQL Injection Defense**: Built lexical grammar inspection engine stripping string literals, blocking statement chaining (unquoted semicolons), rejecting DDL/DML mutations, and enforcing result caps.
- **Cryptographic Audit Log**: Implemented monotonic 64-bit sequence numbering, previous-hash linkage (`prev_entry_hash`), canonical JSON SHA-256 payload hashing, and Ed25519 digital signatures (`verify_log()` detects deletion, reordering, and tampering).
- **CSPRNG API Keys**: 32-byte CSPRNG secrets (`rand::rngs::OsRng`), SHA-256 verifiers (purged MD5), single-use plaintext response, and scope/expiration validation.
- **Identity Propagation**: Preserved caller principal and tenant context across REST, WebSocket, and gRPC streaming queries.
- **Bounded Rate Limiting**: Capped limiter map to 10,000 entries with automatic LRU pruning and immediate non-blocking 429 Too Many Requests response.
- **Fail-Closed ML Exporter**: Exporter raises `FileNotFoundError` if model checkpoints are missing, preventing untrained models in production.
- **Live E2E Verification**: Created `scripts/e2e_live_test.py` running 12 live integration assertions across real TCP/HTTP network sockets.
- **Dashboard & CI Gates**: Added Vitest test suite (`npm test`), added `master` branch to GitHub Actions CI triggers, and removed failure suppression in fuzzing steps.

---

## [1.0.1] - 2026-10-18

### Fixed
- **Flink Job (Java)**: Configured Protobuf `java_multiple_files = true`, upgraded to Gradle 9.7.1 wrapper and `com.gradleup.shadow` 8.3.6 with `isZip64 = true`, resolved `DeliveryGuarantee` and async Flink DataStream bindings.
- **Dashboard (TypeScript/React 19)**: Relocated `SpaceTree` component into `src/components/`, removed unused imports, fixed WebSocket reconnect recursive references, added `npm test` script, achieved 0 lint errors/warnings.
- **ML Workspace (Python)**: Added comprehensive `pytest` test suite in `targeting/ml/tests/` covering DQN agent, drift detection, and ONNX export; fixed Ruff linting issues and added timezone-aware datetime usage.
- **Cargo Configuration**: Removed invalid `-fuse-ld=lld` flag for Windows MSVC builds to eliminate linker warnings.
- **Repository Hygiene**: Removed stray tracked file `1000000`, cleaned cache directories, and updated `.gitignore` for `audit-logs/`.

---

## [1.0.0] - 2026-08-30

### Added
- Initial production release of AEGIS Intelligence-Driven Internet Scanner
- **Packet Engine**: AF_XDP zero-copy, SipHash-2-4 seq encoding, cyclic permutation traversal, LZR classifier (445+ signatures)
- **Interrogation Suite**: TLS/JA4+, HTTP/2+gRPC, QUIC, SSH behavioral, LLM refusal-vector, MCP/A2A stubs
- **Targeting**: GPS co-occurrence (IPv4), 6Hit/6SENSE RL (IPv6), VistaScan visibility scheduler
- **Enrichment**: 3-tier geolocation (RPKI→RTT→WHOIS), CVE correlation, asset graph (TLS/SSH/JARM)
- **Gateway**: Axum REST/WS/gRPC, JWT/RBAC, rate limiting, audit logging, tenant isolation
- **ML**: PyTorch Dueling DQN + Ray, FastAPI serving, Triton export, drift monitoring
- **Storage**: Iceberg on MinIO/S3, Nessie catalog, Trino SQL, time-travel queries
- **Infra**: Kind cluster, Helm charts (Kafka, Flink, MinIO, Polaris, Trino, Iceberg), synthetic generator
- **Dashboard**: React 19 + Vite, TanStack Query, Recharts, WebSocket streaming
- **Testing**: 730+ unit/integration tests, cargo-fuzz targets, Kani formal verification
- **CI/CD**: GitHub Actions with matrix builds, clippy -D warnings, cargo audit, cargo fuzz, Kani proofs

### Security
- Opt-out enforcement at socket layer
- Identity transparency (User-Agent, TLS SNI)
- Adaptive rate limiting (token bucket)
- Signed audit logs (Ed25519)
- Pod security standards (restricted)

---

## [0.9.0] - 2026-07-15

### Added
- Core packet engine with AF_XDP and SipHash-2-4
- LZR classifier with Aho-Corasick + regex
- TLS/JA4+ fingerprinting suite
- IPv4 GPS co-occurrence targeting
- Kafka/Flink/Iceberg data spine

### Changed
- Refactored traversal to use constant-time modular arithmetic
- Improved LZR hot-reload performance

### Fixed
- AF_XDP memory leak in completion ring
- JA4 extension hash collision handling

---

## [0.8.0] - 2026-06-01

### Added
- Initial IPv6 targeting with 6Hit RL agent
- VistaScan visibility-aware scheduling
- Gateway with JWT auth and RBAC
- Dashboard with React + TanStack Query

### Changed
- Migrated from ZMQ to Kafka for message backbone
- Upgraded to Rust 1.98, Tokio 1.53

---

## [0.7.0] - 2026-04-15

### Added
- Enrichment pipelines: geolocation, CVE, asset graph
- Synthetic generator for development
- Kind-based local development cluster

### Fixed
- Iceberg snapshot cleanup race condition
- Flink checkpoint alignment issue