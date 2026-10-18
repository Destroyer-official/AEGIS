//! Cross-crate micro-benchmarks for sequence + traversal crates
//! Target: <50ns per operation for both crates

use aegis_sequence::SeqKey;
use aegis_traversal::{
    CyclicPermutation, PermutationShard, Shard, ShardSet, DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32,
    NUM_SHARDS,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::net::Ipv4Addr;

fn bench_siphash_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("siphash_sequence");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let data = b"192.168.1.1:80:1234567890"; // 22 bytes - typical probe data

    group.bench_function("encode_22bytes", |b| b.iter(|| key.encode(black_box(data))));

    group.bench_function("hash_22bytes", |b| b.iter(|| key.hash(black_box(data))));

    // Test with different data sizes
    for size in [0, 10, 22, 64, 128, 256] {
        let data = vec![0xAAu8; size];
        group.bench_with_input(BenchmarkId::new("encode_var", size), &data, |b, d| {
            b.iter(|| key.encode(black_box(d)))
        });
    }

    group.finish();
}

fn bench_seq_encode_ipv4(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_encode_ipv4");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let ip = Ipv4Addr::new(8, 8, 8, 8);
    let port = 443u16;
    let ts = 1234567890u32;

    group.bench_function("encode_ipv4", |b| {
        b.iter(|| key.encode_ipv4(black_box(ip), black_box(port), black_box(ts)))
    });

    group.bench_function("validate_ipv4", |b| {
        let seq = key.encode_ipv4(ip, port, ts);
        b.iter(|| {
            key.validate_ipv4(
                black_box(ip),
                black_box(port),
                black_box(ts),
                black_box(seq),
            )
        })
    });

    group.bench_function("encode_full_ipv4", |b| {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let src_port = 12345u16;
        b.iter(|| {
            key.encode_full_ipv4(
                black_box(ip),
                black_box(port),
                black_box(src_ip),
                black_box(src_port),
                black_box(ts),
            )
        })
    });

    group.finish();
}

fn bench_traversal_permutation(c: &mut Criterion) {
    let mut group = c.benchmark_group("traversal_permutation");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    group.bench_function("next", |b| {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| black_box(perm.next()))
    });

    group.bench_function("next_ipv4", |b| {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| black_box(perm.next_ipv4()))
    });

    group.bench_function("current", |b| {
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| black_box(perm.current()))
    });

    group.bench_function("steps", |b| {
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| black_box(perm.steps()))
    });

    group.finish();
}

fn bench_traversal_shard(c: &mut Criterion) {
    let mut group = c.benchmark_group("traversal_shard");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    group.bench_function("next_target", |b| {
        let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();
        b.iter(|| black_box(shard.next_target()))
    });

    group.bench_function("send", |b| {
        let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();
        b.iter(|| black_box(shard.send()))
    });

    group.bench_function("recv", |b| {
        let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();
        b.iter(|| black_box(shard.recv()))
    });

    group.finish();
}

fn bench_traversal_shardset(c: &mut Criterion) {
    let mut group = c.benchmark_group("traversal_shardset");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    group.bench_function("send_all", |b| {
        b.iter(|| {
            let mut set = ShardSet::new_default(NUM_SHARDS).unwrap();
            black_box(set.send_all())
        })
    });

    group.bench_function("recv_all", |b| {
        b.iter(|| {
            let mut set = ShardSet::new_default(NUM_SHARDS).unwrap();
            black_box(set.recv_all())
        })
    });

    group.finish();
}

fn bench_combined_seq_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_seq_traversal");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);

    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();
    let port = 443u16;
    let ts = 1234567890u32;

    group.bench_function("probe_encode", |b| {
        b.iter(|| {
            let ip = shard.next_target().unwrap();
            let seq = key.encode_ipv4(Ipv4Addr::from(ip), port, ts);
            black_box((ip, seq))
        })
    });

    group.bench_function("probe_validate", |b| {
        let ip = shard.next_target().unwrap();
        let seq = key.encode_ipv4(Ipv4Addr::from(ip), port, ts);
        b.iter(|| {
            let valid = key.validate_ipv4(Ipv4Addr::from(ip), port, ts, seq);
            black_box(valid)
        })
    });

    group.finish();
}

fn bench_batch_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_throughput");
    group.sample_size(10);

    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let port = 443u16;
    let ts = 1234567890u32;

    for batch_size in [1, 10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(batch_size));

        group.bench_with_input(
            BenchmarkId::new("batch_encode", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut local_shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();
                    for _ in 0..size {
                        let ip = local_shard.next_target().unwrap();
                        let seq = key.encode_ipv4(Ipv4Addr::from(ip), port, ts);
                        black_box((ip, seq));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_siphash_sequence,
    bench_seq_encode_ipv4,
    bench_traversal_permutation,
    bench_traversal_shard,
    bench_traversal_shardset,
    bench_combined_seq_traversal,
    bench_batch_throughput
);
criterion_main!(benches);
