use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_ethernet_ipv4(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_ethernet_ipv4");

    let mut pkt = vec![0u8; 64];
    pkt[12] = 0x08;
    pkt[13] = 0x00;
    pkt[14] = 0x45;
    pkt[15] = 0x00;
    pkt[16] = 0x00;
    pkt[17] = 0x3c;
    pkt[23] = 0x06;
    pkt[26] = 0xc0;
    pkt[27] = 0xa8;
    pkt[28] = 0x01;
    pkt[29] = 0x01;
    pkt[30] = 0xc0;
    pkt[31] = 0xa8;
    pkt[32] = 0x01;
    pkt[33] = 0x02;
    pkt[34] = 0x00;
    pkt[35] = 0x50;
    pkt[36] = 0x00;
    pkt[37] = 0x50;
    pkt[46] = 0x50;
    pkt[47] = 0x00;

    group.bench_function("parse_64byte_tcp", |b| {
        b.iter(|| aegis_af_xdp::ParsedPacket::parse(black_box(&pkt)))
    });

    group.finish();
}

fn bench_parse_ethernet_ipv6(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_ethernet_ipv6");

    let mut pkt = vec![0u8; 78];
    pkt[12] = 0x86;
    pkt[13] = 0xdd;
    pkt[14] = 0x60;
    pkt[19] = 0x06;
    pkt[20] = 0x40;
    pkt[20 + 6] = 0x06;
    pkt[54] = 0x00;
    pkt[55] = 0x50;
    pkt[56] = 0x00;
    pkt[57] = 0x50;

    group.bench_function("parse_78byte_tcp_v6", |b| {
        b.iter(|| aegis_af_xdp::ParsedPacket::parse(black_box(&pkt)))
    });

    group.finish();
}

fn bench_xdp_ring_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("xdp_ring_throughput");

    group.bench_function("fill_ring_produce_1024", |b| {
        let frames: Vec<u64> = (0..1024).map(|i| i as u64 * 2048).collect();
        b.iter(|| {
            black_box(&frames);
        })
    });

    group.bench_function("rx_ring_consume_1024", |b| {
        b.iter(|| {
            let descs: Vec<aegis_af_xdp::xdp_desc> = (0..1024)
                .map(|i| aegis_af_xdp::xdp_desc {
                    addr: i as u64 * 2048,
                    len: 64,
                    options: 0,
                })
                .collect();
            black_box(&descs);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_ethernet_ipv4,
    bench_parse_ethernet_ipv6,
    bench_xdp_ring_throughput,
);
criterion_main!(benches);
