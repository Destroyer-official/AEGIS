use aegis_traversal::{CyclicPermutation, DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_perm_next(c: &mut Criterion) {
    let mut group = c.benchmark_group("permutation");
    group.throughput(Throughput::Elements(1));
    group.bench_function("next_u64", |b| {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| {
            black_box(perm.next());
        })
    });
    group.finish();
}

fn bench_perm_next_ipv4(c: &mut Criterion) {
    let mut group = c.benchmark_group("permutation_ipv4");
    group.throughput(Throughput::Elements(1));
    group.bench_function("next_ipv4", |b| {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        b.iter(|| black_box(perm.next_ipv4()))
    });
    group.finish();
}

fn bench_perm_batch_10k(c: &mut Criterion) {
    let mut group = c.benchmark_group("permutation_batch");
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("10k_next_ipv4", |b| {
        b.iter(|| {
            let mut perm =
                CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
            for _ in 0..10_000 {
                black_box(perm.next_ipv4());
            }
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_perm_next,
    bench_perm_next_ipv4,
    bench_perm_batch_10k
);
criterion_main!(benches);
