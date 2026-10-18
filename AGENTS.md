# AEGIS Agent Instructions

**Project:** AEGIS — Intelligence-Driven Internet Scanner  
**Purpose:** Build, test, and lint commands for AI agents working on this codebase.

---

## 1. Repository Overview

| Workspace | Language | Build Tool | Test Command |
|-----------|----------|------------|--------------|
| `packet-engine/` | Rust | Cargo | `cargo test --workspace` |
| `enrichment/` | Rust | Cargo | `cargo test --workspace` |
| `gateway/` | Rust | Cargo | `cargo test` |
| `targeting/ipv6/` | Rust | Cargo | `cargo test --workspace` |
| `targeting/vistascan/` | Rust | Cargo | `cargo test --workspace` |
| `infra/synthetic-generator/` | Rust | Cargo | `cargo test` |
| `infra/flink-job/` | Java | Gradle | `./gradlew test` |
| `targeting/ml/` | Python | pytest | `pytest` |
| `dashboard/` | TypeScript | Vite/npm | `npm test` |

---

## 2. Required Commands

### 2.1 Build Verification (Run in Order)

```bash
# 1. All Rust workspaces compile
cd packet-engine && cargo build --workspace --all-targets
cd enrichment && cargo build --workspace --all-targets
cd gateway && cargo build --all-targets
cd targeting/ipv6 && cargo build --workspace --all-targets
cd targeting/vistascan && cargo build --workspace --all-targets
cd infra/synthetic-generator && cargo build --release

# Java
cd infra/flink-job && ./gradlew shadowJar

# Python
cd targeting/ml && source .venv/bin/activate && python -m compileall .

# Dashboard
cd dashboard && npm run build
```

### 2.2 Lint Verification (All Must Pass)

```bash
# Rust: clippy with deny-warnings
cd packet-engine && cargo clippy --workspace --all-targets -- -D warnings
cd enrichment && cargo clippy --workspace --all-targets -- -D warnings
cd gateway && cargo clippy --all-targets -- -D warnings
cd targeting/ipv6 && cargo clippy --workspace --all-targets -- -D warnings
cd targeting/vistascan && cargo clippy --workspace --all-targets -- -D warnings
cd infra/synthetic-generator && cargo clippy --all-targets -- -D warnings

# Formatting
cargo fmt --check

# Python
cd targeting/ml && ruff check .

# Dashboard
cd dashboard && npm run lint

# Java
cd infra/flink-job && ./gradlew checkstyleMain
```

### 2.3 Test Verification (All Must Pass)

```bash
# Rust unit/integration tests
cd packet-engine && cargo test --workspace
cd enrichment && cargo test --workspace
cd gateway && cargo test
cd targeting/ipv6 && cargo test --workspace
cd targeting/vistascan && cargo test --workspace
cd infra/synthetic-generator && cargo test

# Java
cd infra/flink-job && ./gradlew test

# Python
cd targeting/ml && source .venv/bin/activate && pytest -v

# Dashboard
cd dashboard && npm test
```

### 2.4 Full CI Simulation (Single Command)

```bash
#!/bin/bash
set -e

echo "=== BUILD ==="
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  echo "Building $d..."
  (cd "$d" && cargo build --workspace --all-targets 2>&1 | tail -3)
done
cd infra/flink-job && ./gradlew shadowJar -q
cd targeting/ml && source .venv/bin/activate && python -m compileall . -q
cd dashboard && npm run build 2>&1 | tail -3

echo "=== LINT ==="
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  echo "Linting $d..."
  (cd "$d" && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3)
done
cargo fmt --check
cd targeting/ml && ruff check .
cd dashboard && npm run lint 2>&1 | tail -5
cd infra/flink-job && ./gradlew checkstyleMain -q

echo "=== TEST ==="
for d in packet-engine enrichment gateway targeting/ipv6 targeting/vistascan infra/synthetic-generator; do
  echo "Testing $d..."
  (cd "$d" && cargo test --workspace 2>&1 | grep "test result:")
done
cd infra/flink-job && ./gradlew test -q 2>&1 | tail -3
cd targeting/ml && source .venv/bin/activate && pytest -q 2>&1 | tail -3
cd dashboard && npm test 2>&1 | tail -5

echo "✅ ALL CHECKS PASSED"
```

---

## 3. File Patterns to Watch

### 3.1 Critical Paths (Trigger Full CI)

```
**/Cargo.toml
**/Cargo.lock
**/build.rs
**/*.rs
**/build.gradle.kts
**/settings.gradle.kts
**/src/main/java/**/*.java
**/requirements.txt
**/pyproject.toml
**/*.py
**/package.json
**/package-lock.json
**/*.ts
**/*.tsx
**/*.vue
**/vite.config.*
**/tsconfig.json
**/build.gradle.kts
**/settings.gradle.kts
.github/workflows/*.yml
docker/**/*
infra/charts/**/*.yaml
infra/charts/**/values.yaml
```

### 3.2 Documentation Paths

```
README.md
AGENTS.md
docs/**/*.md
CHANGELOG.md
LICENSE
```

---

## 4. Common Failure Patterns & Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `protoc not found` | Missing protobuf compiler | Install protoc, set `PROTOC` env var |
| `linker: lld not found` | Missing lld linker | Install lld or remove `-fuse-ld=lld` from `.cargo/config.toml` |
| `Trait bound not satisfied: ToSchema` | Missing utoipa derive | Add `#[derive(utoipa::ToSchema)]` |
| `filter_map` infinite loop | Iterator produces Err | Use `.map_while(Result::ok)` |
| `Result<_, Box<dyn Error>>` too large | Clippy `result_large_err` | Add `#[allow(clippy::result_large_err)]` |
| `manual_is_multiple_of` | `n % d == 0` | Use `n.is_multiple_of(d)` |
| `sort_by` with `cmp` | Manual comparison | Use `sort_by_key(\|x| Reverse(x.field))` |
| `from_str` confusable | Inherent method shadows trait | Implement `std::str::FromStr` |
| `map_or` with bool | `opt.map_or(true, \|x| pred)` | Use `opt.is_none_or(pred)` |
| `unused_import` | Dead code | Remove or `#[allow(dead_code)]` with comment |

---

## 5. Performance Baselines

| Metric | Target | Command |
|--------|--------|---------|
| L4 probe velocity | ≥ 1M pps/core | `cd packet-engine/af_xdp && make bench` |
| LZR classify latency | < 10 ms | `cargo bench -p aegis-lzr` |
| Sequence encoding | < 100 ns/op | `cargo bench -p aegis-sequence` |
| ML inference latency | < 100 ms | `cd targeting/ml && python -m pytest tests/ -v` |
| Gateway P99 latency | < 200 ms | `hey -n 10000 -c 100 http://localhost:3000/health` |
| Flink checkpoint | < 5 min | Flink UI → Checkpoints tab |
| Iceberg compaction | < 30 min | MinIO metrics → compaction duration |

---

## 6. Security Checks

```bash
# Dependency audit (all workspaces)
cargo audit

# Python
cd targeting/ml && pip-audit

# Dashboard
cd dashboard && npm audit

# Java
cd infra/flink-job && ./gradlew dependencyCheckAnalyze
```

---

## 7. Pre-Commit Checklist

Before committing, ensure:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes (all Rust workspaces)
- [ ] `cargo test --workspace` passes (all Rust workspaces)
- [ ] `npm run lint` passes (dashboard)
- [ ] `ruff check .` passes (targeting/ml)
- [ ] `./gradlew checkstyleMain test` passes (flink-job)
- [ ] No `TODO`/`FIXME` without issue reference
- [ ] No `unwrap()`/`expect()` in production paths
- [ ] No hardcoded secrets/tokens
- [ ] Documentation updated for API changes
- [ ] CHANGELOG.md updated for user-facing changes

---

## 8. Release Checklist

- [ ] Version bumped in all `Cargo.toml`, `package.json`, `pyproject.toml`
- [ ] `CHANGELOG.md` updated
- [ ] All CI checks pass on `main`
- [ ] Docker images built and pushed
- [ ] Helm charts version bumped
- [ ] GitHub Release created with artifacts
- [ ] Documentation deployed