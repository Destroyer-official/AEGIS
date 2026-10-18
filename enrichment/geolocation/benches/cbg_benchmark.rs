use aegis_geolocation::{haversine_km, GeoLocation, Landmark, RttMeasurement, RttTriangulator};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

/// Ground truth locations for benchmarking
/// These are real cities with known coordinates
fn benchmark_targets() -> Vec<(&'static str, f64, f64)> {
    vec![
        ("Amsterdam", 52.3676, 4.9041),
        ("Frankfurt", 50.1109, 8.6821),
        ("London", 51.5074, -0.1278),
        ("Paris", 48.8566, 2.3522),
        ("Madrid", 40.4168, -3.7038),
        ("Rome", 41.9028, 12.4964),
        ("Berlin", 52.5200, 13.4050),
        ("Vienna", 48.2082, 16.3738),
        ("Warsaw", 52.2297, 21.0122),
        ("Prague", 50.0755, 14.4378),
    ]
}

/// Landmarks distributed across Europe for triangulation
fn benchmark_landmarks() -> Vec<Landmark> {
    vec![
        Landmark {
            label: "AMS-1".into(),
            latitude: 52.3676,
            longitude: 4.9041,
            probe_addr: IpAddr::from_str("145.40.64.1").unwrap(),
        },
        Landmark {
            label: "FRA-1".into(),
            latitude: 50.1109,
            longitude: 8.6821,
            probe_addr: IpAddr::from_str("145.40.64.2").unwrap(),
        },
        Landmark {
            label: "LHR-1".into(),
            latitude: 51.5074,
            longitude: -0.1278,
            probe_addr: IpAddr::from_str("145.40.64.3").unwrap(),
        },
        Landmark {
            label: "CDG-1".into(),
            latitude: 48.8566,
            longitude: 2.3522,
            probe_addr: IpAddr::from_str("145.40.64.4").unwrap(),
        },
        Landmark {
            label: "MAD-1".into(),
            latitude: 40.4168,
            longitude: -3.7038,
            probe_addr: IpAddr::from_str("145.40.64.5").unwrap(),
        },
        Landmark {
            label: "FCO-1".into(),
            latitude: 41.9028,
            longitude: 12.4964,
            probe_addr: IpAddr::from_str("145.40.64.6").unwrap(),
        },
        Landmark {
            label: "BER-1".into(),
            latitude: 52.5200,
            longitude: 13.4050,
            probe_addr: IpAddr::from_str("145.40.64.7").unwrap(),
        },
        Landmark {
            label: "VIE-1".into(),
            latitude: 48.2082,
            longitude: 16.3738,
            probe_addr: IpAddr::from_str("145.40.64.8").unwrap(),
        },
        Landmark {
            label: "WAW-1".into(),
            latitude: 52.2297,
            longitude: 21.0122,
            probe_addr: IpAddr::from_str("145.40.64.9").unwrap(),
        },
        Landmark {
            label: "PRG-1".into(),
            latitude: 50.0755,
            longitude: 14.4378,
            probe_addr: IpAddr::from_str("145.40.64.10").unwrap(),
        },
    ]
}

/// Simulate RTT measurements based on physical distance + noise
/// Returns (measurements, true_location)
fn simulate_measurements(
    target_lat: f64,
    target_lon: f64,
    landmarks: &[Landmark],
    noise_factor: f64,
) -> Vec<RttMeasurement> {
    let mut measurements = Vec::new();
    let speed_of_light_km_ms = 299_792.458;
    let fiber_factor = 2.0 / 3.0;

    for landmark in landmarks {
        let geo_dist = haversine_km(
            landmark.latitude,
            landmark.longitude,
            target_lat,
            target_lon,
        );

        // One-way latency in ms
        let base_rtt = (geo_dist / (speed_of_light_km_ms * fiber_factor)) * 2.0;

        // Add routing inefficiency (1.2x - 2.0x typical)
        let routing_factor = 1.2 + (noise_factor * 0.8);

        // Add measurement noise (±10%)
        let noise = 1.0 + (noise_factor - 0.5) * 0.2;

        let measured_rtt = base_rtt * routing_factor * noise;

        measurements.push(RttMeasurement {
            landmark: landmark.clone(),
            min_rtt_ms: measured_rtt.max(0.5), // Minimum 0.5ms
            probe_count: 10,
        });
    }

    measurements
}

/// Run CBG triangulation on simulated measurements
fn run_cbg(landmarks: &[Landmark], measurements: &[RttMeasurement]) -> Option<GeoLocation> {
    let triangulator = RttTriangulator::new(landmarks.to_vec());

    // Step 3: Self-calibrate
    let bestlines = triangulator.calibrate_bestlines(measurements);

    // Steps 4-6: Multilaterate
    triangulator.multilaterate(measurements, &bestlines)
}

fn bench_cbg_accuracy(c: &mut Criterion) {
    let landmarks = benchmark_landmarks();
    let targets = benchmark_targets();

    let mut group = c.benchmark_group("cbg_accuracy");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    for (name, target_lat, target_lon) in &targets {
        group.bench_with_input(BenchmarkId::new("median_error", name), name, |b, _| {
            b.iter(|| {
                let mut errors = Vec::new();

                // Run multiple times with different noise to get distribution
                for seed in 0..20 {
                    let noise = (seed as f64 * 0.05) % 1.0;
                    let measurements =
                        simulate_measurements(*target_lat, *target_lon, &landmarks, noise);

                    if let Some(result) = run_cbg(&landmarks, &measurements) {
                        let error_km = haversine_km(
                            result.latitude,
                            result.longitude,
                            *target_lat,
                            *target_lon,
                        );
                        errors.push(error_km);
                    }
                }

                // Calculate median error
                errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let median = errors[errors.len() / 2];

                black_box(median)
            });
        });
    }

    group.finish();
}

fn bench_cbg_optimization_comparison(c: &mut Criterion) {
    let landmarks = benchmark_landmarks();
    let targets = benchmark_targets();

    let mut group = c.benchmark_group("cbg_optimization");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(30);

    // Test with different noise levels to simulate real-world conditions
    for noise_level in [0.1, 0.3, 0.5, 0.7, 0.9] {
        group.bench_with_input(
            BenchmarkId::new(
                "current_implementation",
                format!("noise_{:.1}", noise_level),
            ),
            &noise_level,
            |b, &noise| {
                b.iter(|| {
                    let mut all_errors = Vec::new();

                    for (_, target_lat, target_lon) in &targets {
                        let measurements =
                            simulate_measurements(*target_lat, *target_lon, &landmarks, noise);

                        if let Some(result) = run_cbg(&landmarks, &measurements) {
                            let error_km = haversine_km(
                                result.latitude,
                                result.longitude,
                                *target_lat,
                                *target_lon,
                            );
                            all_errors.push(error_km);
                        }
                    }

                    all_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = all_errors[all_errors.len() / 2];

                    black_box(median)
                });
            },
        );
    }

    group.finish();
}

fn bench_cbg_landmark_count(c: &mut Criterion) {
    let all_landmarks = benchmark_landmarks();
    let targets = benchmark_targets();
    let noise = 0.5;

    let mut group = c.benchmark_group("cbg_landmark_count");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(30);

    for count in [3, 4, 5, 6, 7, 8, 9, 10] {
        let landmarks = all_landmarks[..count].to_vec();

        group.bench_with_input(
            BenchmarkId::new("landmarks", count),
            &landmarks,
            |b, landmarks| {
                b.iter(|| {
                    let mut all_errors = Vec::new();

                    for (_, target_lat, target_lon) in &targets {
                        let measurements =
                            simulate_measurements(*target_lat, *target_lon, landmarks, noise);

                        if let Some(result) = run_cbg(landmarks, &measurements) {
                            let error_km = haversine_km(
                                result.latitude,
                                result.longitude,
                                *target_lat,
                                *target_lon,
                            );
                            all_errors.push(error_km);
                        }
                    }

                    all_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = all_errors[all_errors.len() / 2];

                    black_box(median)
                });
            },
        );
    }

    group.finish();
}

fn bench_cbg_outlier_filter(c: &mut Criterion) {
    let landmarks = benchmark_landmarks();
    let targets = benchmark_targets();
    let noise = 0.5;

    let mut group = c.benchmark_group("cbg_outlier_filter");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(30);

    for outlier_threshold in [1.5, 2.0, 2.5, 3.0] {
        group.bench_with_input(
            BenchmarkId::new("threshold", format!("{:.1}x", outlier_threshold)),
            &outlier_threshold,
            |b, &threshold| {
                b.iter(|| {
                    let mut all_errors = Vec::new();

                    for (_, target_lat, target_lon) in &targets {
                        let measurements = simulate_measurements_with_outliers(
                            *target_lat,
                            *target_lon,
                            &landmarks,
                            noise,
                            threshold,
                        );

                        if let Some(result) = run_cbg(&landmarks, &measurements) {
                            let error_km = haversine_km(
                                result.latitude,
                                result.longitude,
                                *target_lat,
                                *target_lon,
                            );
                            all_errors.push(error_km);
                        }
                    }

                    all_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = all_errors[all_errors.len() / 2];

                    black_box(median)
                });
            },
        );
    }

    group.finish();
}

/// Simulate measurements with outlier injection
fn simulate_measurements_with_outliers(
    target_lat: f64,
    target_lon: f64,
    landmarks: &[Landmark],
    noise_factor: f64,
    outlier_threshold: f64,
) -> Vec<RttMeasurement> {
    let mut measurements = Vec::new();
    let speed_of_light_km_ms = 299_792.458;
    let fiber_factor = 2.0 / 3.0;

    for landmark in landmarks {
        let geo_dist = haversine_km(
            landmark.latitude,
            landmark.longitude,
            target_lat,
            target_lon,
        );

        let base_rtt = (geo_dist / (speed_of_light_km_ms * fiber_factor)) * 2.0;
        let routing_factor = 1.2 + (noise_factor * 0.8);
        let noise = 1.0 + (noise_factor - 0.5) * 0.2;

        let mut measured_rtt = base_rtt * routing_factor * noise;

        // Inject outliers for some landmarks (10% chance)
        if fastrand::f64() < 0.1 {
            measured_rtt *= outlier_threshold * 1.5;
        }

        measurements.push(RttMeasurement {
            landmark: landmark.clone(),
            min_rtt_ms: measured_rtt.max(0.5),
            probe_count: 10,
        });
    }

    measurements
}

criterion_group!(
    benches,
    bench_cbg_accuracy,
    bench_cbg_optimization_comparison,
    bench_cbg_landmark_count,
    bench_cbg_outlier_filter
);
criterion_main!(benches);
