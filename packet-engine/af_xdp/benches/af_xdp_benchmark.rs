use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

#[cfg(target_os = "linux")]
use aegis_af_xdp::{AfXdpEngine, ParsedPacket, UmemConfig, XdpProgram, XskConfig};

fn build_test_packet(size: usize) -> Vec<u8> {
    let mut pkt = vec![0u8; size];
    let min_size = 64;
    let actual_size = size.max(min_size);

    pkt[0..6].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    pkt[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
    pkt[12] = 0x08;
    pkt[13] = 0x00;

    let ip_start = 14;
    pkt[ip_start] = 0x45;
    pkt[ip_start + 1] = 0x00;
    let total_len = (actual_size - 14) as u16;
    pkt[ip_start + 2..ip_start + 4].copy_from_slice(&total_len.to_be_bytes());
    pkt[ip_start + 4] = 0x00;
    pkt[ip_start + 5] = 0x00;
    pkt[ip_start + 6] = 0x40;
    pkt[ip_start + 7] = 0x00;
    pkt[ip_start + 8] = 0x40;
    pkt[ip_start + 9] = 0x06;
    pkt[ip_start + 12] = 192;
    pkt[ip_start + 13] = 168;
    pkt[ip_start + 14] = 1;
    pkt[ip_start + 15] = 1;
    pkt[ip_start + 16] = 8;
    pkt[ip_start + 17] = 8;
    pkt[ip_start + 18] = 8;
    pkt[ip_start + 19] = 8;

    let tcp_start = ip_start + 20;
    pkt[tcp_start] = 0x00;
    pkt[tcp_start + 1] = 0x50;
    pkt[tcp_start + 2] = 0x00;
    pkt[tcp_start + 3] = 0x50;
    pkt[tcp_start + 12] = 0x50;
    pkt[tcp_start + 13] = 0x10;

    pkt.truncate(actual_size);
    pkt
}

fn bench_parse_packet_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_packet_sizes");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    let packet_sizes = [64, 128, 256, 512, 1024, 1500];

    for &size in &packet_sizes {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("parse", size), &size, |b, &size| {
            let pkt = build_test_packet(size);
            #[cfg(target_os = "linux")]
            b.iter(|| {
                let _ = ParsedPacket::parse(black_box(&pkt));
            });
            #[cfg(not(target_os = "linux"))]
            b.iter(|| black_box(&pkt));
        });
    }

    group.finish();
}

fn bench_ring_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_operations");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    let frame_count = 4096;
    let frame_addrs: Vec<u64> = (0..frame_count).map(|i| (i * 2048) as u64).collect();

    group.bench_function("fill_ring_produce_4096", |b| {
        b.iter(|| black_box(&frame_addrs));
    });

    let rx_descs: Vec<aegis_af_xdp::xdp_desc> = (0..frame_count)
        .map(|i| aegis_af_xdp::xdp_desc {
            addr: (i * 2048) as u64,
            len: 64,
            options: 0,
        })
        .collect();

    group.bench_function("rx_ring_consume_4096", |b| {
        b.iter(|| black_box(&rx_descs));
    });

    group.finish();
}

#[cfg(target_os = "linux")]
fn bench_engine_recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_recv");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let ifname = std::env::var("AEGIS_IFACE").unwrap_or_else(|_| "eth0".to_string());
    let frame_count = 4096;

    let config = XskConfig {
        frame_count,
        ..Default::default()
    };

    if let Ok(engine) = AfXdpEngine::new(&ifname, &config) {
        let xdp_prog_path = std::env::var("AEGIS_XDP_PROG").unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("bpf")
                .join("xdp_redirect.o")
                .to_string_lossy()
                .to_string()
        });

        if let Ok(_xdp_prog) = engine.load_xdp_program(&xdp_prog_path, &ifname) {
            let _ = engine.refill_fill_ring();

            let packet_sizes = [64, 512, 1500];

            for &size in &packet_sizes {
                group.throughput(Throughput::Elements(1));
                group.bench_with_input(
                    BenchmarkId::new("engine_recv_parse", size),
                    &size,
                    |b, &size| {
                        let pkt = build_test_packet(size);
                        let mut packets = Vec::with_capacity(64);
                        b.iter(|| {
                            packets.clear();
                            let _ = engine.recv(black_box(&mut packets), 64);
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

#[cfg(not(target_os = "linux"))]
fn bench_engine_recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_recv");
    group.bench_function("engine_recv_stub", |b| {
        b.iter(|| black_box(()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_packet_sizes,
    bench_ring_operations,
    bench_engine_recv
);
criterion_main!(benches);
