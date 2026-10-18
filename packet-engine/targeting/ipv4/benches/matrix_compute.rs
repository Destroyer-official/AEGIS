use aegis_targeting_ipv4::{Feature, HostObservation, OptimizedMatrix};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Simulate a realistic GPS seed scan dataset.
///
/// In production, this data comes from a 1% LZR scan of IPv4
/// across all 65,535 ports. Here we generate synthetic data
/// that follows realistic co-occurrence patterns.
fn generate_seed_data(n_hosts: usize) -> Vec<HostObservation> {
    let mut rng_state: u64 = 0xDEAD_BEEF_CAFE_1234;
    let mut rng = || -> u64 {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        rng_state
    };

    let cloud_asns = [16509, 14618, 13335, 8075, 32934];
    let cdn_asns = [13335, 20940, 16625, 33182];
    let edu_asns = [2, 11, 17, 33, 47];
    let gov_asns = [721, 112, 16];

    let _common_ports = [22, 80, 443, 25, 53, 21, 3306, 3389];

    (0..n_hosts)
        .map(|i| {
            let r = rng();
            let ip = (r >> 32) as u32;
            let asn_seed = (r & 0xFFFF) as u32;

            let asn = match i % 10 {
                0..=3 => cloud_asns[(r % cloud_asns.len() as u64) as usize],
                4..=5 => cdn_asns[(r % cdn_asns.len() as u64) as usize],
                6 => edu_asns[(r % edu_asns.len() as u64) as usize],
                7 => gov_asns[(r % gov_asns.len() as u64) as usize],
                _ => asn_seed % 65535 + 1,
            };

            // Simulate realistic port co-occurrence:
            // - HTTP servers almost always have HTTPS too
            // - Cloud hosts tend to have more open ports
            // - SSH hosts rarely have SMTP
            let mut open_ports = Vec::new();
            let mut features = Vec::new();

            // ~40% of hosts have HTTP
            if r % 100 < 40 {
                open_ports.push(80);
                features.push(Feature::PortHttp);
                features.push(Feature::AppHttpServer);

                // 90% of HTTP hosts also have HTTPS
                if r % 100 < 90 {
                    open_ports.push(443);
                    features.push(Feature::PortHttps);
                    features.push(Feature::AppTls);
                    features.push(Feature::AppGrpcAlpn);
                }
            }

            // ~25% have SSH
            if r % 100 < 25 {
                open_ports.push(22);
                features.push(Feature::PortSsh);
                features.push(Feature::AppSshBanner);
            }

            // ~10% have SMTP
            if r % 100 < 10 {
                open_ports.push(25);
                features.push(Feature::PortSmtp);
                features.push(Feature::AppSmtpBanner);
            }

            // ~5% have MySQL
            if r % 100 < 5 {
                open_ports.push(3306);
                features.push(Feature::PortMysql);
                features.push(Feature::AppMysqlProtocol);
            }

            // ~8% have DNS
            if r % 100 < 8 {
                open_ports.push(53);
                features.push(Feature::PortDns);
            }

            // ~3% have RDP
            if r % 100 < 3 {
                open_ports.push(3389);
                features.push(Feature::PortRdp);
            }

            // Network features based on ASN
            if cloud_asns.contains(&asn) {
                features.push(Feature::NetCloudProvider);
                features.push(Feature::NetDynIp);
            }
            if cdn_asns.contains(&asn) {
                features.push(Feature::NetCdnProvider);
            }
            if edu_asns.contains(&asn) {
                features.push(Feature::NetEducational);
            }
            if gov_asns.contains(&asn) {
                features.push(Feature::NetGovMilitary);
            }

            // ~20% residential
            if r % 100 < 20 {
                features.push(Feature::NetResidential);
            }

            features.push(Feature::NetAsn);

            HostObservation {
                ip,
                open_ports,
                features,
                asn,
            }
        })
        .collect()
}

fn bench_matrix_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("gps_matrix_build");

    for size in [1000, 10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let data = generate_seed_data(size);
            b.iter(|| {
                let mut matrix = OptimizedMatrix::new();
                matrix.observe_batch_parallel(&data).unwrap();
                matrix.build().unwrap();
                matrix
            });
        });
    }

    group.finish();
}

fn bench_port_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("gps_port_prediction");

    // Build matrix with 100K hosts
    let data = generate_seed_data(100_000);
    let mut matrix = OptimizedMatrix::new();
    matrix.observe_batch_parallel(&data).unwrap();
    matrix.build().unwrap();

    let features_sets = [
        vec![Feature::PortHttp],
        vec![Feature::PortHttp, Feature::AppTls],
        vec![
            Feature::PortSsh,
            Feature::PortHttp,
            Feature::AppTls,
            Feature::NetCloudProvider,
        ],
        vec![
            Feature::AppGrpcAlpn,
            Feature::NetCloudProvider,
            Feature::NetDynIp,
        ],
    ];

    for (i, features) in features_sets.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("predict", format!("features_{i}")),
            features,
            |b, features| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for port in [22, 80, 443, 25, 53, 21, 3306, 3389] {
                        let p = matrix.predict_port(port, features).unwrap();
                        results.push((port, p));
                    }
                    results
                });
            },
        );
    }

    group.finish();
}

fn bench_remaining_services(c: &mut Criterion) {
    let mut group = c.benchmark_group("gps_remaining_services");

    let data = generate_seed_data(100_000);
    let mut matrix = OptimizedMatrix::new();
    matrix.observe_batch_parallel(&data).unwrap();
    matrix.build().unwrap();

    let features = vec![
        Feature::PortHttp,
        Feature::AppTls,
        Feature::NetCloudProvider,
    ];

    group.bench_function("predict_all_above_0.1", |b| {
        b.iter(|| matrix.predict_remaining_services(&features, 0.1).unwrap());
    });

    group.finish();
}

fn bench_first_service(c: &mut Criterion) {
    let mut group = c.benchmark_group("gps_first_service");

    let data = generate_seed_data(100_000);
    let mut matrix = OptimizedMatrix::new();
    matrix.observe_batch_parallel(&data).unwrap();
    matrix.build().unwrap();

    let network_features = vec![
        Feature::NetCloudProvider,
        Feature::NetDynIp,
        Feature::NetAsn,
    ];

    group.bench_function("predict_first_with_network_features", |b| {
        b.iter(|| matrix.predict_first_service(&network_features).unwrap());
    });

    group.bench_function("predict_first_no_features", |b| {
        b.iter(|| matrix.predict_first_service(&[]).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_matrix_build,
    bench_port_prediction,
    bench_remaining_services,
    bench_first_service,
);
criterion_main!(benches);
