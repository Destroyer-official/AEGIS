# AEGIS AF_XDP Zero-Copy Packet Engine

Zero-copy RX/TX path using Linux AF_XDP sockets for the AEGIS scanning platform.
Target: >= 1M pps single core (§9 L4 Probe Velocity).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Kernel BPF Program                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  xdp_redirect_prog                                  │    │
│  │  - Parse Ethernet/IP/TCP/UDP                        │    │
│  │  - Extract src/dst IP, ports, TCP flags             │    │
│  │  - Emit perf events for monitoring                  │    │
│  │  - Redirect via bpf_redirect_map() → XSKMAP        │    │
│  └─────────────────────────────────────────────────────┘    │
│                            │                                 │
│                            ▼                                 │
│                    ┌──────────────┐                          │
│                    │   XSKMAP     │                          │
│                    │  (per-queue) │                          │
│                    └──────┬───────┘                          │
│                           │                                  │
└───────────────────────────┼──────────────────────────────────┘
                            │
                ┌───────────┴───────────┐
                │   AF_XDP Socket       │
                │   (setsockopt UMEM)   │
                │                       │
                │  ┌─────┐  ┌─────┐    │
                │  │ FILL│  │ RX  │    │
                │  │ RING│  │ RING│    │
                │  └──┬──┘  └──┬──┘    │
                │     │        │        │
                │  ┌──┴──┐  ┌──┴──┐    │
                │  │COMP │  │ TX  │    │
                │  │RING │  │ RING│    │
                │  └─────┘  └─────┘    │
                └───────────────────────┘
                            │
                ┌───────────┴───────────┐
                │   UMEM Region         │
                │   (4096 frames × 2KB) │
                │   = 8MB zero-copy     │
                └───────────────────────┘
```

## Requirements

- Linux kernel 5.10+ (AF_XDP zero-copy support)
- `CAP_BPF` or `CAP_NET_ADMIN` capability
- libbpf development headers
- clang/LLVM for BPF program compilation

```bash
# Debian/Ubuntu
sudo apt install libbpf-dev clang llvm linux-headers-$(uname -r)

# RHEL/Fedora
sudo dnf install libbpf-devel clang llvm kernel-devel
```

## Building

```bash
cd packet-engine/af_xdp

# Compile BPF program + Rust crate
make

# Or separately:
make bpf          # compile XDP program
cargo build --release  # compile Rust crate
```

## Running

```bash
# Load XDP program and start engine
AEGIS_IFACE=eth0 make run

# Or manually:
sudo ip link set dev eth0 xdp obj bpf/xdp_redirect.o sec xdp
AEGIS_IFACE=eth0 cargo run --release

# Detach XDP program
make detach IFACE=eth0
```

## Benchmarking

```bash
make bench
```

### Target Performance (§9 L4 Probe Velocity)

| Metric | Target | Backend |
|--------|--------|---------|
| Single-core throughput | >= 1M pps | AF_XDP |
| Zero-copy latency | < 1µs per packet | AF_XDP |
| Fill ring batch size | 2048 frames | FILL_RING |
| RX batch size | 64 frames/batch | RX_RING |

### Expected perf output

```
parse_ethernet_ipv4/parse_64byte_tcp
                        time:   [45 ns 46 ns 47 ns]
parse_ethernet_ipv6/parse_78byte_tcp_v6
                        time:   [52 ns 53 ns 54 ns]
xdp_ring_throughput/fill_ring_produce_1024
                        time:   [1.2 µs 1.3 µs 1.4 µs]
xdp_ring_throughput/rx_ring_consume_1024
                        time:   [1.8 µs 1.9 µs 2.0 µs]
```

## Module Structure

```
src/
├── lib.rs          # AfXdpEngine: new(), recv(), send(), poll(), refill()
├── main.rs         # Example binary: recv loop with live pps counter
├── umem.rs         # UMEM: zero-copy memory region, free-stack allocator
├── rings.rs        # FILL/COMPLETION/TX/RX ring buffers (lock-free)
├── socket.rs       # AF_XDP socket: setsockopt UMEM, mmap rings, bind
├── parse.rs        # Ethernet/IPv4/IPv6/TCP/UDP parser (~46ns/pkt)
└── xdp_loader.rs   # BPF program loader, auto-detach on Drop

bpf/
└── xdp_redirect.c  # Kernel XDP program: parse + redirect to XSKMAP

Makefile             # BPF compilation, attach/detach, bench, run
```

## API

```rust
use aegis_af_xdp::{AfXdpEngine, XskConfig, ParsedPacket};

let config = XskConfig {
    frame_count: 4096,
    ..Default::default()
};

let mut engine = AfXdpEngine::new("eth0", &config)?;

// Refill fill ring with free UMEM frames
engine.refill_fill_ring_batch(2048)?;

// Poll for RX readiness
if engine.poll_rx(100)? {
    // Receive and parse packets
    let mut packets = Vec::new();
    let count = engine.recv(&mut packets, 64)?;

    for pkt in &packets {
        println!("{}:{} -> {}:{} ({:?})",
            pkt.src_ip, pkt.src_port,
            pkt.dst_ip, pkt.dst_port,
            pkt.protocol);
    }
}

// Drain completion ring to reclaim frames
engine.completion_drain()?;
```

## Data Flow

```
1. Kernel XDP program parses Ethernet/IP/TCP/UDP headers
2. bpf_redirect_map() sends packet to AF_XDP socket via XSKMAP
3. RX ring descriptor written (addr=UMEM_frame_offset, len=pkt_len)
4. Userspace poll(POLLIN) detects RX readiness
5. consume_batch() reads RX descriptors from ring
6. ParsedPacket::parse() reads frame data from UMEM (zero-copy)
7. Frame addresses recycled back to FILL ring for reuse
8. completion_drain() reclaims TX-completed frames
```

## Safety & Ethics

Per §3.9 of the AEGIS Master Document:
- Rate limiting enforced at the socket abstraction layer
- Opt-out lists checked before probe dispatch
- All probes carry AEGIS identification strings
- XDP program only redirects -- no forwarding to network stack
