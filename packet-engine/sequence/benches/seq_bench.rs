use aegis_sequence::pipeline::{BatchPipeline, ProbePipeline};
use aegis_sequence::simd;
use aegis_sequence::{siphash24, SeqKey};
use aegis_traversal::CyclicPermutation;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::net::Ipv4Addr;

fn bench_siphash24(c: &mut Criterion) {
    let key = [0x42u8; 16];
    let data = b"192.168.1.1:80:1234567890";
    let k0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
    let k1 = u64::from_le_bytes(key[8..16].try_into().unwrap());
    let mut group = c.benchmark_group("siphash24");
    group.throughput(Throughput::Elements(1));
    group.bench_function("siphash24_22bytes", |b| {
        b.iter(|| siphash24(black_box(k0), black_box(k1), black_box(data)))
    });
    group.finish();
}

fn bench_seq_encode_ipv4(c: &mut Criterion) {
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let ip = Ipv4Addr::new(8, 8, 8, 8);
    let port = 443u16;
    let ts = 1234567890u32;
    let mut group = c.benchmark_group("seq_encode");
    group.throughput(Throughput::Elements(1));
    group.bench_function("encode_ipv4", |b| {
        b.iter(|| key.encode_ipv4(black_box(ip), black_box(port), black_box(ts)))
    });
    group.finish();
}

fn bench_seq_throughput_10m(c: &mut Criterion) {
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let _ip = Ipv4Addr::new(192, 168, 1, 1);
    let port = 80u16;
    let ts = 0u32;
    let mut group = c.benchmark_group("seq_throughput");
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("10k_batch", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                let ip = Ipv4Addr::from(0xC0A80001u32.wrapping_add(i));
                black_box(key.encode_ipv4(ip, port, ts));
            }
        })
    });
    group.finish();
}

fn bench_pipeline(c: &mut Criterion) {
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let mut group = c.benchmark_group("pipeline");
    group.throughput(Throughput::Elements(1));
    group.bench_function("next_probe", |b| {
        let perm = CyclicPermutation::new_default(1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 12345);
        b.iter(|| black_box(pipeline.next(443)))
    });
    group.bench_function("next_probe_validate", |b| {
        let perm = CyclicPermutation::new_default(1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 12345);
        b.iter(|| {
            let probe = pipeline.next(443).unwrap();
            black_box(pipeline.validate(&probe, probe.seq.wrapping_add(1)))
        })
    });
    group.finish();
}

fn bench_batch_pipeline(c: &mut Criterion) {
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let mut group = c.benchmark_group("batch_pipeline");
    let _perm = CyclicPermutation::new_default(1).unwrap();
    group.throughput(Throughput::Elements(100));
    group.bench_function("send_batch_100", |b| {
        b.iter_with_setup(
            || {
                let perm = CyclicPermutation::new_default(1).unwrap();
                BatchPipeline::<100>::new(ProbePipeline::new(key, perm, 5000))
            },
            |mut batch| {
                batch.send_batch(443);
                black_box(batch.len())
            },
        )
    });
    group.finish();
}

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let k0 = 0x0706050403020100u64;
    let k1 = 0x0f0e0d0c0b0a0908u64;
    let ips4 = [
        Ipv4Addr::new(1, 1, 1, 1),
        Ipv4Addr::new(8, 8, 8, 8),
        Ipv4Addr::new(192, 168, 1, 1),
        Ipv4Addr::new(10, 0, 0, 1),
    ];
    let ports4 = [80u16, 443, 8080, 9090];
    let ts4 = [1000u32, 2000, 3000, 4000];

    let mut group = c.benchmark_group("simd_vs_scalar");
    group.throughput(Throughput::Elements(4));

    group.bench_function("scalar_4x", |b| {
        b.iter(|| {
            let mut out = [0u32; 4];
            for i in 0..4 {
                let mut buf = [0u8; 10];
                buf[0..4].copy_from_slice(&ips4[i].octets());
                buf[4..6].copy_from_slice(&ports4[i].to_le_bytes());
                buf[6..10].copy_from_slice(&ts4[i].to_le_bytes());
                out[i] = siphash24(black_box(k0), black_box(k1), black_box(&buf)) as u32;
            }
            black_box(out)
        })
    });

    group.bench_function("simd_4x", |b| {
        b.iter(|| black_box(simd::siphash24_4x_ipv4(k0, k1, ips4, ports4, ts4)))
    });

    // 8x batch
    let ips8 = [
        Ipv4Addr::new(1, 0, 0, 1),
        Ipv4Addr::new(1, 0, 0, 2),
        Ipv4Addr::new(1, 0, 0, 3),
        Ipv4Addr::new(1, 0, 0, 4),
        Ipv4Addr::new(1, 0, 0, 5),
        Ipv4Addr::new(1, 0, 0, 6),
        Ipv4Addr::new(1, 0, 0, 7),
        Ipv4Addr::new(1, 0, 0, 8),
    ];
    let ports8 = [80u16; 8];
    let ts8 = [1000u32; 8];

    group.throughput(Throughput::Elements(8));
    group.bench_function("scalar_8x", |b| {
        b.iter(|| {
            let mut out = [0u32; 8];
            for i in 0..8 {
                let mut buf = [0u8; 10];
                buf[0..4].copy_from_slice(&ips8[i].octets());
                buf[4..6].copy_from_slice(&ports8[i].to_le_bytes());
                buf[6..10].copy_from_slice(&ts8[i].to_le_bytes());
                out[i] = siphash24(black_box(k0), black_box(k1), black_box(&buf)) as u32;
            }
            black_box(out)
        })
    });

    group.bench_function("simd_8x", |b| {
        b.iter(|| black_box(simd::siphash24_8x_ipv4(k0, k1, ips8, ports8, ts8)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_siphash24,
    bench_seq_encode_ipv4,
    bench_seq_throughput_10m,
    bench_pipeline,
    bench_batch_pipeline,
    bench_simd_vs_scalar
);
criterion_main!(benches);
