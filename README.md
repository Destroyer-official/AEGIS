# AEGIS — Intelligence-Driven Internet Scanner

[![Build Status](https://github.com/Destroyer-official/AEGIS/actions/workflows/ci.yml/badge.svg)](https://github.com/Destroyer-official/AEGIS/actions/workflows/ci.yml)
[![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-730%2B_passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.98%2B-orange.svg)]()
[![Node Version](https://img.shields.io/badge/node-26%2B-green.svg)]()
[![Python Version](https://img.shields.io/badge/python-3.14%2B-blue.svg)]()

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

## 🎬 Demo & Examples

### 🖥️ Dashboard Preview
```
┌─────────────────────────────────────────────────────────────┐
│  AEGIS Dashboard                                    🔍 ⚙️  │
├─────────────────────────────────────────────────────────────┤
│  Overview  │  Query  │  Scans  │  Assets  │  Admin          │
├────────────┼─────────┼─────────┼──────────┼─────────────────┤
│  Hosts: 1.2M    Services: 4.8M    CVEs: 12.4K    Alerts: 3  │
├─────────────────────────────────────────────────────────────┤
│  📈 Host Timeline          │  🔴 Vulnerability Distribution  │
│  ┌─────────────────────┐   │  ┌─────────────────────────┐   │
│  │ ████████░░ 1.2M      │   │  │ Critical:  ████  234    │   │
│  │ ███████░░░ 980K      │   │  │ High:      ███████  892  │   │
│  │ ██████░░░░ 720K      │   │  │ Medium:    █████████ 1.2K │   │
│  │ █████░░░░░ 510K      │   │  │ Low:       ███████████ 2.1K│   │
│  └─────────────────────┘   │  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```
*Run `cd dashboard && npm run dev` to see the live dashboard at `http://localhost:5173`*

### 🔍 API Usage Examples

#### Query Vulnerable Services (REST)
```bash
curl -X POST https://api.aegis.example.com/query \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT h.ip, s.port, s.protocol, v.cve, v.cvss FROM hosts h JOIN services s ON h.ip = s.host_ip JOIN LATERAL (SELECT unnest(s.cves) AS cve, unnest(s.cvss_scores) AS cvss) v ON true WHERE v.cvss >= 7.0 ORDER BY v.cvss DESC LIMIT 50"
  }'
```

#### Stream Query Results (WebSocket)
```javascript
const ws = new WebSocket('wss://api.aegis.example.com/ws/query', ['bearer', TOKEN]);
ws.onmessage = (event) => {
  const chunk = JSON.parse(event.data);
  console.log(`Batch: ${chunk.rows.length} rows, Last: ${chunk.is_last}`);
};

ws.send(JSON.stringify({
  sql: "SELECT * FROM services WHERE ja4 LIKE 't13d1516h2%'",
  batch_size: 1000
}));
```

#### Python SDK Example
```python
from aegis import AegisClient

client = AegisClient(
    base_url="https://api.aegis.example.com",
    username="analyst@company.com",
    password="secure-password"
)

# Find vulnerable services
results = client.query("""
    SELECT h.ip, s.port, s.protocol, v.cve, v.cvss
    FROM hosts h
    JOIN services s ON h.ip = s.host_ip
    JOIN LATERAL (
      SELECT unnest(s.cves) AS cve, unnest(s.cvss_scores) AS cvss
    ) v ON true
    WHERE v.cvss >= 7.0
    ORDER BY v.cvss DESC
    LIMIT 100
""")

for row in results:
    print(f"{row['ip']}:{row['port']} {row['protocol']} - {row['cve']} (CVSS {row['cvss']})")

# Export audit log
audit = client.admin_action("export", tenant_id="tenant-a")
print(f"Exported {len(audit)} audit entries")
```

### 🔧 Configuration Examples

#### Environment Variables (.env)
```bash
# Gateway
JWT_SECRET=your-256-bit-secret-change-in-production
JWT_EXPIRATION_SECS=3600
TRINO_COORDINATOR_URL=http://trino:8080
TRINO_CATALOG=iceberg
TRINO_SCHEMA=aegis
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=3000
GRPC_PORT=50051

# Rate Limiting
RATE_LIMIT_DEFAULT_RPS=100
RATE_LIMIT_BURST_SIZE=200

# Audit Log
AUDIT_LOG_DIR=/var/log/aegis/audit

# ML Serving
ML_MODEL_PATH=/models/6hit_dqn.pt
ML_DEVICE=cuda
```

#### Tenant Configuration (via API)
```json
{
  "action": "create",
  "config": {
    "id": "tenant-acme",
    "name": "Acme Corporation",
    "max_rate_rps": 200,
    "max_scans_per_day": 10000,
    "max_queries_per_hour": 50000,
    "data_retention_days": 180,
    "enabled": true
  }
}
```

---

## 🛠️ Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| **Core Scanning** | Rust, Tokio, AF_XDP, pnet | 1.98+ |
| **Protocol Parsing** | Custom parsers, JA4+, LZR | — |
| **Message Backbone** | Apache Kafka (KRaft) | 3.6+ |
| **Stream Processing** | Apache Flink | 1.18+ |
| **Storage** | Apache Iceberg, MinIO/S3 | 1.4+ |
| **Catalog** | Nessie (Iceberg REST) | 0.70+ |
| **Query Engine** | Trino | 440+ |
| **API Gateway** | Axum, Tokio, Tower | 0.7+ |
| **ML Targeting** | PyTorch, Ray, FastAPI | 2.3+, 2.8+, 0.100+ |
| **Dashboard** | React, Vite, TanStack Query | 19, 5, 5 |
| **SDKs** | Go, Python, TypeScript | 1.22+, 3.14+, 5.3+ |
| **Orchestration** | Kubernetes, Helm, Kind | 1.28+, 3.13+, 0.23+ |
| **CI/CD** | GitHub Actions | — |

---

---

## ⚡ Quick Start: How Normal Users & Analysts Use AEGIS

You don't need a massive Kubernetes cluster to start using AEGIS right now. Below are direct, hands-on examples for running individual probes, starting the API gateway, and visualizing data.

### 🌐 Mode 1: Live Internet TLS Probing (No Cluster Required)

Directly interrogate public internet endpoints using AEGIS zero-copy TLS parser and JA4S engine over live TCP:

```bash
# Probe Cloudflare DNS over real WAN (computes live JA4S fingerprint)
cd packet-engine/interrogation/tls
cargo run --example live_tls_probe -- 1.1.1.1:443

# Probe GitHub or any domain/IP
cargo run --example live_tls_probe -- github.com:443
cargo run --example live_tls_probe -- 8.8.8.8:443
```

**Real Live Output Example:**
```text
========================================================
  AEGIS Live TLS / JA4S Internet Probe
  Target:   1.1.1.1:443
  SNI Host: one.one.one.one
========================================================
[1/4] Connecting to 1.1.1.1:443 via TCP...
      Connected successfully!
[2/4] Sending RFC 8446 ClientHello probe (209 bytes)...
[3/4] Awaiting live TLS response from remote server...
      Received 2615 bytes from remote server!
[4/4] Parsing TLS records and calculating JA4S fingerprint...
  ├── Content-Type: Handshake
  ├── Wire Legacy Version: Tls12
  └── Record Payload Size: 122 bytes

  [ServerHello Details]
    • Wire Legacy Version: 0x0303
    • Negotiated TLS Version: 0x0304 (TLS 1.3)
    • Selected Cipher Suite: 0x1301
    • Extension Count: 2
    • Extension IDs: [51, 43]

  ========================================================
  🎯 LIVE JA4S FINGERPRINT: s13_1301_a56c5b993250
     Version: 13 | Cipher: 1301 | ExtHash: a56c5b993250
  ========================================================
```

---

### 🚀 Mode 2: Start the AEGIS Gateway & Interactive API Docs

Launch the Axum REST/WebSocket/gRPC Gateway locally:

```bash
cd gateway
cargo run
```
The gateway starts on port `3000` (HTTP/WS) and `50051` (gRPC).

1. **Verify Health:**
   ```bash
   curl http://localhost:3000/health
   # Returns: {"status":"ok","version":"0.1.0","active_queries":0,"uptime_secs":4}
   ```
2. **Interactive Swagger UI:**
   Open your browser to:
   ```
   http://localhost:3000/swagger-ui
   ```
   Explore and test all authentication, query, scanning, and user administration endpoints interactively.
3. **Download OpenAPI 3.0 Specification:**
   ```bash
   curl http://localhost:3000/api-docs/openapi.json
   ```

---

### 🖥️ Mode 3: Interactive Web Dashboard

Launch the modern React + Vite dashboard:

```bash
cd dashboard
npm install
npm run dev
```
Open **`http://localhost:5173`** in your browser to view real-time host exploration, JA4 fingerprinting analytics, and vulnerability breakdowns.

---

### 🧠 Mode 4: Python ML Targeting & IPv6 Evaluation

Test the reinforcement learning target predictor and hit rate evaluation pipeline:

```bash
cd targeting/ml
python -m venv .venv

# On Linux/macOS:
source .venv/bin/activate
# On Windows PowerShell:
# .venv\Scripts\Activate.ps1

pip install -r requirements.txt
pytest tests/test_ml_pipeline.py -v
```

---

### 🏗️ Mode 5: Full Enterprise Streaming Cluster (Optional)

For large-scale internet operations across Kafka, Flink, Iceberg, and Trino:

```bash
cd infra
./scripts/bootstrap.sh
./scripts/verify.sh
```

---

## ⚙️ Configuration

### Environment Variables
```bash
# Copy template
cp .env.example .env

# Edit required values
vim .env
```

**Key Variables:**
| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | ✅ | — | 256-bit base64 secret |
| `TRINO_COORDINATOR_URL` | ✅ | — | Trino coordinator URL |
| `KAFKA_BROKERS` | ✅ | localhost:9092 | Kafka bootstrap servers |
| `MINIO_ENDPOINT` | ✅ | http://localhost:9000 | MinIO S3 endpoint |
| `MINIO_ACCESS_KEY` | ✅ | minioadmin | MinIO access key |
| `MINIO_SECRET_KEY` | ✅ | minioadmin | MinIO secret key |
| `NESSIE_URI` | ✅ | http://localhost:19120/api/v1 | Nessie catalog URI |
| `RUST_LOG` | ❌ | info | Log level (trace/debug/info/warn/error) |

### Kubernetes Deployment
```bash
# Apply manifests
kubectl apply -k infra/k8s/overlays/production

# Or use Helm
helm install aegis infra/charts/aegis \
  --namespace aegis \
  --create-namespace \
  --set image.tag=v1.0.0 \
  --set jwt.secret=$(openssl rand -base64 64)
```

---

## 📊 Usage Examples

### Query Active Services
```sql
-- All active HTTPS services with JA4 fingerprint
SELECT s.host_ip, s.port, s.ja4, s.ja4s, s.status
FROM services s
WHERE s.protocol = 'HTTPS' AND s.status = 'active'
ORDER BY s.last_seen DESC
LIMIT 100;
```

### Find Vulnerable Services
```sql
SELECT h.ip, s.port, s.protocol, v.cve, v.cvss
FROM hosts h
JOIN services s ON h.ip = s.host_ip
JOIN LATERAL (
  SELECT unnest(s.cves) AS cve, unnest(s.cvss_scores) AS cvss
) v ON true
WHERE v.cvss >= 7.0
ORDER BY v.cvss DESC
LIMIT 50;
```

### Time-Travel Analysis
```sql
-- Host state 24 hours ago
SELECT * FROM hosts
FOR SYSTEM_TIME AS OF TIMESTAMP '2026-08-29 12:00:00'
WHERE ip = '192.0.2.1';
```

### Asset Correlation
```sql
-- Find hosts sharing TLS certificate
SELECT c.serial_number, array_agg(DISTINCT s.host_ip) AS ips
FROM certificates c
JOIN services s ON c.sha256_fingerprint = s.cert_sha256
GROUP BY c.serial_number
HAVING cardinality(array_agg(DISTINCT s.host_ip)) > 1;
```

---

## 🧪 Testing & Development

### Run All Tests
```bash
# Rust (all workspaces)
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do (cd "$d" && cargo test --workspace); done

# Dashboard
cd dashboard && npm test

# Python ML
cd targeting/ml && source .venv/bin/activate && pytest -v

# Java Flink
cd infra/flink-job && ./gradlew test
```

### Linting
```bash
# Rust
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# Python
cd targeting/ml && ruff check . && ruff format --check .

# Dashboard
cd dashboard && npm run lint
```

### Fuzzing
```bash
cd packet-engine/fuzz
cargo fuzz run fuzz_ja4 -- -max_total_time=60
cargo fuzz run fuzz_tls_record -- -max_total_time=60
```

### Kani Formal Verification
```bash
cd packet-engine/proofs
cargo kani --harness siphash_collision_bound
cargo kani --harness permutation_bijection
```

---

## 📁 Project Structure

```
AEGIS/
├── packet-engine/          # Core Rust workspace (12 crates)
├── enrichment/             # Enrichment Rust workspace (4 crates)
├── gateway/                # Axum API Gateway (standalone)
├── targeting/              # Targeting + ML (Rust + Python)
├── infra/                  # Infra as Code (Helm, Flink, Kind)
├── dashboard/              # React 19 + Vite + TanStack Query
├── sdks/                   # Go, Python, TypeScript clients
└── docs/                   # Architecture, API, Deployment, Development
```

---

## 🛡 Ethics & Safety

| Control | Implementation |
|---------|----------------|
| **Identity** | `User-Agent: AEGIS-Scanner/1.0 (+https://aegis.example.com/robots; contact@aegis.example.com)` |
| **Opt-Out** | CIDR blocklist at socket layer |
| **Rate Limiting** | Token-bucket per /24, per ASN, global PPS |
| **Audit** | Ed25519-signed append-only log |
| **Scope** | Lab-only until legal clearance |

**Legal**: CFAA (US), IT Act (India), Computer Misuse Act (UK). Use dedicated infrastructure. Consider Censys/Shodan APIs as alternatives.

---

## 🤝 Contributing

```bash
# 1. Fork & clone
# 2. Create feature branch: git checkout -b feat/amazing-feature
# 3. Make changes and run full CI locally
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  (cd "$d" && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace)
done
cargo fmt --check
(cd dashboard && npm run lint && npm test)
(cd targeting/ml && ruff check . && pytest -v)
(cd infra/flink-job && ./gradlew test)

# 4. Submit PR with clear description
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE).

---

## 🙏 Acknowledgments

Built on foundational research:
- **ZMap** (Durumeric et al., USENIX Security 2013)
- **XMap** (Li et al., ACSAC 2025)
- **GPS** (Izhikevich et al., SIGCOMM 2022)
- **LZR** (Izhikevich et al., USENIX Security 2021)
- **6Hit** (Hou et al., INFOCOM 2021)
- **Censys** (Durumeric et al., SIGCOMM 2025)
- **JA4+** (FoxIO)
- **VistaScan** (Hu et al., Sensors 2026)

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/Destroyer-official/AEGIS/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Destroyer-official/AEGIS/discussions)
- **Security**: security@aegis.example.com (PGP key in repo)
- **Docs**: [Full Documentation](docs/)

---

**Made with ❤️ by the AEGIS Team** — Building the future of internet intelligence.