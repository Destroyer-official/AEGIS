#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    use aegis_af_xdp::{AfXdpEngine, XskConfig};
    use std::time::{Duration, Instant};

    let iface = std::env::var("AEGIS_IFACE").unwrap_or_else(|_| "eth0".to_string());
    let bpf_path = std::env::var("AEGIS_BPF").unwrap_or_else(|_| "bpf/xdp_redirect.o".to_string());

    eprintln!("AF_XDP Zero-Copy Packet Engine");
    eprintln!("  interface: {}", iface);
    eprintln!("  bpf prog:  {}", bpf_path);

    let config = XskConfig {
        rx_queue: 0,
        tx_queue: 0,
        frame_count: 4096,
        ..Default::default()
    };

    let mut engine = AfXdpEngine::new(&iface, &config)?;

    let xdp_prog = engine.load_xdp_program(&bpf_path, &iface)?;
    eprintln!("  xdp loaded: prog_fd={}", xdp_prog.prog_fd());

    let filled = engine.refill_fill_ring_batch(2048)?;
    eprintln!("  fill ring: {} frames posted", filled);

    eprintln!("entering recv loop (Ctrl+C to exit)");

    let start = Instant::now();
    let mut total_packets: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut last_report = Instant::now();
    let mut packets = Vec::with_capacity(1024);

    loop {
        if last_report.elapsed() >= Duration::from_secs(1) {
            let elapsed = start.elapsed().as_secs_f64();
            let pps = total_packets as f64 / elapsed;
            let bps = (total_bytes as f64 * 8.0) / elapsed;

            eprintln!(
                "[{:6.1}s] pkts={} rate={:.0} pps ({:.2} Mpps) throughput={:.2} Gbps",
                elapsed,
                total_packets,
                pps,
                pps / 1_000_000.0,
                bps / 1_000_000_000.0,
            );
            last_report = Instant::now();
        }

        if !engine.poll_rx(100)? {
            continue;
        }

        packets.clear();
        let count = engine.recv(&mut packets, 64)?;

        if count == 0 {
            continue;
        }

        total_packets += count as u64;

        for pkt in &packets {
            total_bytes += pkt.raw_len as u64;
        }

        if !packets.is_empty() {
            let pkt = &packets[0];
            eprintln!(
                "  sample: {}:{} -> {}:{} {:?} flags={:?}",
                pkt.src_ip, pkt.src_port, pkt.dst_ip, pkt.dst_port, pkt.protocol, pkt.tcp_flags,
            );
        }

        let free = engine.free_frame_count();
        if free < 1024 {
            let _ = engine.refill_fill_ring_batch(2048 - free as u32)?;
        }

        let _ = engine.completion_drain()?;
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("AF_XDP is only supported on Linux (kernel 5.10+)");
    eprintln!("This binary must be compiled and run on a Linux host.");
    eprintln!();
    eprintln!("Build the BPF program on Linux:");
    eprintln!("  make bpf");
    eprintln!();
    eprintln!("Run on Linux:");
    eprintln!("  AEGIS_IFACE=eth0 cargo run --release");
    std::process::exit(1);
}
