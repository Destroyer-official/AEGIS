# AEGIS Development Guide

**Version:** 1.0  
**Audience:** Contributors, Maintainers

---

## 1. Environment Setup

### 1.1 Required Tools

```bash
# Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt
rustup target add x86_64-unknown-linux-gnu

# Node.js (via nvm or fnm)
fnm install 26

# Python (for ML)
python3 -m venv .venv
source .venv/bin/activate
pip install -r targeting/ml/requirements.txt

# protoc (for gateway protobuf)
# Windows (scoop): scoop install protobuf
# Linux: sudo apt install protobuf-compiler
# macOS: brew install protobuf

# Kani (formal verification)
cargo install --locked kani-verifier
```

### 1.2 IDE Configuration

**VS Code Extensions:**
- `rust-analyzer` (official)
- `Even Better TOML`
- `Error Lens`
- `GitLens`
- `REST Client`

**Settings (`.vscode/settings.json`):**
```json
{
  "rust-analyzer.checkOnSave": "clippy",
  "rust-analyzer.cargo.allTargets": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript][javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

---

## 2. Project Structure

```
AEGIS/
├── packet-engine/          # Core Rust workspace (12 crates)
├── enrichment/             # Enrichment Rust workspace (4 crates)
├── gateway/                # Axum API Gateway (standalone crate)
├── targeting/              # Targeting + ML
│   ├── ipv6/               # Rust
│   ├── ml/                 # Python (PyTorch, Ray)
│   └── vistascan/          # Rust
├── infra/                  # Infrastructure
│   ├── charts/             # Helm charts
│   ├── flink-job/          # Java (Gradle)
│   ├── synthetic-generator/ # Rust
│   └── kind/               # Kind cluster config
├── dashboard/              # React 19 + Vite
├── sdks/                   # Go, Python, TypeScript
└── docs/                   # Documentation
```

---

## 3. Build Commands

### 3.1 All Rust Workspaces

```bash
# From repo root
# Packet engine (12 crates)
cd packet-engine && cargo build --workspace --all-targets

# Enrichment (4 crates)
cd enrichment && cargo build --workspace --all-targets

# Gateway (standalone)
cd gateway && cargo build --all-targets

# Targeting (ipv6, vistascan)
cd targeting/ipv6 && cargo build --workspace --all-targets
cd targeting/vistascan && cargo build --workspace --all-targets

# Infra tools
cd infra/synthetic-generator && cargo build --release
cd infra/flink-job && ./gradlew shadowJar
```

### 3.2 Dashboard

```bash
cd dashboard
npm install
npm run build          # Production build
npm run dev            # Dev server with HMR
```

### 3.3 Python ML

```bash
cd targeting/ml
source .venv/bin/activate
pip install -r requirements.txt
```

### 3.4 Java Flink Job

```bash
cd infra/flink-job
./gradlew shadowJar
```

---

## 4. Linting & Formatting

### 4.1 Rust (All Workspaces)

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Clippy with deny-warnings (CI standard)
cargo clippy --workspace --all-targets -- -D warnings

# Fix auto-fixable warnings
cargo clippy --fix --allow-no-vcs --workspace --all-targets
```

### 4.2 Dashboard

```bash
cd dashboard
npm run lint          # oxlint
npm run build         # Vite production build (also checks types)
```

### 4.3 Python ML

```bash
cd targeting/ml
source .venv/bin/activate
ruff check .          # Lint
ruff format .         # Format
ruff check --fix .    # Auto-fix
```

### 4.4 Java Flink Job

```bash
cd infra/flink-job
./gradlew checkstyleMain
```

---

## 5. Testing

### 5.1 Rust Unit/Integration Tests

```bash
# All tests (all workspaces)
cd packet-engine && cargo test --workspace
cd enrichment && cargo test --workspace
cd gateway && cargo test
cd targeting/ipv6 && cargo test --workspace
cd targeting/vistascan && cargo test --workspace
cd infra/synthetic-generator && cargo test

# Specific crate
cargo test -p aegis-tls
cargo test -p aegis-lzr --lib

# With output
cargo test -- --nocapture
cargo test test_prime_greater_than_2pow32 -- --nocapture

# Benchmarks
cd packet-engine/sequence && cargo bench
```

### 5.2 Fuzzing

```bash
cd packet-engine/fuzz
cargo fuzz run fuzz_ja4 -- -max_total_time=60
cargo fuzz run fuzz_tls_record -- -max_total_time=60
cargo fuzz run fuzz_mcp_initialize -- -max_total_time=60
```

### 5.3 Kani Formal Verification

```bash
cd packet-engine/proofs
cargo kani --harness siphash_collision_bound
cargo kani --harness permutation_bijection
```

### 5.4 Live Internet Probing (Real WAN Validation)

Verify live TCP handshake, zero-copy TLS parsing, and JA4S calculation against live public endpoints without spinning up infrastructure:

```bash
cd packet-engine/interrogation/tls

# Test against Cloudflare (1.1.1.1)
cargo run --example live_tls_probe -- 1.1.1.1:443

# Test against GitHub
cargo run --example live_tls_probe -- github.com:443

# Test against Google DNS (8.8.8.8)
cargo run --example live_tls_probe -- 8.8.8.8:443
```

### 5.5 Dashboard Tests

```bash
cd dashboard
npm test              # Vitest / Vite build verification
npm run build         # Type checking via Vite
```

### 5.6 Python ML Tests

```bash
cd targeting/ml
source .venv/bin/activate
pytest -v
pytest tests/test_ml_pipeline.py -v
```

### 5.7 Java Flink Tests

```bash
cd infra/flink-job
./gradlew test
```

### 5.8 Disk Space Management (Clean Build Caches)

Rust and Gradle store compilation artifacts locally that can grow to ~46 GB. To reclaim this disk space without affecting git:

```bash
# Clean Rust workspaces
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  (cd "$d" && cargo clean)
done

# Clean Gradle build caches
(cd infra/flink-job && ./gradlew clean)
```

### 5.9 Run All Tests (CI Simulation)

```bash
#!/bin/bash
set -e

# Rust
for dir in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  echo "=== Testing $dir ==="
  (cd "$dir" && cargo test --workspace 2>&1 | tail -5)
done

# Dashboard
(cd dashboard && npm run lint && npm run build)

# Python
(cd targeting/ml && source .venv/bin/activate && ruff check . && pytest -q)

# Java
(cd infra/flink-job && ./gradlew test -q)

echo "✅ All tests passed"
```

---

## 6. Debugging

### 6.1 Rust Debugging

```bash
# Debug build
cargo build --workspace

# Run with logging
RUST_LOG=debug cargo run -p aegis-gateway
RUST_LOG=aegis_tls=trace cargo test -p aegis-tls -- --nocapture

# GDB/LLDB
rust-gdb target/debug/aegis-gateway
rust-lldb target/debug/aegis-gateway

# Memory profiling
cargo install flamegraph
cargo flamegraph -p aegis-lzr
```

### 6.2 Dashboard Debugging

```bash
cd dashboard
npm run dev          # Vite dev server with HMR
# Browser DevTools → React DevTools, Network, Console
```

### 6.3 Python ML Debugging

```bash
cd targeting/ml
source .venv/bin/activate
python -m pdb -m pytest tests/ -v
# Or VS Code debug configuration
```

### 6.3 Flink Job Debugging

```bash
cd infra/flink-job
# Remote debug port 5005
./gradlew run --debug-jvm
# Attach IDE debugger to localhost:5005
```

---

## 7. Profiling & Performance

### 7.1 CPU Profiling (Rust)

```bash
cargo install flamegraph
cargo flamegraph -p aegis-lzr --bin lzr
# Opens flamegraph.svg in browser

# perf (Linux)
perf record -g target/release/aegis-lzr
perf report
```

### 7.2 Memory Profiling

```bash
# heaptrack (Linux)
heaptrack target/release/aegis-lzr
heaptrack_gui heaptrack.aegis-lzr.gz

# valgrind
valgrind --tool=massif target/release/aegis-lzr
ms_print massif.out.*
```

### 7.3 Benchmarks

```bash
cd packet-engine/sequence
cargo bench --bench seq_bench

cd packet-engine/af_xdp
make bench
```

---

## 8. Common Tasks

### 8.1 Adding a New Interrogator

```bash
# 1. Create crate in packet-engine/interrogation/
cd packet-engine/interrogation
cargo new --lib myproto
# Add to packet-engine/Cargo.toml workspace.members

# 2. Implement Interrogator trait (see emerging/src/lib.rs)
# 3. Add to InterrogatorRegistry in main.rs
# 4. Add tests
# 5. Run cargo test -p myproto
```

### 8.2 Adding a New LZR Signature

```bash
# Edit packet-engine/lzr/data/signatures.csv
# Format: category,protocol,signature,priority
# Example: web,HTTP/1.1,^HTTP/1\.[01] ,50

# Reload at runtime via gRPC or restart
```

### 8.3 Adding a New ML Feature

```bash
cd targeting/ml
# 1. Add feature to dqn_model.py STATE_DIM
# 2. Update encode_state() in dqn_model.py
# 3. Retrain: python ray_train.py
# 3. Export: python export_onnx.py
# 4. Deploy: docker build -t ml-serving targeting/ml
```

### 8.4 Adding a New Dashboard Page

```bash
cd dashboard/src
# 1. Create page in pages/
# 2. Add route in App.jsx
# 3. Add nav link in App.jsx
# 4. npm run build to verify
```

---

## 9. Release Process

```bash
# 1. Update version in all Cargo.toml files
# 2. Update version in dashboard/package.json
# 3. Update CHANGELOG.md
# 4. Run full test suite
# 5. Tag release
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin v1.2.0

# GitHub Actions builds and publishes:
# - Docker images to ghcr.io
# - Python packages to PyPI
# - npm packages to npmjs
# - GitHub Release with artifacts
```

---

## 10. Useful Aliases

Add to `.bashrc` / `.zshrc`:

```bash
alias aegis-build='cargo build --workspace --all-targets'
alias aegis-test='cargo test --workspace'
alias aegis-clippy='cargo clippy --workspace --all-targets -- -D warnings'
alias aegis-fmt='cargo fmt --check'
alias aegis-full='aegis-fmt && aegis-clippy && aegis-test'

# Per-workspace
alias pe='cd ~/AEGIS/packet-engine'
alias en='cd ~/AEGIS/enrichment'
alias gw='cd ~/AEGIS/gateway'
alias tg='cd ~/AEGIS/targeting'
alias db='cd ~/AEGIS/dashboard'
alias inf='cd ~/AEGIS/infra'
```

---

## 11. Troubleshooting

| Issue | Solution |
|-------|----------|
| `protoc not found` | Install protobuf-compiler, set `PROTOC` env var |
| `linker: lld not found` | Install `lld` or remove `-C link-arg=-fuse-ld=lld` from `.cargo/config.toml` |
| `Kafka connection refused` | Check `bootstrap.sh` ran, `kubectl get pods -n aegis` |
| `protoc-gen-tonic not found` | `cargo install tonic-build` |
| `Kani: unsupported feature` | Use `--harness` with supported code only |
| `npm run dev` fails | Delete `node_modules`, `package-lock.json`, reinstall |
| `ruff` not found | `pip install ruff` in `.venv` |