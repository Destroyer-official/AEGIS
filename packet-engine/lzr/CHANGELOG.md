# AEGIS LZR — Changelog

All notable changes to this crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.3.0] — 2026-09-02

### Added
- **Signature versioning & rollback** (`signature_store.rs`):
  - Git-backed `SignatureStore` — init, push, pull, rollback, list, current
  - `SemVer` struct with parse/display/ordering
  - `SignatureVersion` with timestamp, author, description, commit hash, signature count
  - CLI binary: `lzr signatures init|push|pull|rollback|list|current|export|path`
  - 9 tests: semver parsing, display, ordering, git operations, error cases
- **Hot-reload DFA** (`hot_reload.rs`):
  - `HotReloadEngine` — double-buffered Aho-Corasick with atomic pointer swap
  - `prepare()` → `swap()` two-phase reload (zero packet loss)
  - `reload()` combined prepare+swap
  - `spawn_reload_watcher()` for background SIGHUP/file-based reload
  - `EngineState` metadata: version, reload_count, signature_count
  - 11 tests: classify, swap, version tracking, concurrent reads, double reload, drop
- **FP/FN Dashboard** (`stats_server.rs`):
  - Embedded HTTP server on `std::net::TcpListener` (zero new deps)
  - `GET /classify/stats` — full per-signature FP/FN/precision/recall JSON
  - `GET /health` — health check endpoint
  - `StatsRegistry` — thread-safe per-signature counters
  - `StatsSnapshot` with timestamp, engine_version, per_signature, totals
  - 6 tests: health, stats empty, stats with data, 404, counters, engine version

### Changed
- Test count: 68 → 95 (unit+integration+doc)
- Modules added: `hot_reload`, `signature_store`, `stats_server`
- CLI binary added: `lzr` (signatures management)

---

## [1.2.0] — 2026-09-01

### Added
- **Signature corpus expansion**: 445 unique signatures across 23 categories (was 206)
  - Covers web, mail, remote, database, messaging, streaming, IoT, AI, cloud, VPN, TLS, QUIC, STUN, TURN, ICE, DNS, NTP, SNMP, RDP, VNC, SIP, proxy, banner, misc
  - Zero duplicate patterns, zero cross-category conflicts
  - `scripts/fetch_signatures.py` — Python corpus generator from ZGrab2/Censys/Shodan patterns
- **DFA serialization** (`dfa_serde.rs`):
  - `serialize_dfa()` / `deserialize_dfa()` — binary format via `bincode`
  - `DfaSnapshot` struct with version marker for forward compatibility
  - DFA load time <50ms in release mode; memory <100MB
  - 6 tests: roundtrip, bytes roundtrip, timing, memory, version mismatch, protocol verification
- **Classifier gRPC service** (`service.rs`):
  - `LzrClassifier` tonic service with `Classify`, `ClassifyBatch`, `HealthCheck` RPCs
  - Proto messages: `ClassifyRequest`, `ClassifyResponse`, `ClassifyBatchRequest/Response`, `HealthCheckResponse`
  - Validation: 64KB payload limit, 1000 batch limit
  - 6 tests: classify, batch, health, unknown, payload_too_large, batch_too_large
- **Coverage** improved from 80.5% to 89.2% line coverage

### Changed
- Signature count: 206 → 445 (2.2x expansion)
- Categories: 15 → 23
- Test count: 56 → 68 (unit+integration+doc)
- Dependencies added: `bincode`, `tonic`, `prost`, `tonic-build`, `prost-build`

---

## [1.1.0] — 2026-08-31

### Added
- **Fuzzing harness** (`lzr/fuzz/`) with 6 property-based fuzz tests covering:
  - Arbitrary byte sequences (0-2048 bytes)
  - Known protocol prefixes + random suffixes
  - Repeating patterns (pathological for Aho-Corasick)
  - Overlapping signature-like patterns
  - Edge cases (empty, single byte, all byte values)
- **Cross-crate benchmark** (`benches/cross_crate_bench.rs`) measuring end-to-end TLS → JA4 → LZR latency
- **Coverage infrastructure** via `cargo-llvm-cov` (80.5% line coverage)

### Changed
- **Signature matching engine** upgraded from per-signature regex to Aho-Corasick multi-pattern automaton + regex fallback
  - 206 signatures from CSV split into AC tier (literal prefixes) and regex tier (complex patterns)
  - O(n) single-pass scanning for common protocols
  - Priority-ordered matching across both tiers
- **Performance**: Classify latency improved from ~370ns to 1.3-2.4µs (still well under 10ms target)

### Fixed
- SMTP detection now correctly returns highest-priority match (SMTP over SMTP-220)
- IMAP/PostgreSQL/RTMP/VNC/WebSocket detection categories aligned with CSV data

### Removed
- `ja4/` crate deleted (duplicate of Dev 4's `aegis-tls::ja4`)

---

## [1.0.0] — 2026-08-30

### Added
- Initial LZR crate with 206 protocol signatures from CSV
- Aho-Corasick based multi-pattern matching
- Hybrid AC + regex engine for complex patterns
- `LzrEngine::new()`, `from_csv()`, `from_csv_file()`, `from_embedded_csv()`
- `classify(&[u8]) -> LzrMatch` — highest-priority match at position 0
- `classify_all(&[u8]) -> Vec<LzrMatch>` — all matches sorted by priority
- 15 unit tests + 40 integration tests covering all protocol categories
- Criterion benchmarks (6 test vectors)

---

## API Stability Guarantee

### v1.x → v1.y (Minor) — **No Breaking Changes**

The following API surface is **frozen** for the 1.x series:

```rust
// Core types — stable
pub enum Category { Web, Mail, Remote, Database, ... }
pub struct Signature { category, protocol, signature, priority }
pub struct LzrMatch { protocol, category, priority, matched }

// Engine — stable
pub struct LzrEngine;
impl LzrEngine {
    pub fn new() -> Self;
    pub fn with_signatures(signatures: Vec<Signature>) -> Self;
    pub fn from_csv(csv_data: &str) -> Result<Self, LzrError>;
    pub fn from_csv_file(path: &str) -> Result<Self, LzrError>;
    pub fn from_embedded_csv() -> Result<Self, LzrError>;
    pub fn classify(&self, payload: &[u8]) -> LzrMatch;
    pub fn classify_all(&self, payload: &[u8]) -> Vec<LzrMatch>;
    pub fn signature_count(&self) -> usize;
}

// Errors — stable
pub enum LzrError {
    CsvError(String),
    IoError(std::io::Error),
}
```

### Versioning Policy

| Change Type | Version Bump | Example |
|-------------|--------------|---------|
| Bug fixes, internal improvements | Patch (1.0.1) | Fix priority tie-breaking |
| New signatures added to CSV | Minor (1.1.0) | Add QUIC v2 signatures |
| New optional methods (non-breaking) | Minor (1.2.0) | Add `classify_with_context()` |
| Breaking changes (removed/renamed types) | Major (2.0.0) | Change `LzrMatch` fields |

### Dependabot / Consumer Guidance

```toml
[dependencies]
aegis-lzr = "1"  # Accepts any 1.x, no breaking changes
```

---

## Migration Guide (Future v2.0.0)

When v2.0.0 is released, breaking changes will be documented here with migration steps.

---

## Links

- [Documentation](https://docs.rs/aegis-lzr)
- [Repository](https://github.com/aegis-scanner/aegis/tree/main/packet-engine/lzr)
- [Issues](https://github.com/aegis-scanner/aegis/issues)