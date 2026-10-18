# Master Architectural Blueprint, Technical Specification, and Implementation Roadmap for a   Intelligence-Driven Internet Scanner

**Document Version:** 1.1  
**Classification:** Master Reference Document  
**Consolidation Scope:** 113 pages of primary technical documentation + 19-page research learning project synthesized into a single canonical reference  
**Date:** August 29, 2026  

---

## Document Purpose and Lineage

This master reference document synthesizes four independently authored research blueprints and system engineering reports into a unified, non-redundant architectural reference for a next-generation internet-wide scanning platform. Consolidating 113 pages of primary technical documentation, this report establishes a single canonical architecture by affirming consensus recommendations while explicitly evaluating and documenting architectural divergences as structured decision records.

The resulting framework outlines a complete, from-scratch platform designed to operate at  max capabilities. The system deliberately avoids runtime or compile-time dependencies on legacy scanning utilities such as ZMap, Nmap, or Masscan. Instead, it leverages the underlying academic and empirical research—including XMap, HMap, 6SENSE, 6Hit, LZR, and GPS—as conceptual prior art to inform its custom Rust- and Go-based components.

**Consolidated Source Materials:**

| Source Document | Approximate Pages | Primary Contribution |
|---|---|---|
| Research Blueprint: Internet-Wide Scanner Learning Project | 19 | Foundational research grounding, paper analysis, and phased learning plan |
| System Engineering Report: Project AEGIS Architecture | 42 | Full system architecture, component design, and technology specifications |
| Technical Specification: Project Horizon-Scanner Master Blueprint | 28 | Detailed protocol specifications, data models, and performance targets |
| Engineering Analysis: From Static Lists to Intelligence-Driven Scanning | 24 | Strategic analysis of scanning paradigms and evolution from legacy approaches |

---

# PART I: SOFTWARE ARCHITECTURE

---

## 1. Architectural Philosophy

The operational environment of  a fundamental departure from legacy brute-force packet blasting. The platform design is governed by four core architectural principles:

### 1.1 Intelligence over Brute Force

Exhaustive enumeration of the IPv6 address space (2^128 host addresses) is computationally impossible; a single /64 subnet contains 2^64 addresses, vastly exceeding the entire IPv4 address space (~4.3 billion addresses). Target selection must therefore function as an evidence-driven, adaptive search process powered by predictive statistical models and reinforcement learning rather than sequential sweeps.

The research literature confirms this imperative: GPS (Izhikevich et al., SIGCOMM 2022) demonstrated that 92.5% of all IPv4 services across all 65,535 ports can be discovered with 131x less bandwidth than exhaustive scanning by using co-occurrence probabilistic models. For IPv6, 6Hit (Hou et al., INFOCOM 2021) demonstrated reinforcement learning-based target generation achieving 3.5%–11.5% hit rates—a 7.7%–630% improvement over state-of-the-art methods—by dynamically allocating exploration resources to high-density regions based on real-time scanning feedback.

### 1.2 Streaming over Batch

Legacy internet measurement architectures treat scanning as a periodic, batch-oriented sweep that yields static snapshots after hours or days. The target architecture models internet mapping as a continuous, always-on event stream, achieving sub-minute data freshness from network probe to queryable index.

Censys's SIGCOMM 2025 architecture paper confirms this paradigm shift: in 2024, Censys processed approximately 5 billion events per day through a Command Query Responsibility Segregation (CQRS)-based architecture, decoupling write-side (scan ingestion) and read-side (user queries) processing to independently scale each path. The write side ingests scan results as commands that update entity state, journals delta-encoded events, and enqueues downstream processing asynchronously—enabling continuous freshness rather than periodic batch updates.

### 1.3 Depth over Breadth

Identifying a raw open Layer 4 port yields minimal actionable intelligence. The platform implements a strict two-phase probing model—a stateless Layer 4 discovery sweep followed immediately by a stateful, protocol-aware Layer 7 interrogation. This defeats service obfuscation, non-standard port allocations, and CDN/reverse-proxy distortion.

The research foundation is unequivocal: LZR (Izhikevich et al., USENIX Security 2021) demonstrated that under 3% of HTTP services actually live on port 80, meaning tools relying exclusively on port-number assumptions miss the vast majority of modern web applications operating on non-standard ports, under encryption, or behind WAF layers. Censys's own evaluation confirms that their predictive scanning engine still discovers a largely different set of services than comprehensive scanning—predictive methods are a supplement, not a replacement, for deep protocol interrogation.

### 1.4 Ethics, Legal Safety, and Verifiability by Design

System claims regarding host vitality, geographic location, or vulnerability status must be backed by verifiable, cryptographic evidence where available. Network probes must carry transparent identity markers, enforce strict rate-limiting, and honor opt-out lists at the core engine level from day zero.

Censys treats this as core engineering rather than public relations: they publish reverse DNS, WHOIS contact information, an identifying HTTP page, and a documented opt-out process, while handling a steady stream of abuse complaints. This document mandates identical discipline, embedding safety mechanics directly within low-level networking abstractions so they cannot be bypassed by upper-level targeting logic.

---

## 2. System Overview and Macro Data Flow

The platform is architected as a set of loosely coupled microservices deployed across geographically distributed Points of Presence (PoPs) and a centralized analytics core. The system decouples stateful analytics from stateless probing agents, permitting each tier to scale horizontally based on network throughput or processing demands.

### 2.1 Eight-Stage Continuous Operational Pipeline

Data flows through eight continuous operational stages:

**Stage 1 — Passive Intelligence Aggregation:**
Continuous ingestion of public Certificate Transparency (CT) logs, passive DNS records, BGP routing tables, RDAP/WHOIS registrations, and RPKI geofeeds to seed active targeting pipelines and minimize redundant network probing. If a host and port combination has been observed in public CT logs within a pre-configured time window, active discovery probes can be bypassed or directed straight to L7 interrogation.

**Stage 2 — Intelligent Target Generation:**
Dual-family targeting where IPv4 utilizes co-occurrence statistical models (derived from GPS) and IPv6 utilizes a dynamic reinforcement learning agent (derived from 6SENSE and 6Hit) navigating BGP-advertised prefix seeds. The targeting service computes conditional probabilities such as P(Port_a | Port_b, Application_Feature, Network_AS) to predict likely service locations across all 65,535 ports.

**Stage 3 — Two-Phase Probing Execution:**
High-velocity stateless Layer 4 SYN/UDP probing triggers stateful Layer 7 protocol interrogators upon detecting responsive ports. Phase 1 generates probes at line rate using packet-encoded sequence numbers for zero-state response matching. Phase 2 dispatches protocol-aware interrogators that complete full protocol handshakes and extract structured parameters.

**Stage 4 — Streaming Ingestion:**
Distributed scanning agents serialize observation records (L4 responses, L7 banners, TLS handshakes) into dedicated, high-throughput message topics (raw-l4-scans, tls-fingerprints, l7-banners) via Apache Kafka or Redpanda.

**Stage 5 — In-Flight Stream Processing:**
Apache Flink consumes these topics to execute real-time deduplication via sliding time-window operators, protocol heuristic reclassification (LZR), and passive context tagging before data persistence. Flink enforces exactly-once processing semantics through non-blocking barrier snapshots.

**Stage 6 — Durable Multi-Engine Storage:**
Persistence into Apache Iceberg table format running on MinIO or S3 object storage, cataloged via Apache Polaris, providing ACID table transactions, concurrent reader-writer snapshot isolation, schema evolution without data rewriting, and native time-travel queries. Every Flink checkpoint commit generates an immutable Iceberg snapshot.

**Stage 7 — Intelligence Enrichment:**
Three asynchronous intelligence pipelines: (a) multi-source geolocation resolution using RPKI-validated geofeeds, RTT triangulation, and passive WHOIS fallback; (b) automated vulnerability correlation mapping CPE strings against local NVD CVE caches; (c) asset-relationship graphing correlating shared TLS certificate serial numbers, SSH host keys, and JARM signatures.

**Stage 8 — Serving and Query Access:**
Programmatic REST/Streaming APIs and interactive graphical user interfaces backed by Trino federated SQL queries over Iceberg tables. The API Gateway provides token authentication, rate limiting, query parsing, and query routing.

### 2.2 Macro Architecture Diagram (Textual)

```
[Passive Intel Sources]          [BGP/CT/DNS Feeds]
        |                              |
        v                              v
+------------------+    +------------------------+
| Passive Intel    |    | Predictive Scan Engine |
| Aggregator       |    | (GPS/6Sense/6Hit)      |
+------------------+    +------------------------+
        |                              |
        v                              v
+--------------------------------------------------+
|           Intelligent Target Generator            |
|   (IPv4 Co-occurrence + IPv6 RL Traversal)       |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|         Scanning Points of Presence (PoPs)        |
|  [PoP-US-East] [PoP-EU-West] [PoP-APAC]         |
|     |               |              |              |
|  Phase 1: Stateless L4 Discovery (SYN/UDP)       |
|     |                                            |
|  Phase 2: Stateful L7 Interrogation              |
|  (HTTP/S, SSH, gRPC, QUIC, LLM endpoints)        |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|     Streaming Data Backbone (Kafka/Redpanda)      |
|  [raw-l4-scans] [tls-fingerprints] [l7-banners]  |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|     Apache Flink Stream Processing Cluster        |
|  - Deduplication (sliding windows)                |
|  - LZR Heuristic Reclassification                |
|  - In-Flight CT/DNS Enrichment                   |
|  - Exactly-Once Checkpointing                    |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|     Storage Lakehouse (Iceberg on S3/MinIO)       |
|  - Delta-Encoded Event Journal                    |
|  - Immutable Snapshots (Time Travel)              |
|  - Apache Polaris REST Catalog                    |
|  - Trino Interactive SQL                          |
+--------------------------------------------------+
        |                    |
        v                    v
+------------------+  +------------------+
| Enrichment       |  | Serving Layer    |
| Pipelines        |  | - API Gateway    |
| - Geolocation    |  | - Web Dashboard  |
| - CVE Correlation|  | - REST/WS/gRPC   |
| - Asset Graph    |  +------------------+
+------------------+
```

---

## 3. Detailed Component Architecture

### 3.1 Intelligent Targeting Layer

The targeting architecture segregates handling by IP address family to address the fundamental structural differences between IPv4 and IPv6.

#### 3.1.1 IPv4 Targeting Mechanics

The IPv4 space (~4.3 billion addresses) is dense, fully mapped, and highly dynamic due to cloud ephemeral hosting. The IPv4 target generator combines rapid background sweeps with predictive co-occurrence prioritization based on the GPS research paradigm.

**GPS Algorithm Architecture (from SIGCOMM 2022):**

The GPS system operates in four phases:

1. **Seed Collection Phase:** GPS collects a seed set through sub-sampled IPv4 scanning across all 65,535 ports. A sample seed scan (1% IPv4 LZR scan across all 65K ports) provides the foundational data for pattern learning.

2. **Probabilistic Model Construction:** GPS identifies the most predictive feature values using conditional probabilities across three feature categories:
   - **Transport layer correspondence:** P(Port_a | Port_b) — the probability that a host will have Port_a open given the host has Port_b open
   - **Transport and application layer correspondence:** P(Port_a | Port_b, Application_Port_b) — incorporating application-layer feature values
   - **Transport and network correspondence:** P(Port_a | Port_b, Network_IP) — incorporating network/ASN information
   - **Transport, application and network correspondence:** P(Port_a | Port_b, Application_Port_b, Network_IP) — the full conditional probability

3. **First-Service Discovery:** GPS uses network-layer features to predict at least one service on every responsive IPv4 host, computing the feature-tuple that maximizes P(Port_a) for each IP.

4. **Remaining Service Prediction:** Once at least one service is found on each host, GPS uses the full probabilistic model to predict all remaining services across all 65K ports.

GPS uses 25 unique application and network layer features spanning all TCP protocols with available banners. The computational bottleneck is parallelized using Google BigQuery, computing predictions in 13 minutes (5,870x faster than prior work on a single core). GPS achieves 92.5% of all services across all ports with 131x less bandwidth and 204x more precision compared to exhaustive scanning.

**Co-occurrence Model Implementation:**

The service computes co-occurrence probabilities between Autonomous Systems (ASNs), hosting providers, and service ports. If a target host in a specific ASN displays open web services, the model elevates the scanning priority of adjacent hosts for related administrative ports, maximizing yield per probe. This is implemented as a conditional probability matrix that maps known service patterns to predicted service locations.

#### 3.1.2 IPv6 Targeting Mechanics

Brute-force IPv6 scanning is mathematically non-viable. A single /64 subnet contains 2^64 addresses—vastly exceeding the entire IPv4 address space. The IPv6 targeting service implements a dynamic, feedback-driven search agent derived from the HMap, 6SENSE, and 6Hit architectures.

**6Hit Reinforcement Learning Architecture (from INFOCOM 2021):**

6Hit models IPv6 address discovery as a sequential decision-making process:

- **State Space Formulation:** Encodes subnet prefix features, historical hit rates, address density, allocation patterns, and BGP Autonomous System path properties. The IPv6 address space is divided into regions using divisive hierarchical clustering on seed addresses, constructing a space tree that represents a partition of the address space.

- **Action Space Formulation:** Selects specific /64 or /120 address subranges for immediate probe dispatch. Each region in the address space is associated with a prior of the scanning reward according to the distribution of seeds.

- **Reward Function Mechanics:** Positive rewards are generated upon discovering live, non-alias, security-relevant hosts. Negative rewards are applied when probing barren subnets or alias patterns where a single router responds for an entire address block. The expected reward of each region is modified based on evaluative feedback from existing scanning results.

- **Exploration-Exploitation Balance:** 6Hit manages the tradeoff between exploring new regions and exploiting known high-density areas. It employs space repartition to avoid premature convergence—stopping exploration of saturated regions and regenerating the space tree to discover new high-density areas.

**Supplementary IPv6 Discovery Mechanisms:**

- **Subnet-Router Anycast (SRA) Probing:** Targets the router interface at the base of allocated subnets, yielding significantly higher hit rates for routing infrastructure without triggering ICMP rate limits. The /64 subnet boundary is inferred by probing across different bit positions within an ISP prefix.

- **Link-Local Multicast Name Resolution (LLMNR) Queries:** Target dual-stack systems to harvest active IPv6 interfaces by sending multicast queries to the link-local scope multicast address ff02::1:3.

- **IP-ID Behavioral Analysis:** Tracks packet-level IPv6 field generation mechanics to infer host activity, leveraging differences in IP-ID implementations across IPv6 stacks.

**XMap IPv6 Scanning Architecture (from ACSAC 2025):**

XMap represents the current state-of-the-art for IPv6 network scanning, featuring:

- **Modular Architecture:** System initialization, target network space processing, address randomization generation, network protocol scanning, packet sending/receiving, and result output/storage modules.

- **Address Randomization Generation Module:** Uses a permutation modulo multiplication algorithm with the GNU Multiple Precision Arithmetic Library (GMP) to convert IPv4 or IPv6 addresses into custom large integers for randomization. Supports scanning any segment of the address space at any bit position (e.g., 2001:db8::/32-64).

- **Flexible Target Specification:** Unlike ZMap and Masscan that only specify address prefixes, XMap supports scanning any segment of the IPv4 or IPv6 address space, such as from the 16th to the 24th bit.

- **IPv6 Periphery Discovery:** XMap discovered 52.5M unique, non-aliased last-hop IPv6 addresses belonging to 52.1M distinct /64 prefixes across 12 ISPs in India, America, and China, demonstrating the feasibility of large-scale IPv6 periphery discovery.

#### 3.1.3 Passive Intelligence Aggregator

Positioned upstream of active targeting, this component continuously ingests CT logs, passive DNS, and BGP dumps.

**Certificate Transparency Integration:**
The aggregator monitors CT logs for newly issued certificates, extracting Subject Alternative Names (SANs), domain names, and IP address associations. When a host and port combination has been observed in public CT logs within a pre-configured time window, active discovery probes can be bypassed or directed straight to L7 interrogation. This suppresses redundant internet noise and provides a validation signal against active probe false positives.

**BGP Routing Intelligence:**
Continuous ingestion of BGP routing tables extracts BGP-advertised IPv6 prefixes as seed subnets for the targeting engine. Route Origin Authorizations (ROAs) provide cryptographic validation of prefix-to-ASN mappings, enabling the geolocation pipeline to validate geofeed signatures.

**Passive DNS Correlation:**
Historical DNS resolution data provides domain-to-IP mappings, MX record associations, and CNAME chains that inform targeting decisions without active probing. Censys's Active DNS data product demonstrates how passive DNS observations can be used to infer asset relationships and identify host migrations across IP addresses over time—this approach supplements active probing by providing historical context for dynamic infrastructure.

---

### 3.2 Probing & Interrogation Engine

The probing engine operates in two decoupled phases, optimizing throughput while maximizing protocol identification accuracy.

#### 3.2.1 Phase 1: Stateless L4 Discovery

Derived from ZMap and XMap principles, the Phase 1 scanner generates custom TCP SYN or UDP probes at line rate. To eliminate the O(N) memory state table required to track millions of concurrent outbound connections, the prober uses packet-encoded sequence numbers.

**Secret-Keyed Sequence Number Encoding:**

Outbound TCP sequence numbers (or UDP ephemeral source ports) are generated via a cryptographic hash (SipHash-2-4) of the target IP, target port, local secret key, and probe timestamp:

```
seq_number = SipHash-2-4(secret_key, target_ip, target_port, timestamp)
```

When an inbound SYN-ACK or RST packet arrives at the receiver thread, the sequence number is decoded and verified against the secret key. Valid responses prove probe authenticity and identify the original target without requiring memory lookups. This eliminates per-connection state entirely—the full probe identity (src_ip, dst_ip, dst_port, src_port, phase) is encoded into the 32-bit sequence number.

**Address Traversal Randomization:**

To prevent localized network saturation and avoid alerting Intrusion Detection Systems (IDS), address traversal follows a cyclic multiplicative group permutation over Z*_p (the multiplicative group of integers modulo a prime p). This scatters probes randomly across the global address space while guaranteeing every address is probed exactly once per pass.

The permutation is computed as: addr_next = (addr_current * g) mod p, where g is a generator of the multiplicative group and p is a prime slightly larger than the address space. This produces a full-period pseudo-random permutation—every address appears exactly once in the sequence, making the scan undetectable as a sequential sweep while ensuring complete coverage.

**Packet Engine Architecture:**

The system implements a multi-backend packet engine with automatic capability detection:

| Backend | Mechanism | Requirements | Throughput |
|---|---|---|---|
| AF_XDP | Zero-copy userspace ring buffer | Kernel 5.10+, CAP_BPF, libbpf | ~10M pps |
| sendmmsg | Batched sendmmsg() syscalls | Linux, CAP_NET_RAW | ~5M pps |
| pnet | Portable single-packet raw sockets | Any platform with CAP_NET_RAW | ~1M pps |

For extreme-throughput deployments, kernel-bypass libraries such as DPDK (Data Plane Development Kit) are considered as an alternative to AF_XDP, providing zero-copy packet processing directly from NIC ring buffers without kernel involvement. DPDK requires dedicated CPU cores and hugepage memory but can achieve significantly higher throughput than kernel-mediated approaches.

The engine uses a pipelined architecture with decoupled send and receive threads: a receiver thread blocks on poll(2) draining incoming packets while the main thread sends probes in parallel. Recv-key registration happens before each send to eliminate the "reply landed before we knew to listen" race condition. Up to 16 concurrent shards per address family enable linear scaling of throughput.

#### 3.2.2 Phase 2: Stateful L7 Interrogation

Any port that responds positively in Phase 1 is handed off to a pool of stateful, protocol-aware interrogators executing complete protocol handshakes and extracting structured parameters. The interrogator pool is designed as a pluggable architecture—3 to 4 core interrogators at initial deployment (HTTP/S, SSH, gRPC/TLS), with additional interrogators (MCP, A2A) added as the protocol landscape evolves.

**Pseudo-Service Detection:**

Raw L4 responsiveness is never published directly as a service indicator. Censys research demonstrates that load balancers, middleboxes, and "pseudo-services" answer on every port, distorting results—a host responding to a SYN does not reliably indicate a real service. Every Phase 1 positive must be confirmed through Phase 2 protocol handshake before being classified as a live service.

**TLS/SSL Fingerprinting (JA4 / JA4+):**

The interrogator captures the full TLS Client Hello and Server Hello, calculating JA4 fingerprints based on the following algorithm:

JA4 format: `{proto}{ver}{sni}{cipher_count}{ext_count}{alpn}_{cipher_hash}_{ext_hash}`

- **proto:** Protocol transport type (t for TCP, q for QUIC, d for DTLS)
- **ver:** 2-character TLS version (13 for TLS 1.3)
- **sni:** SNI present (d) or no SNI (i)
- **cipher_count:** 2-character count of cipher suites (ignoring GREASE values)
- **ext_count:** 2-character count of extensions (ignoring GREASE values)
- **alpn:** First and last characters of first ALPN extension value
- **cipher_hash:** Truncated SHA-256 hash of sorted cipher suite hex codes (12 characters)
- **ext_hash:** Truncated SHA-256 hash of sorted extension hex codes concatenated with signature algorithms in original order (12 characters)

Example: `t13d1516h2_8daaf6152771_e5627efa2ab1`

JA4 resists TLS extension-order randomization enforced by modern web browsers because the extension hash sorts extensions by hex value rather than preserving original order. To bypass Web Application Firewalls (WAFs) like Cloudflare that drop non-browser TLS handshakes, interrogators dynamically spoof legitimate browser Client Hello signatures from the JA4 database.

The JA4+ suite extends beyond TLS to include:
- **JA4S:** Server fingerprint from ServerHello messages
- **JA4H:** HTTP client fingerprint from request headers and cookies
- **JA4T:** TCP client OS fingerprint from SYN packets (window size, options, MSS, wscale)
- **JA4TS:** TCP server fingerprint from SYN-ACK packets
- **JA4L:** Light distance and latency estimation
- **JA4X:** X.509 certificate structure fingerprint from OID sequences
- **JA4SSH:** SSH session type classification from traffic patterns
- **JA4D/JA4D6:** DHCPv4/v6 per-packet fingerprinting

**Protocol-Agnostic Reclassification (LZR Heuristic):**

When an interrogator receives a response on a non-standard port, the LZR heuristic inspects payload semantics rather than relying on port numbers. Applied inside Apache Flink stream pipelines, incoming payload bytes are matched against regular expression signatures representing over 200 protocols. If a response on a non-standard port matches an established signature (such as `^SSH-2\.0-`), the stream job mutates the service_name field to the matched protocol and routes the record to the appropriate L7 parser topic.

LZR demonstrated that under 3% of HTTP services actually live on port 80. If port 80 returns an SMTP banner or SSH greeting, the system reclassifies the service and dispatches the correct secondary L7 handler, defeating port-obfuscation tactics.

**gRPC and HTTP/2 Negotiation:**

Interrogators explicitly send Application-Layer Protocol Negotiation (ALPN) extensions during the TLS handshake specifying h2 and grpc. Upon handshake completion, the client dispatches an HTTP/2 Connection Preface (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) followed by a SETTINGS frame. Standard HTTP/1.1 probes fail against raw gRPC endpoints, making ALPN compliance mandatory for asset detection.

**QUIC and HTTP/3 Analysis:**

Because QUIC encrypts transport payloads beyond initial connection packets, interrogators parse unencrypted initial connection headers, extracting connection IDs, transport parameters, and cryptographic versions. Interrogators craft QUIC Initial packets with short connection IDs and parse Version Negotiation packets and unencrypted transport parameter blocks.

**SSH Behavioral Fingerprinting:**

Beyond reading version strings, the interrogator tracks key-exchange algorithm preferences, public key types, and structural patterns in authentication failure responses to build server-side implementation signatures. The interrogator measures precise byte lengths and structural framing of SSH_MSG_UNIMPLEMENTED responses triggered by malformed packets.

**LLM Endpoint Behavioral Fingerprinting:**

The interrogator identifies exposed Large Language Model inference endpoints (such as Ollama, vLLM, or OpenAI-compatible APIs). Based on refusal-vector research, the interrogator sends a small, standardized set of three refusal-inducing prompts and measures numerical output variations and refusal response structures. The interrogator computes cosine similarity vectors over response tokens, creating a model-family behavioral signature (distinguishing GPT-4, Llama-3, or Claude) based on refusal phrase structure without requiring authenticated access.

AI-Infra-Guard (Tencent Zhuque Lab, Apache 2.0) is an open-source AI red teaming framework that provides deterministic fingerprinting and vulnerability matching for AI infrastructure components. Its infrastructure scan module identifies 100+ AI framework components (Ollama, vLLM, ComfyUI, Triton Inference Server, n8n) against 2,000+ CVE vulnerability rules using declarative YAML fingerprint rules evaluated by a compact Go engine. For MCP server and agent skill auditing, it employs LLM-driven agentic analysis. The framework's fingerprinting approach—matching HTTP response patterns against known signatures—is directly applicable to the platform's LLM endpoint identification pipeline.

**TCP/IP Stack Fingerprinting (Device/IoT):**

For devices lacking banners, the platform performs TCP/IP stack fingerprinting: analyzing initial TCP window size, IP Time-To-Live (TTL), IP options flags, and TCP option ordering. Machine learning models classify these network characteristics to identify underlying IoT firmware types.

---

### 3.3 Streaming Data Backbone

The platform utilizes Apache Kafka (or Redpanda/Apache Pulsar) as a durable, decoupled message backbone between distributed scanning agents and core processing pipelines.

**Message Backbone Selection Matrix:**

| Feature | Apache Kafka | Redpanda | Apache Pulsar |
|---|---|---|---|
| Architecture | Integrated storage/compute | Single binary (C++) | Separated compute/storage |
| Latency (P99) | 5–25ms (JVM GC dependent) | 1–2ms (thread-per-core) | 5–10ms (+ BookKeeper hop) |
| Throughput | Excellent (standard) | Highest per-node | Excellent (scales horizontally) |
| Operational Complexity | Moderate (KRaft mode) | Low (single binary) | High (Brokers + Bookies + ZK) |
| Ecosystem | Largest (Connect, Streams, ksqlDB) | Kafka-compatible | Built-in functions, tiered storage |
| Geo-Replication | MirrorMaker (external) | External tools | Native, built-in |
| Memory Footprint | 8–16GB (JVM heap) | 2–6GB (native) | 12–20GB (JVM + BookKeeper) |

**Recommendation:** Redpanda for new deployments requiring lowest latency and operational simplicity; Kafka for teams with existing expertise and ecosystem dependencies; Pulsar for multi-tenant platform deployments requiring native geo-replication.

**Topic Architecture:**

Raw observations are published to dedicated topics:
- `raw-l4-scans` — Raw L4 discovery responses (IP, port, protocol, response type)
- `tls-fingerprints` — JA4/JA4+ fingerprints and TLS handshake data
- `l7-banners` — Protocol-specific banner data and application-layer responses
- `scan-commands` — Target generation commands from the targeting engine
- `enrichment-events` — Events from downstream enrichment pipelines

**Apache Flink Stream Processing:**

An Apache Flink distributed stream processing cluster consumes these topics to process telemetry in-flight:

- **Deduplication:** Sliding time-window operators deduplicate duplicate responses sent by multiple geographic PoPs scanning overlapping ranges.
- **In-Flight Enrichment:** Flink joins incoming IP observation streams with cached CT log and passive DNS tables, tagging host records prior to storage persistence.
- **LZR Protocol Reclassification:** The LZR heuristic operates within Flink operators, matching payload bytes against 200+ protocol signatures and routing records to appropriate L7 parser topics.
- **Stream Checkpointing:** Flink enforces exactly-once processing semantics through non-blocking barrier snapshots. Checkpoint intervals are tuned between 1 and 5 minutes to balance streaming pipeline freshness against object storage metadata commit overhead.

**Dynamic Iceberg Sink Pattern:**

Starting with Apache Iceberg 1.10.0 and Flink 1.20+, the Dynamic Iceberg Sink enables a single Flink job to write to multiple Iceberg tables dynamically, with automatic schema evolution and table creation—no job restarts required. This is critical for the scanning platform where new Kafka topics may be added as new protocol interrogators come online. The sink automatically detects new fields in incoming records and calls the Iceberg catalog's updateSchema() API to add new columns as metadata-only operations—no existing data files are rewritten.

---

### 3.4 Storage & Query Layer Architectural Decision Record

Two primary storage paradigms exist across the research literature. This blueprint evaluates both and establishes a primary recommendation alongside an operational alternative.

#### Pattern A: Streaming Lakehouse (Primary Recommendation)

Pattern A combines Apache Iceberg table format running on MinIO or Amazon S3 object storage, cataloged via Apache Polaris (or AWS Glue Data Catalog), and queried using Trino and Apache Spark.

**Capabilities:**
- ACID table transactions with concurrent reader-writer snapshot isolation
- Schema evolution without data rewriting (add nullable columns to live tables while Flink is writing)
- Native time-travel queries: `SELECT * FROM hosts FOR SYSTEM_TIME AS OF '2026-03-01 00:00:00'`
- Every Flink checkpoint commit generates an immutable Iceberg snapshot (30-second granularity typical)
- Three query engines can access the same data without copying: Flink writes, Trino serves dashboards, Spark handles batch analytics, and DuckDB handles ad-hoc analysis

**Schema Registry:**
Using a schema registry in conjunction with Iceberg enables backward-compatible schema changes. The registry tracks schema versions and compatibility rules, ensuring that producers (Flink jobs) and consumers (Trino, Spark) agree on data formats even as the schema evolves over time.

**Snapshot Management:**
A streaming table with 1-minute commit intervals accumulates 1,440 snapshots per day, 10,080 per week. Automated maintenance (snapshot expiration, file compaction, orphan cleanup) must run continuously as a background process—triggered by actual table conditions rather than static cron schedules.

#### Pattern B: Wide-Column + Search Hybrid (Alternative Pattern)

Pattern B pairs Apache Cassandra for raw time-series telemetry storage with Elasticsearch for structured host/service lookups, and JanusGraph or Neo4j for asset relationship graphing.

**Trade-offs:**
- Offers operational familiarity for teams accustomed to traditional NoSQL write-heavy pipelines
- Provides sub-second full-text index queries out of the box
- Lacks native time-travel capabilities, requiring manual, custom-built index versioning logic
- Higher operational complexity due to managing multiple storage systems

#### Final Recommendation

Pattern A (Streaming Lakehouse) is selected as the primary system of record to ensure data immutability, ACID safety, and native time-travel auditing. If full-text interactive search performance on Trino falls short of requirements, an Elasticsearch cluster can be deployed downstream as a secondary serving index fed directly from Iceberg change-data-capture (CDC) streams.

---

### 3.5 Intelligence Enrichment Layer

Raw host records are enriched via three asynchronous intelligence pipelines:

#### 3.5.1 Multi-Source Geolocation Engine

Bypasses traditional city-level GeoIP database inaccuracies via a three-tier fallback pipeline:

**Tier 1 — Cryptographic Geolocation (Highest Precedence, Confidence: 1.0):**
Uses an automated fetcher to download operator-published geofeed CSVs (RFC 9092 / RFC 9632) via RDAP links and validates their digital signatures against RPKI Route Origin Authorizations (ROAs) using rpki-rs. Assigns a confidence score of 1.0 (Verifiable Fact).

Current adoption: Tier 1 cryptographic geofeeds are published by only ~0.25% of IPv4 blocks and ~0.30% of IPv6 blocks, making Tier 2 and Tier 3 fallbacks critical for 100% address coverage.

**Tier 2 — Active RTT Triangulation (Secondary Fallback, Confidence: 0.6–0.8):**
Executes multi-vantage-point Round-Trip Time (RTT) delay measurements from global PoPs and solves bounding constraints via CBG (Constraint-Based Geolocation) or GeoCET/NeighborGeo algorithms.

CBG (Gueye et al., IMC 2004) transforms delay measurements to geographic distance constraints using the fact that digital information travels along fiber optic cables at approximately 2/3 the speed of light. Each landmark computes a "bestline" capturing the least distorted relationship between geographic distance and network delay, calibrated in a distributed self-calibrating fashion. CBG achieves median error distances below 100 km for US datasets and below 25 km for Western Europe datasets.

GeoCET (Du et al., 2019) extends CBG using constraint-based elliptical trajectories for improved accuracy. NeighborGeo (Wang et al., 2024) employs graph structure learning with reparameterization and supervised contrastive learning to capture neighbor relationships between nodes, achieving significantly higher localization accuracy especially with unevenly distributed landmarks. Spotter is an additional RTT-based technique that uses fingerprint databases of known network paths to match observed delay patterns against labeled training data.

Recent ML-enhanced approaches (Ortega-Velazquez et al., 2025) use KNN and MLP algorithms trained on RTT fingerprints from distributed monitors, achieving mean geolocation error of 317.6 km—a 38% reduction compared to CBG baseline—with less than 5 seconds to complete the geolocation process.

**Tier 3 — Passive Registration (Tertiary Fallback, Confidence: 0.2–0.4):**
Ingests RDAP/WHOIS organization addresses, BGP AS locations, and reverse DNS naming structures. Assigns a confidence score between 0.2 and 0.4 (Probabilistic Guess).

#### 3.5.2 Vulnerability Correlation Pipeline

Normalized application identifiers extracted from banners and certificates are mapped to Common Platform Enumeration (CPE) strings. An asynchronous worker queries an internal, locally cached copy of the National Vulnerability Database (NVD) CVE index, tagging host records with matching vulnerability identifiers and CVSS severity scores.

The pipeline maintains a local NVD cache updated on a configurable schedule, performing CPE-to-CVE matching using version comparison algorithms. Enriched records include CVE identifiers, CVSS v3.1 severity scores, affected version ranges, and exploit availability indicators.

#### 3.5.3 Asset-Relationship Graphing

Correlates shared infrastructure characteristics across distinct IP addresses:
- Hosts sharing identical TLS certificate serial numbers
- Hosts sharing SSH host keys
- Hosts with matching JARM TLS fingerprints
- Hosts in the same BGP AS with similar service profiles
- Hosts resolving to the same DNS names

These correlations are linked within an asset graph to map corporate infrastructure boundaries and threat-actor command-and-control networks.

---

### 3.6 AI/ML Layer

Machine learning is embedded across four operational functions:

#### 3.6.1 RL IPv6 Target Traversal

The core targeting model optimizing IPv6 address block exploration based on live probe feedback. Implemented using PyTorch and served via a low-latency FastAPI microservice. The model uses a Deep Q-Network (DQN) architecture where:
- **State:** Encoded subnet prefix features, historical hit rates, BGP path properties
- **Action:** Selection of specific /64 or /120 address subranges
- **Reward:** Discovery of live, non-alias hosts (+1) vs. barren subnets or alias patterns (-1)

Training data is collected from Flink execution feedback, with the model retrained on a continuous schedule using Ray for distributed training orchestration.

#### 3.6.2 Adaptive Scan Prioritization

A predictive gradient-boosting model analyzing historical change frequencies to allocate higher scan cadence to dynamic cloud subnets while reducing probe frequency on static infrastructure. The model predicts host churn rates based on ASN, hosting provider, port, and time-of-day features. The scheduler prioritizes tasks based on historical change frequency, reported vulnerabilities, and user-defined interest—allowing operators to elevate specific subnets or ASNs for increased scanning cadence.

VistaScan (Hu et al., Sensors 2026) demonstrates a visibility-aware approach to distributed scanning, constructing a Visibility Matrix through anchor-based probing that quantifies per-node visibility for each CIDR block. By assigning scanning tasks to the most capable node based on visibility consistency within CIDR blocks, VistaScan achieves near-optimal coverage (97.95%) while reducing probe volume by 64–93% and completion time by 13–19× compared to centralized approaches.

#### 3.6.3 Anomaly Detection Pipeline

Isolation Forest models operating on Flink telemetry streams to flag anomalous network behavior, such as rogue services or honeypots exhibiting non-standard protocol stack signatures. The anomaly detector operates on feature vectors including response timing distributions, banner entropy measurements, and protocol compliance scores.

#### 3.6.4 LLM-Assisted Intelligence Summarization

Downstream user-facing capability utilizing open-weights LLMs to convert raw JSON host records, certificate chains, and CVE tags into natural-language threat summaries. LLMs operate strictly as presentation-layer summarizers and are barred from injecting unverified facts into the database. Cisco's Skill Scanner demonstrates a complementary "LLM-as-a-judge" pattern, combining rule-based detection with LLM evaluation to assess vulnerability severity and suggest remediation—this pattern can be adopted for automated severity scoring of discovered services.

---

### 3.7 Serving Layer: API Gateway & Web UI

The serving layer exposes the platform via dual interfaces:

#### 3.7.1 API Gateway

Developed in Rust (Axum/Actix-web), providing RESTful and Streaming WebSocket/gRPC APIs modeled on Shodan and Censys query structures. The gateway handles:
- API token authentication with role-based access control
- Rate limiting using token-bucket algorithms
- Query parsing and validation
- Trino/Iceberg query routing
- Response caching for frequently accessed queries
- Streaming responses for large result sets

#### 3.7.2 Web Dashboard

React/Vue.js frontend rendering:
- Interactive asset lookups with full-text search
- Time-travel history charts showing host evolution over time
- Global network topology visualizations
- Vulnerability exposure metrics and CVE severity distributions
- Certificate transparency monitoring dashboards

---

### 3.8 Orchestration & Deployment

The entire software lifecycle is containerized via Docker and orchestrated via Kubernetes. A specialized Scheduler & Task Manager Operator maintains the system scanning queues, allocating jobs to stateless scanning agent pods distributed across cloud regions and physical PoPs. The scheduler maintains a task queue (Redis or in-memory structure initially) that prioritizes jobs based on historical change frequency, reported vulnerabilities, and user-defined interest, allocating higher scan cadence to dynamic subnets.

**Container Architecture:**
- Scanning agent pods are stateless and horizontally scalable
- Kafka/Redpanda brokers run as StatefulSets with persistent volumes
- Flink TaskManagers auto-scale based on consumer lag
- Iceberg tables stored on MinIO (S3-compatible) with lifecycle policies
- Trino coordinators and workers run as Deployments with auto-scaling

**Global PoP Distribution:**
Following the Censys model of multi-vantage-point deployment (Chicago, Frankfurt, Hong Kong), the platform distributes scanning agents across geographically diverse Points of Presence to overcome visibility issues caused by network partitioning, geoblocking, routing anomalies, and national firewalls.

---

### 3.9 Ethics, Legal, and Operational Safety

Ethics controls are enforced at the core network abstraction layer:

#### 3.9.1 Identity Transparency

All HTTP, TLS, and L7 probes inject explicit identification strings containing a dedicated domain and contact email address. The User-Agent header includes:
```
AEGIS-Scanner/1.0 (+https://aegis.example.com/robots; contact@aegis.example.com)
```

#### 3.9.2 Automated Opt-Out Enforcement

The core prober loads an internal blocklist before dispatching packets. Operators opting out have their CIDR blocks drop-routed at the prober interface level. The opt-out mechanism is checked at the socket abstraction layer—it cannot be bypassed by upper-level targeting logic.

#### 3.9.3 Adaptive Rate Limiting

Token-bucket rate limiters enforce local packet caps per target network to prevent accidental Denial-of-Service (DoS) conditions. Rate limits are configurable per-target and include:
- Global packets-per-second ceiling
- Per-/24 network rate caps
- Per-ASN aggregate rate limits
- Exponential backoff on complaint signals

#### 3.9.4 Legal Authorization Boundaries

Initial deployment limits active scanning to authorized lab environments and explicitly owned network ranges until formal legal clearance under applicable frameworks is completed. Applicable legal frameworks include the U.S. Computer Fraud and Abuse Act (CFAA), India's Information Technology Act, and the UK Computer Misuse Act. Cloud provider Terms of Service (AWS, GCP, Azure, DigitalOcean) explicitly prohibit unauthorized port scanning of the wider Internet from their infrastructure—scanning agents must be deployed on dedicated infrastructure or explicitly authorized environments. The system maintains an audit log of all scanning activity with timestamps, target ranges, and authorization references.

**Alternative Data Source Strategy:**
Rather than scanning the entire Internet from scratch, teams should consider leveraging Censys's or Shodan's own API and research-access programs. Over 500 published papers have used Censys data for research rather than conducting independent scans. This approach provides access to production-quality datasets without the operational overhead and legal exposure of global scanning infrastructure.

---

# PART II: MASTER TECHNICAL SPECIFICATION

---

## 4. Research Foundation and Prior Art

The platform is informed by the following foundational research, which serves as conceptual prior art without creating runtime dependencies:

### 4.1 Core Scanning Engines

**ZMap (Durumeric et al., USENIX Security 2013):**
The foundational packet-engine design introducing stateless SYN scanning with cyclic multiplicative group permutation for address traversal. Achieves full public IPv4 space scanning on one port from one host in under 45 minutes on 1 Gbps, ~5 minutes with 10 GigE + PF_RING. ZMap's own paper found that sending only one probe per host still achieves ~98% coverage, since Internet path loss dominates over retries anyway.

**XMap (Li et al., ACSAC 2025):**
The modern academic successor to ZMap, featuring comprehensive re-implementation with IPv6 support, modular architecture, address randomization using GMP library, and flexible target specification supporting any bit-range segment of IPv4 or IPv6 address space. Referenced in 52 research papers with 450+ GitHub stars.

**Masscan (Rob Graham):**
Alternative lineage with same conceptual approach, custom TCP/IP stack, different codebase. Provides reference implementation for high-speed packet generation.

### 4.2 Intelligence and Prediction

**GPS (Izhikevich et al., SIGCOMM 2022):**
Predictive scanning system that finds 92.5% of all IPv4 services across all 65K ports using conditional probability models. Computes predictions in 13 minutes using BigQuery parallelization. Open source at github.com/stanford-esrg/gps.

**LZR (Izhikevich et al., USENIX Security 2021):**
Protocol-agnostic reclassification heuristic demonstrating that under 3% of HTTP services live on port 80. Provides the foundation for the platform's payload-based service identification.

### 4.3 IPv6 Discovery

**6Hit (Hou et al., INFOCOM 2021):**
Reinforcement learning-based target generation achieving 3.5%–11.5% hit rates with 7.7%–630% improvement over state-of-the-art. Uses divisive hierarchical clustering on seeds to construct space trees for address space partitioning.

**6Sense (Hsu et al., 2024):**
State-of-the-art IPv6 scanning referenced in Censys's SIGCOMM 2025 paper as the model Censys is adopting for their emerging IPv6 scanning capabilities.

**AddrMiner (Song et al., USENIX ATC 2022):**
Global active IPv6 address discovery using structural inference and seed-based generation.

### 4.4 Deployment and Operations

**Censys (Durumeric et al., SIGCOMM 2025):**
The definitive reference for production internet-wide scanning architecture. Details CQRS-based storage processing ~5B events/day, two-phase scanning (ZMap for discovery, ZGrab2 for L7 interrogation), 72-hour staleness eviction, 10K+ static fingerprints, and multi-PoP deployment across North America, Europe, and Asia. Censys's proprietary scanning engine is a Go reimplementation of the ZMap concept—not a different architecture. Their read side employs three distinct query paths: fast key-value lookups (Bigtable-style), a full-text/field search engine (Lucene-style), and a columnar analytics warehouse (BigQuery-style) for longitudinal research queries. A full 65K-port sweep of IPv4 takes Censys approximately 9 months even at ~17 Gbps continuous sustained throughput. Censys maintains 794M IPv4 services in their index, but notably does not perform comprehensive IPv6 scanning—tracking IPv6 addresses passively from CT logs, DNS, and HTTP redirects rather than brute-force probing the address space. In May 2025, Censys dropped approximately 492 million IPv6 records, indicating the poor quality of historical IPv6 scan data. Censys's own data confirms that raw L4 responsiveness is unreliable—load balancers, middleboxes, and pseudo-services answer on every port, meaning raw L4 data is never published directly.

**On the Origin of Scanning (Wan et al., IMC 2020):**
Demonstrates why multi-vantage-point deployment is essential—single vantage points miss services behind geoblocking, routing anomalies, and national firewalls.

---

## 5. Recommended Technology Stack

### 5.1 Systems Implementation Language Decision

Rust is selected as the primary implementation language for performance-critical and packet-crafting modules. Rust's compile-time memory safety (ownership model) eliminates buffer overflows, double-frees, and use-after-free vulnerabilities when parsing untrusted, adversarial network packets at line rate. Its asynchronous runtime (tokio) provides non-blocking I/O performance on par with C/C++. Go is approved specifically for rapid prototyping of complex Layer 7 application interrogators due to its mature standard network libraries.

**Language Justification Matrix:**

| Requirement | Rust | Go | Justification |
|---|---|---|---|
| Memory safety | Compile-time (ownership) | Runtime (GC) | Rust eliminates entire vulnerability classes without GC overhead |
| Async I/O | tokio (epoll/kqueue/io_uring) | goroutines (scheduler) | Comparable performance; Rust offers finer control |
| Raw packet crafting | pnet, nix, raw sockets | Limited (net package) | Rust's pnet provides zero-copy packet construction |
| Protocol parsing | Zero-cost abstractions | Reflection-based | Rust's type system enables compile-time protocol validation |
| Performance | Near-C | 10–20% overhead | Critical for line-rate packet processing |
| Ecosystem maturity | Growing rapidly | Mature networking | Go preferred for rapid L7 protocol prototyping |

### 5.2 Complete Technology Stack

| Architectural Layer | Recommended Technology | Technical Justification |
|---|---|---|
| Core Systems / L4 Probing | Rust (tokio, pnet, raw sockets, nix) | Memory safety without GC overhead; zero-cost abstractions for raw packet crafting. |
| L7 Interrogators | Rust (Primary) / Go (Secondary) | Rapid protocol parser development; Go's standard crypto/tls and HTTP/2 stacks allow fast iteration. |
| Message Backbone | Apache Kafka / Redpanda / Apache Pulsar | High-throughput, durable stream buffering decoupling scanning agents from stateful processors. |
| Stream Processing | Apache Flink | Stateful, low-latency stream processing with exactly-once checkpointing semantics. |
| Storage Engine | Apache Iceberg on MinIO / S3 | ACID table semantics, snapshot isolation, time-travel queries, and schema evolution. |
| Metadata Catalog | Apache Polaris (REST Catalog) | Vendor-neutral, open-source REST catalog enabling shared metadata across Trino, Spark, and Flink. |
| Interactive Query Engine | Trino | Distributed SQL query engine optimized for fast interactive execution over Iceberg tables. |
| ML & Targeting Infrastructure | Python (PyTorch, Ray), FastAPI | Standard ecosystem for training reinforcement learning agents; served via low-latency REST microservices. |
| Geolocation & RPKI | Rust (rpki-rs) | Fast, cryptographically secure validation of RPKI Route Origin Authorizations and geofeeds. |
| Container Orchestration | Kubernetes | Automated deployment, horizontal scaling, self-healing, and task scheduling across global PoPs. |
| API Gateway | Rust (Axum / Actix-web) | Asynchronous, high-concurrency API layer for token auth, rate limiting, and query translation. |
| Web Dashboard | React / Vue.js | Responsive visualization of network topology, asset histories, and vulnerability metrics. |

---

## 6. Protocol & Fingerprinting Specifications

### 6.1 TLS Fingerprinting — JA4 / JA4+

The engine parses raw TLS Client Hello packets and computes JA4 fingerprints using the following algorithm:

**Fingerprint Construction:**

1. Parse TLS Client Hello record
2. Skip GREASE values throughout
3. Determine protocol transport: 't' (TCP), 'q' (QUIC), 'd' (DTLS)
4. Extract 2-character TLS version (e.g., '13' for TLS 1.3)
5. Check SNI extension: 'd' (domain present) or 'i' (IP address)
6. Count cipher suites (excluding GREASE): 2-character count
7. Count extensions (excluding GREASE): 2-character count
8. Extract first ALPN extension value: first and last characters
9. Sort cipher suite hex codes alphabetically, compute SHA-256, truncate to 12 chars
10. Sort extension hex codes alphabetically (excluding SNI 0x0000 and ALPN 0x0010), append signature algorithms in original order, compute SHA-256, truncate to 12 chars

**Fingerprint Format:** `{proto}{ver}{sni}{ciphers}{exts}{alpn}_{cipher_hash}_{ext_hash}`

**Example:** `t13d1516h2_8daaf6152771_e5627efa2ab1`

**Anti-Evasion Properties:**
- Sorting extensions by hex value resists Chrome's extension permutation randomization
- Including signature algorithms in original order captures client implementation differences
- SNI and ALPN excluded from extension hash to maintain consistency across domain/IP/ALPN variations
- GREASE values excluded throughout to prevent fingerprint inflation

### 6.2 Protocol-Agnostic Reclassification (LZR Heuristic)

Applied inside Apache Flink stream pipelines:

1. Incoming payload bytes are matched against regular expression signatures representing 200+ protocols
2. Signatures are ordered from most specific to most general
3. If a response on a non-standard port matches an established signature (e.g., `^SSH-2\.0-`, `^220 .* SMTP`, `^HTTP/1\.[01]`), the stream job:
   - Mutates the `service_name` field to the matched protocol
   - Routes the record to the appropriate L7 parser topic
   - Updates the `detection_method` field to indicate heuristic reclassification

**Signature Categories:**
- Web protocols: HTTP/1.x, HTTP/2, WebSocket, gRPC
- Mail protocols: SMTP, POP3, IMAP, Exchange
- Remote access: SSH, Telnet, RDP, VNC
- Databases: MySQL, PostgreSQL, MongoDB, Redis, Elasticsearch
- Messaging: AMQP, MQTT, STOMP
- Streaming: RTSP, RTMP, HLS
- IoT/Industrial: Modbus, BACnet, OPC-UA
- AI/ML endpoints: Ollama, vLLM, OpenAI-compatible APIs

### 6.3 Modern Protocol Negotiation Specifications

**gRPC / HTTP/2:**
The L7 interrogator constructs a TLS Client Hello containing the ALPN extension populated with `\x02h2\x05grpc`. Upon handshake completion, the client dispatches an HTTP/2 Connection Preface (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) followed by a SETTINGS frame with appropriate initial window size and maximum concurrent streams. As a forward-looking note, recent gRPC releases now default to post-quantum cryptography in their TLS key exchange, which the interrogator suite should track as the broader ecosystem migrates.

**QUIC / HTTP/3:**
Interrogators craft QUIC Initial packets with:
- Version field set to the supported QUIC version
- Short connection IDs (1–4 bytes)
- Frame types: CRYPTO (for TLS handshake), PADDING
- Parser extracts Version Negotiation packets and parses unencrypted transport parameter blocks including max_data, max_stream_data, initial_max_streams_bidi

### 6.4 Behavioral Fingerprinting Suite

**Active TLS Entropy Analysis:**
Measures Shannon entropy across TLS handshake field lengths and parameter distributions to derive implementation signatures. Different TLS stacks (OpenSSL, BoringSSL, LibreSSL, Windows SChannel) produce measurably different entropy distributions.

**SSH Behavioral Analysis:**
Measures precise byte lengths and structural framing of SSH_MSG_UNIMPLEMENTED responses triggered by malformed packets. Different SSH implementations (OpenSSH, Dropbear, libssh, Cisco SSH) produce distinct response patterns.

**LLM Refusal Vector Analysis:**
Dispatches three standardized prompts designed to trigger safety refusal logic:
1. A request to perform a clearly harmful action
2. A request for disallowed content
3. A jailbreak attempt

The interrogator computes cosine similarity vectors over response tokens, identifying model implementations based on refusal phrase structure. Different model families (Llama, GPT, Claude, Gemini) produce distinct refusal patterns.

**Website Fingerprinting (WF):**
Analyzes encrypted traffic patterns—packet sizes, timing, and sequencing—to infer the specific website or application being served over an encrypted connection, without decrypting anything. Key protocols: HTTPS, Tor. WF operates on the principle that different web applications produce distinguishable traffic patterns even when encrypted, enabling classification of target services based purely on network-layer observations.

**JA4 Bot Score:**
JA4 fingerprints can be augmented with a "bot score" metric that quantifies the likelihood a client is automated rather than human. This is derived from TLS handshake anomalies, HTTP header ordering, and request cadence patterns. Bot scores enable the scanner to distinguish between real user-facing services and automated infrastructure (load balancers, CDNs, monitoring agents).

### 6.5 Device & IoT Fingerprinting

For devices lacking banners, the platform performs TCP/IP stack fingerprinting:
- Initial TCP window size
- IP Time-To-Live (TTL) value
- IP options flags (LSRR, SSRR, Record Route)
- TCP option ordering and values (MSS, Window Scale, SACK, Timestamps)
- TCP initial sequence number generation pattern
- Don't Fragment (DF) flag behavior

Machine learning models (Random Forest classifiers trained on labeled datasets) classify these network characteristics to identify underlying IoT firmware types and operating systems.

**RF/Physical-Layer Fingerprinting:**
For wireless and IoT devices where hardware-level radio characteristics are observable, the platform can perform RF physical-layer fingerprinting. This technique identifies devices based on unique radio frequency hardware characteristics—transmitter distortions, power amplifier signatures, and modulation anomalies—even when software-level information is absent. This complements network-layer fingerprinting by providing a hardware-level identification channel for devices operating on wireless protocols.

**Emerging Protocol Interrogators (MCP, A2A):**

The interrogator architecture must accommodate expanding protocol landscapes. Machine Control Protocol (MCP) and Agent-to-Agent (A2A) protocols represent emerging application-level protocols that LLM and AI agent infrastructure will increasingly expose. New interrogators for protocols like MCP or A2A can be plugged into the modular framework as their specifications stabilize. Security assessment of MCP endpoints (arxiv 2608.00150) indicates that these protocols will require specialized fingerprinting approaches similar to existing LLM endpoint analysis.

---

## 7. Data Model Specification

The logical data schema adopts an entity-centric design structured around three core record types:

### 7.1 Host Record

Keyed by IPv4/IPv6 address. Stores:
- **Primary identifiers:** IP address (IPv4 or IPv6), host ID (internal UUID)
- **Network attributes:** ASN (Autonomous System Number), registered organization, BGP prefix, BGP AS path
- **DNS data:** Reverse DNS (rDNS), forward DNS correlations
- **Geolocation:** Multi-tier geolocation data with confidence scores (tier_used, latitude, longitude, city, country, confidence_score)
- **Activity timestamps:** first_seen, last_seen, scan_count
- **Status:** active, pending_eviction, evicted
- **Relationships:** One-to-many with Service records

### 7.2 Service Record

Keyed by Host IP + transport protocol + port number. Captures:
- **Service identifiers:** host_ip, transport_protocol (TCP/UDP), port_number
- **Protocol data:** reclassified_protocol_type (post-LZR application), detection_method (port-number, lzl-heuristic, active-probe)
- **Banner data:** raw_banner (hex-encoded), sanitized_banner (UTF-8 safe), banner_hash
- **TLS data:** ja4_fingerprint, ja4s_fingerprint, tls_version, cipher_suite, certificate_chain
- **Application data:** http_method, http_status, ssh_version, ssh_key_exchange_algorithms
- **Security data:** array of correlated CVE identifiers, CVSS scores
- **Activity timestamps:** first_seen, last_seen, reprobe_interval
- **Status:** active, pending_eviction, evicted
- **Relationships:** Many-to-many with Certificate records

### 7.3 Certificate Record

Keyed by SHA-256 certificate hash. Stores:
- **Certificate identifiers:** sha256_fingerprint, serial_number
- **Raw data:** Base64-encoded certificate material
- **Parsed fields:** subject, issuer, subject_cn, issuer_cn, organization
- **SANs:** Subject Alternative Names (DNS names, IP addresses)
- **Validity:** not_before, not_after, is_expired
- **Chain data:** certificate_chain (ordered list of issuer certificates)
- **RPKI data:** rpki_validation_status, roa_prefix
- **Activity timestamps:** first_seen, last_seen
- **Relationships:** Many-to-many with Service records

### 7.4 Journaling and Staleness Eviction Mechanics

**Delta-Encoded Event Journaling:**
Updates are stored as immutable event logs with monotonic sequence numbers. Overwrites are prohibited to maintain Iceberg time-travel isolation. Each event records only the delta (changed fields) between the previous and current state of an entity, minimizing storage overhead.

**72-Hour Staleness Eviction Policy:**
If a service fails to respond during a scheduled re-probe, its status is flagged as `pending_eviction`. If unreachability persists across subsequent checks for 72 consecutive hours, a tombstone event drops the service from the active view. This policy guarantees that live index accuracy remains above 90%.

Censys's own data validates this approach: their 72-hour eviction policy is a deliberate accuracy-over-coverage tradeoff, and they explicitly note that this makes their dataset an underestimate of what is really out there.

**Additive Schema Evolution:**
New attributes (such as post-quantum TLS signatures) are added as nullable columns, maintaining backward compatibility with historical datasets. Existing rows return NULL for new columns; new rows populate them. This operation is metadata-only in Iceberg—no existing data files are rewritten.

---

## 8. Geolocation Technical Specification

The geolocation pipeline evaluates geographic data via strict fallback precedence:

### 8.1 Tier 1: Cryptographic Geolocation (Highest Precedence)

**Confidence Score:** 1.0 (Verifiable Fact)

**Process:**
1. Automated fetcher identifies RDAP links for target IP prefixes
2. Downloads RFC 9092 / RFC 9632 Geofeed CSVs from operator-published URLs
3. Validates digital signatures against RPKI Route Origin Authorizations (ROAs) using rpki-rs
4. Extracts verified latitude, longitude, and accuracy radius from signed geofeed data
5. Assigns confidence score of 1.0 indicating cryptographically verified location

**Current Adoption:**
- ~0.25% of IPv4 blocks publish signed geofeeds
- ~0.30% of IPv6 blocks publish signed geofeeds
- Adoption is highly uneven, with RIPE NCC's region leading significantly while other RIR regions lag behind
- Adoption is growing but remains insufficient for standalone coverage

**Real-World Geofeed Quality Failures:**
Beyond low adoption, published geofeeds exhibit significant quality problems that necessitate the multi-tier fallback architecture:
- **Insecure transport:** A large share of published feeds use plain HTTP instead of HTTPS, exposing geofeed data to interception and tampering.
- **Severe formatting errors:** Data files are plagued by formatting errors that render a substantial portion of available data unusable.
- **RPKI authentication rigidity:** The mandatory RPKI-based authentication mechanism is considered too rigid for real-world deployment, where IP blocks are frequently leased or reassigned between organizations—breaking the cryptographic chain of custody.
- **Rogue ISP tampering:** Rogue ISPs can tamper with or lie about their geofeed data, undermining trust in unsigned feeds.

These failure modes explain why the geolocation module cannot rely solely on this cutting-edge technology and requires mandatory fallback tiers.

### 8.2 Tier 2: Active RTT Triangulation (Secondary Fallback)

**Confidence Score:** 0.6–0.8 (Geometric Estimate)

**CBG Algorithm (Constraint-Based Geolocation):**
1. Select K landmarks with known geographic locations from global PoP network
2. Measure minimum RTT from each landmark to target host (multiple probes, minimum selected)
3. Self-calibrate delay measurements using "bestline" computation between landmark pairs
4. Transform delay measurements to distance constraints using speed-of-light-in-fiber approximation (approximately 2/3 speed of light in vacuum)
5. Apply multilateration with distance constraints to establish geographic region containing target
6. Compute centroid of feasible region as point estimate
7. Assign confidence region based on constraint tightness

**GeoCET Enhancement:**
Extends CBG using constraint-based elliptical trajectories, providing tighter bounds on feasible regions by considering the geometric properties of delay-distance relationships.

**NeighborGeo Enhancement:**
Employs graph structure learning with reparameterization and supervised contrastive learning to capture neighbor relationships between nodes. Uses GNN-based approaches to model IP geolocation as a node regression problem in an attribute graph, achieving higher accuracy especially with unevenly distributed landmarks.

### 8.3 Tier 3: Passive Registration (Tertiary Fallback)

**Confidence Score:** 0.2–0.4 (Probabilistic Guess)

**Data Sources:**
- MaxMind-style GeoIP databases built from RIR allocation records (WHOIS/RDAP)
- RDAP/WHOIS registration organization addresses
- BGP AS registered locations
- Reverse DNS naming structures (e.g., city abbreviations in hostnames)
- Content-based inference from HTTP responses, TLS certificates, and rDNS

**Accuracy Notes:**
Content-based inference (reading HTTP responses, TLS certs, rDNS names, WHOIS org address, BGP AS registered country) is often more reliable than latency triangulation for determining "which country/city is this asset associated with"—which is typically what "device location" means in practice.

---

## 9. Performance & Quality Targets

| Metric | Target Value | Engineering Constraint / Rationale |
|---|---|---|
| L4 Probe Velocity | >= 10,000,000 probes/sec global | Reference benchmark: Censys global engine achieves ~26.5M probes/sec; XMap achieves full IPv4 in <5 minutes at 10 GigE |
| Data Freshness | < 60 seconds probe-to-query | Enabled by streaming Kafka/Flink architecture with Dynamic Iceberg Sink |
| Flink Checkpoint Cadence | 1 to 5 minutes | Balances query freshness against object storage snapshot overhead |
| Live-Service Accuracy | > 90% confirmed live | Maintained via the 72-hour staleness eviction policy |
| New Service Discovery | < 12.3 hours global latency | Reference benchmark: Censys discovers new assets in 12.3h vs Shodan's 76.5h |
| Live-Service Accuracy | > 90% confirmed live | Shodan overreports: only 68% of listed services were live vs. Censys's 92% confirmed-live rate |
| Query Latency (API) | < 200 ms point lookups | Accelerated by key-value indexes or Elasticsearch caching |
| JA4 Fingerprint Computation | < 1 ms per handshake | In-line with TLS handshake processing |
| LZR Protocol Detection | < 10 ms per payload | Regex matching against 200+ signatures within Flink operator |
| Geolocation Accuracy (Tier 1) | < 100 meters | Cryptographically verified geofeed data |
| Geolocation Accuracy (Tier 2) | < 100 km (median) | CBG with 3+ landmarks; <25 km in dense regions |
| Geolocation Accuracy (Tier 3) | City-level | WHOIS/BGP registration data |
| RL Targeting Hit Rate | > 5% (IPv6) | Reference: 6Hit achieves 3.5%–11.5% across 8 datasets |
| Data Volume | ~500 TB/year post-compression | Reference: Censys adds ~500 TB/year |
| Events Processed | ~5 billion/day | Reference: Censys processed ~5B events/day in 2024 |

---

## 10. Legacy vs. Next-Generation Architecture Comparison

| Dimension | Legacy Scanner Architecture | Next-Generation Architecture (This System) |
|---|---|---|
| Core Model | Monolithic application / Periodic batch sweeps | Distributed microservices / Continuous streaming pipeline |
| Scalability | Vertical scale-up; single-node bottlenecks | Horizontal scale-out across global PoP networks |
| Data Freshness | Hours to days (batch index updates) | Seconds to minutes (continuous Flink streaming) |
| Ingestion Pipeline | Batch File Transfers / Traditional ETL | High-throughput durable message backbone (Kafka/Redpanda) |
| Processing Paradigm | Batch MapReduce / Offline Scripts | Stateful real-time stream processing (Apache Flink) |
| Storage Architecture | Relational DB or static Search Index | Open Lakehouse (Iceberg on S3) with ACID & Time Travel |
| Targeting Model | Sequential / Random brute-force sweeps | Predictive co-occurrence (GPS) + RL traversal (6Hit/6Sense) |
| Data Quality Policy | Passive retention; accumulated stale hosts | Delta journaling with 72-hour staleness eviction |
| Protocol Detection | Port-number based assumptions | Payload-based heuristic reclassification (LZR: <3% HTTP on port 80) |
| IPv6 Support | Not supported / Manual target lists | Automated RL-based target generation from BGP seeds |
| TLS Fingerprinting | Basic JA3 (evasion-prone) | JA4/JA4+ (resistant to extension permutation) |
| Geolocation | MaxMind database lookup only | 3-tier: Cryptographic -> RTT triangulation -> Passive fallback |
| Ethics Controls | Afterthought / Manual compliance | Embedded in socket primitives; cannot be bypassed |
| Visibility | Single vantage point | Multi-PoP deployment across 3+ continents |
| Service Coverage | 10–100 ports per host | All 65,535 ports via predictive models |

---

# PART III: SOFTWARE PROJECT BLUEPRINT & IMPLEMENTATION PLAN

---

## 11. Guiding Implementation Principles

Development is governed by five operational imperatives:

1. **Safety and scope precede active scanning:** A controlled lab environment and safety abstractions must be established before touching public networks. Phase 0 VPC lab is mandatory.

2. **Data streaming and storage spine is constructed first:** The streaming and storage infrastructure is built and validated with synthetic data prior to prober integration. This prevents expensive operational rebuilds later.

3. **Heuristic targeting precedes reinforcement learning:** Phase 3a heuristic targeting establishes reliable operational baselines before the RL model is deployed. The RL agent requires policy validation against baseline heuristics before promotion.

4. **Modular interfaces decouple components:** Storage backends, interrogators, and targeting algorithms are decoupled through well-defined interfaces, enabling independent development, testing, and replacement.

5. **Zero legacy dependencies:** No runtime or compile-time dependencies on ZMap, Nmap, Masscan, or other legacy scanning tools. All networking logic is crafted natively in Rust/Go, drawing on the underlying research as conceptual prior art.

---

## 12. Master Phased Roadmap

### Phase 0: Scope & Safety (Week 0)

**Objective:** Establish a fully controlled environment with safety controls embedded at the lowest level.

**Deliverables:**
- Virtual private cloud (VPC) laboratory environment containing 10–20 isolated virtual machines
- Lab VMs running misconfigured services: SSH (multiple versions), Redis (with/without auth), HTTP (Apache/Nginx), FTP, MySQL, MongoDB, gRPC endpoints, a local vLLM endpoint for LLM fingerprinting
- Local development Kubernetes cluster using Minikube or kind
- Lab-scale database alternatives (Postgres or SQLite) for initial schema validation before Iceberg deployment
- Basic search interface (Meilisearch or Elasticsearch instance) for development iteration
- User-agent identity injection compiled directly into Rust socket abstractions
- Token-bucket rate limiters compiled into core networking code
- Opt-out drop list enforcement validated at the prober interface level
- Verification that safety mechanisms cannot be bypassed by upper-level targeting logic
- Written scanning policy document with legal review

**Definition of Done:**
- The isolated test VPC is operational
- Safety controls are compiled directly into socket abstractions
- Opt-out drop lists are validated in testing
- All probes carry identification strings in HTTP User-Agent and TLS extensions

### Phase 1: The Data Foundation (Weeks 1–3)

**Objective:** Deploy and validate core data infrastructure with synthetic data.

**Deliverables:**
- MinIO object storage cluster configured with S3-compatible API
- Apache Kafka (or Redpanda) message broker cluster with topic configuration
- Apache Flink stream processing cluster with checkpointing configured
- Apache Polaris REST catalog connected to Iceberg tables
- Synthetic data generator streaming mock scan events through Kafka -> Flink -> Iceberg
- Trino query engine executing point-in-time SQL queries over historic Iceberg snapshots
- Standalone Rust modules for:
  - Stateless SYN scanning (pnet-based raw socket implementation)
  - TLS Client Hello parsing and JA3/JA4 computation
  - SSH version string reading
  - RPKI geofeed parsing (rpki-rs integration)

**Definition of Done:**
- Synthetic events flow through Kafka and Flink into Iceberg tables on MinIO
- Trino executes `SELECT * FROM hosts FOR SYSTEM_TIME AS OF '<past_timestamp>'` queries
- All Rust modules compile and pass unit tests against synthetic data
- Flink checkpoints complete within configured interval

### Phase 2: Core Probing Engine (Weeks 4–7)

**Objective:** Build and integrate the two-phase scanning engine.

**Deliverables:**
- Complete Rust Phase 1 stateless SYN scanner using secret-keyed sequence number encoding (SipHash-2-4) to avoid per-connection memory allocation
- Cyclic multiplicative group permutation for address traversal randomization
- Pipelined send/recv architecture with 16-shard parallelism per address family
- Stateful Phase 2 interrogators:
  - HTTP/S interrogator with JA4 fingerprinting and WAF evasion (browser fingerprint spoofing)
  - SSH behavioral fingerprinting (key-exchange preferences, host key types, response patterns)
  - gRPC ALPN negotiation and HTTP/2 Connection Preface
  - QUIC Initial packet construction and transport parameter parsing
- LZR heuristic deployed in Apache Flink for port-independent protocol reclassification
- Kafka topic integration: scanning agents publish directly to raw-l4-scans and l7-banners topics

**Definition of Done:**
- Stateless L4 SYN probers achieve >=1M probes/sec in lab environment
- Stateful L7 interrogators pass raw results to Kafka with correct schema
- LZR correctly reclassifies obfuscated lab services (e.g., HTTP on port 8443, SSH on port 2222)
- End-to-end pipeline: probe -> Kafka -> Flink -> Iceberg -> Trino query

### Phase 3: Intelligent Targeting & Passive Aggregation (Weeks 8–11)

**Objective:** Implement predictive targeting and passive intelligence integration.

**Deliverables:**
- Passive Intelligence Aggregator processing:
  - Local BGP dump files for prefix extraction
  - Certificate Transparency log feeds (letstify/crt.sh integration)
  - Passive DNS record correlation
- Phase 3a Heuristic Targeting Service:
  - GPS-derived co-occurrence probability computation
  - Conditional probability matrix for port/ASN/protocol correlations
  - Target prioritization algorithm maximizing yield per probe
- Phase 3b Reinforcement Learning Targeting Model:
  - PyTorch DQN implementation for IPv6 address space exploration
  - State space encoding: subnet prefix features, historical hit rates, BGP path properties
  - Action space: /64 or /120 subrange selection
  - Reward function: positive for live hosts, negative for barren/alias subnets
  - FastAPI microservice serving predictions with <100ms latency
  - Ray-based distributed training pipeline

**Definition of Done:**
- Passive intelligence streams seed targeting queues
- Heuristic targeting outperforms random sweeps by >=2x hit rate in lab
- RL targeting service serves low-latency predictions via REST
- RL agent demonstrates improvement over heuristic baseline after training

### Phase 4: Geolocation, Enrichment & API (Weeks 12–15)

**Objective:** Implement intelligence enrichment and serving layer.

**Deliverables:**
- Multi-tier geolocation engine:
  - Tier 1: rpki-rs integration for cryptographic geofeed validation
  - Tier 2: Multi-vantage-point RTT triangulation across lab nodes (CBG implementation)
  - Tier 3: WHOIS/RDAP and BGP AS location fallback
- Vulnerability correlation pipeline:
  - CPE string extraction from banners and certificates
  - Local NVD CVE cache with scheduled updates
  - CPE-to-CVE matching with CVSS v3.1 severity scoring
- Asset relationship graphing:
  - TLS certificate serial number correlation
  - SSH host key matching
  - JARM signature clustering
- Rust API Gateway (Axum):
  - RESTful endpoints modeled on Shodan/Censys API structures
  - WebSocket streaming for real-time data
  - API token authentication with RBAC
  - Rate limiting with token-bucket algorithm
  - Trino/Iceberg query routing

**Definition of Done:**
- RPKI geofeed verification passes via rpki-rs
- RTT triangulation resolves lab node locations within expected accuracy
- CVEs enriched from NVD data and attached to service records
- API Gateway serves JSON queries with <200ms latency
- Time-travel queries accessible via API

### Phase 5: Hardening, MLOps, and Advanced Fingerprinting (Weeks 16–20)

**Objective:** Advanced fingerprinting, MLOps automation, and security hardening.

**Deliverables:**
- LLM Refusal Vector interrogator module:
  - Three standardized refusal-inducing prompts
  - Cosine similarity computation over response tokens
  - Model-family behavioral signature generation
  - Validation against local Ollama/vLLM endpoints
- Automated MLOps pipeline:
  - Ray/PyTorch continuous retraining on incoming stream feedback
  - Model versioning and A/B testing framework
  - Performance monitoring and drift detection
- WAF evasion testing:
  - JA4 browser fingerprint spoofing validation
  - Dynamic TLS Client Hello generation matching Chrome/Firefox profiles
  - Testing against lab Cloudflare/similar WAF instances
- Security hardening:
  - Complete security code audit
  - Fuzzing of protocol parsers with malformed inputs
  - Memory safety validation under adversarial conditions
  - Operational ethics documentation publication

**Definition of Done:**
- LLM refusal-vector interrogators accurately classify local model families
- WAF evasion tests pass in lab environment
- MLOps retrains targeting models unattended
- Security code audit complete with no critical findings
- Ethics documentation published

### Phase 6: Distributed Scaling & Global Deployment (Post-Week 20)

**Objective:** Production deployment across global infrastructure.

**Deliverables:**
- All microservices containerized with multi-stage Docker builds
- Kubernetes Helm charts for each component
- Multi-region cloud deployment (minimum 3 PoPs)
- Distributed scanning agents across geographically diverse Points of Presence
- Monitoring and alerting: Prometheus metrics, Grafana dashboards, PagerDuty integration
- Formal legal authorization reviews completed
- Incident response procedures documented
- Abuse handling infrastructure (dedicated email, web page, reverse DNS)

**Definition of Done:**
- System operates continuously across Kubernetes clusters in global PoPs
- Global scheduling orchestration distributes scan load
- Monitoring dashboards operational
- Formal legal sign-off obtained for production scanning
- Abuse response infrastructure tested and documented

---

## 13. Master Risk Register & Critical Challenges Matrix

| Risk Event | Severity | Risk Description | Engineering Mitigation Strategy |
|---|---|---|---|
| RL Target Instability | High | Reinforcement learning model generates sub-optimal or stuck probe paths during early training phases. | Phase 3a heuristic targeting runs as a fallback; RL agent requires policy validation against baseline heuristics before promotion. Implement early stopping and reward threshold guards. |
| Low Geofeed Adoption | Medium | Cryptographic geofeeds are published by <0.30% of global subnets, creating coverage gaps. | Implement mandatory automated fallback to active RTT triangulation and passive WHOIS registration tiers. Monitor adoption rates and adjust tier priorities dynamically. |
| Iceberg Snapshot Proliferation | Medium | High-frequency streaming commits generate millions of small files, degrading query planning speed. | Schedule automated Flink/Iceberg maintenance jobs executing snapshot expiration and file compaction every 24 hours. Use write.target-file-size-bytes = 128MB+ and write.distribution-mode = hash. |
| Flink Checkpoint Latency | Medium | Tighter checkpoint intervals lower data freshness latency but increase object storage write overhead. | Tune checkpoint intervals dynamically between 1 and 5 minutes based on live incoming packet velocity. Monitor checkpoint duration; if it exceeds 50% of interval, increase interval or add parallelism. |
| Legal & DoS Exposure | Critical | Internet-wide probes trigger abuse complaints or violate jurisdictional regulations (e.g., CFAA). | Enforce Phase 0 safety controls, inject user-agent contact URIs, maintain automated opt-out drop lists, enforce strict local rate limits. Begin only in lab environments; obtain formal legal clearance before public scanning. |
| WAF / CDN Blocking | High | CDNs (e.g., Cloudflare) drop active probes due to anomalous TLS/HTTP signatures, producing false negatives. | Implement dynamic JA4 browser fingerprint spoofing with rotating Chrome/Firefox profiles. Deploy scanning agents across diverse, non-attributed IP ranges. Monitor block rates and adapt fingerprint selection. |
| Kafka/Redpanda Topic Overflow | Medium | Burst scanning rates overwhelm message backbone capacity, causing consumer lag and data loss. | Implement backpressure mechanisms between scanning agents and Kafka. Use topic partitioning to parallelize consumption. Configure retention policies aligned with processing capacity. |
| Memory Exhaustion in Flink | High | Large state accumulation in Flink operators causes OOM conditions. | Implement state TTL policies. Use RocksDB state backend for large state. Configure state backend with incremental checkpoints. Monitor state size and adjust parallelism. |
| Cryptographic Validation Failure | Medium | rpki-rs validation fails due to expired ROAs or inconsistent RPKI repository data. | Implement graceful degradation: if RPKI validation fails, fall back to unsigned geofeed data with reduced confidence score. Cache validation results to avoid repeated failures. |
| LLM Endpoint Fingerprint Evasion | Low | Adversaries modify LLM refusal behavior to defeat behavioral fingerprinting. | Maintain diverse prompt sets and evolve prompts based on observed evasion patterns. Combine refusal analysis with other fingerprinting methods (JA4, banner analysis). |
| Storage-Pattern Lock-In | Medium | Choosing Cassandra+Elasticsearch (Pattern B) forecloses Iceberg's native time-travel/ACID guarantees unless built by hand. | Commit to Pattern A (Iceberg lakehouse) as primary system of record from Phase 1. If full-text search is required, deploy Elasticsearch as a secondary serving index fed from Iceberg CDC streams, not as a replacement. |
| Operational Cost of Global PoPs | Medium | Maintaining a global network of Points of Presence and computational resources for data processing is substantial, creating a significant cost barrier. | Treat Phase 6 as explicitly optional/scalable-down; the system is fully useful and internally coherent as a single-region deployment through Phase 5. Phase 6 global deployment requires dedicated infrastructure investment. |

---

## 14. Definition of Done, By Phase

- **Phase 0 Complete:** The isolated test VPC is operational, safety controls are compiled directly into socket abstractions, and opt-out drop lists are validated in testing. All probes carry identification strings.

- **Phase 1 Complete:** Synthetic events flow through Kafka and Flink into Iceberg tables on MinIO, and Trino executes point-in-time SQL queries over historic snapshots. All standalone Rust modules pass unit tests.

- **Phase 2 Complete:** Stateless L4 SYN probers achieve >=1M probes/sec in lab; stateful L7 interrogators pass raw results to Kafka; LZR correctly reclassifies obfuscated lab services; end-to-end pipeline validated.

- **Phase 3 Complete:** Passive intelligence streams seed targeting queues; heuristic targeting outperforms random sweeps by >=2x; RL targeting service serves low-latency predictions via REST; RL agent improves over heuristic baseline.

- **Phase 4 Complete:** RPKI geofeed verification passes via rpki-rs; RTT triangulation resolves lab node locations; CVEs enriched from NVD data; API Gateway serves JSON queries with <200ms latency; time-travel queries accessible via API.

- **Phase 5 Complete:** LLM refusal-vector interrogators accurately classify local model families; WAF evasion tests pass; MLOps retrains targeting models unattended; security code audit complete; ethics documentation published.

- **Phase 6 Complete:** The system operates continuously across Kubernetes clusters in global PoPs under global scheduling orchestration, subject to formal legal sign-off. Monitoring and abuse response infrastructure operational.

---

# PART IV: STRATEGIC SYNTHESIS & ACTIONABLE CONCLUSIONS

---

## 15. Strategic Imperatives for   Internet Mapping

The competitive and academic landscape of internet-wide scanning has shifted permanently from packet throughput to intelligence quality. Synthesis of the research literature demonstrates that tools relying exclusively on brute-force L4 sweeps miss up to 97% of modern web applications operating on non-standard ports, under encryption, or behind WAF layers. To deliver value in   beyond, an internet mapping platform must execute deep, multi-phase, protocol-aware interrogations while maintaining strict streaming freshness.

Engineering teams building next-generation platforms must fulfill four strategic imperatives:

### 15.1 Adopt Adaptive Search Over Brute Force

Abandon exhaustive IPv6 enumeration in favor of dynamic reinforcement learning agents navigating BGP-seeded structures. The GPS paradigm demonstrates that co-occurrence probabilistic models can find 92.5% of all services across all ports with 131x less bandwidth. For IPv6, RL-based approaches like 6Hit demonstrate 7.7%–630% improvement over static target generation methods. The future of internet scanning is intelligent, adaptive search—not exhaustive enumeration.

### 15.2 Institutionalize Streaming Data Lakehouses

Replace legacy batch ETL pipelines with continuous Kafka-Flink-Iceberg architectures that offer point-in-time forensic auditing. The streaming lakehouse model provides ACID transactions, snapshot isolation, schema evolution, and native time-travel queries—all essential for maintaining an accurate, auditable map of internet infrastructure. Every Flink checkpoint creates an immutable snapshot, enabling forensic analysis of when specific services appeared or disappeared.

### 15.3 Guarantee Verifiable Data Integrity

Ground platform findings in cryptographic facts—utilizing RPKI-validated geofeeds and enforcing aggressive 72-hour staleness eviction to maintain index live-accuracy above 90%. The three-tier geolocation fallback ensures that every host record carries a verifiable confidence score. Delta-encoded event journaling with monotonic sequence numbers prevents data corruption while maintaining complete audit trails.

### 15.4 Master Modern Encrypted Protocols

Expand behavioral interrogation into emerging asset classes, utilizing JA4+ signatures, ALPN compliance, and refusal-vector behavioral fingerprinting to map exposed AI/LLM infrastructure. As encryption becomes ubiquitous, the ability to identify and classify services depends on behavioral analysis rather than payload inspection. The JA4+ suite provides resistance to common evasion techniques, while LLM endpoint fingerprinting opens an entirely new category of internet-visible assets.

---

## 16. Actionable Implementation Recommendations

For engineering teams commencing platform construction, execution must follow three core directives:

### 16.1 Adhere Strictly to Phased Discipline

Do not deploy active internet scanning modules prior to completing the Phase 0 VPC lab environment and Phase 1 data spine. Validating storage schema evolution and Flink stream checkpointing using mock data prevents expensive operational rebuilds later in development. The phased roadmap provides clear definition-of-done criteria for each stage—advance only when criteria are met.

### 16.2 Prioritize Memory-Safe Core Networking

Implement core packet crafting, sequence number encoding, and L7 handshake parsers exclusively in Rust. Memory safety guarantees at compile time eliminate entire classes of remote code execution vulnerabilities when handling untrusted, malicious network responses at scale. The ownership model prevents buffer overflows, double-frees, and use-after-free vulnerabilities without runtime garbage collection overhead.

### 16.3 Embed Ethics & Safety in Socket Primitives

Implement rate limiting, opt-out checking, and identity injection directly within low-level networking abstractions. Ensuring safety mechanics cannot be bypassed by upper-level targeting logic protects the platform and its operators from legal and operational liability. Ethics controls must be a property of the networking layer, not an afterthought applied at the application layer.

---

## Appendix A: Complete Research Citation Index

| Paper | Venue/Year | Contribution to Architecture |
|---|---|---|
| Durumeric, Wustrow, Halderman — ZMap | USENIX Security 2013 | Foundational stateless packet-engine design, cyclic multiplicative group permutation |
| Adrian et al. — Zippier ZMap | WOOT 2014 | 10x speed optimizations for packet generation |
| Durumeric et al. — A Search Engine Backed by Internet-Wide Scanning | CCS 2015 | Original Censys architecture (ZMap->ZGrab->ZTag pipeline) |
| Gueye et al. — Constraint-Based Geolocation of Internet Hosts | IMC 2004 | CBG multilateration algorithm for delay-to-distance conversion |
| Li, Xie, Sun et al. — XMap | ACSAC 2025 | Current IPv6-capable successor to ZMap with modular architecture |
| Izhikevich, Teixeira, Durumeric — LZR | USENIX Security 2021 | Protocol-agnostic reclassification: <3% of HTTP on port 80 |
| Izhikevich, Teixeira, Durumeric — GPS | SIGCOMM 2022 | Predictive scanning: 92.5% of services with 131x less bandwidth |
| Wan et al. — On the Origin of Scanning | IMC 2020 | Multi-vantage-point deployment rationale |
| Durumeric et al. — Ten Years of ZMap | IMC 2024 | Retrospective on scanning ecosystem evolution |
| Hou et al. — 6Hit | INFOCOM 2021 | RL-based IPv6 target generation: 3.5%-11.5% hit rate |
| Hsu et al. — 6Sense | 2024 | State-of-the-art IPv6 scanning referenced by Censys 2025 |
| Song et al. — AddrMiner | USENIX ATC 2022 | Global active IPv6 address discovery |
| Durumeric et al. — Censys: A Map of Internet Hosts and Services | SIGCOMM 2025 | Production CQRS architecture, 5B events/day, 72-hour eviction |
| Du et al. — GeoCET | CollaborateCom 2019 | Constraint-based elliptical trajectory geolocation |
| Wang et al. — NeighborGeo | Computer Networks 2024 | GNN-based IP geolocation with neighbor relationship learning |
| FoxIO — JA4+ Specification | 2023 | TLS/HTTP/SSH/TCP fingerprinting suite |
| RFC 9092 / RFC 9632 | IETF | Geofeed format and RPKI-validated location data |
| RFC 8805 | IETF | Separate geofeed specification |
| Hu et al. — VistaScan | Sensors 2026 | Visibility-aware distributed task allocation for scanning |
| Tencent — AI-Infra-Guard | arXiv 2026 | Open-source AI infrastructure fingerprinting and vulnerability scanning |

## Appendix B: Glossary of Technical Terms

| Term | Definition |
|---|---|
| 6SENSE/6Hit/HMap/XMap | Family of IPv6 scanning research: 6SENSE (passive measurement), 6Hit (RL target generation), HMap (dynamic feedback-driven probing), XMap (IPv6-capable ZMap successor) |
| A2A | Agent-to-Agent protocol; emerging inter-agent communication standard for AI systems |
| ALPN | Application-Layer Protocol Negotiation; TLS extension for protocol selection |
| ASN | Autonomous System Number; unique identifier for a network operator |
| CBG | Constraint-Based Geolocation; delay-to-distance multilateration algorithm |
| CPE | Common Platform Enumeration; standardized naming for hardware/software platforms |
| CT | Certificate Transparency; public log of issued TLS certificates |
| CQRS | Command Query Responsibility Segregation; architectural pattern separating reads and writes |
| CVE | Common Vulnerabilities and Exposures; standardized identifier for security vulnerabilities |
| DQN | Deep Q-Network; reinforcement learning architecture for sequential decision-making |
| gRPC | Google Remote Procedure Call; HTTP/2-based RPC framework |
| Iceberg | Apache Iceberg; open table format for large analytical datasets |
| JA3/JA4 | Jumpshot Authentication 3/4; TLS fingerprinting standards (JA4 replaces JA3) |
| LZR | Lazy Zombie Recognition; protocol-agnostic service detection heuristic |
| MCP | Machine Control Protocol; emerging application-level protocol for AI/LLM infrastructure |
| PoP | Point of Presence; geographically distributed scanning location |
| RPKI | Resource Public Key Infrastructure; cryptographic validation of IP address assignments |
| RL | Reinforcement Learning; machine learning paradigm for sequential decision-making |
| ROA | Route Origin Authorization; RPKI-signed attestation of prefix-to-ASN mapping |
| SRA | Subnet-Router Anycast; IPv6 addressing mechanism for router interfaces |
| ZGrab2 | Censys's open-source Go-based Layer 7 protocol interrogation tool |

## Appendix C: Document Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-08-29 | Initial consolidated master document from 4 source materials (113 pages) |
| 1.1 | 2026-08-29 | Completeness audit: filled 40+ gaps from all 5 source documents including emerging protocols (MCP/A2A), website/RF fingerprinting, DPDK, ZGrab2, pseudo-service detection, geofeed quality failures, Censys data points, legal references, risk register additions, glossary expansion |
| 1.2 | 2026-08-29 | Deep line-by-line audit: added VistaScan visibility-aware scanning, AI-Infra-Guard AI fingerprinting, Cisco Skill Scanner LLM-as-a-judge, Censys Active DNS data, Redis task queue, user-defined interest prioritization, RFC 8805 citation |

---

*This document represents the consolidated architectural reference for the AEGIS platform. All decisions documented herein are grounded in peer-reviewed research and validated against production-scale reference implementations. This is a living document subject to revision as the platform evolves through its phased implementation.*
