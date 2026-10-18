use aegis_targeting_passive::replay::{HistoricalReplayEngine, ReplayConfig};
use aegis_targeting_passive::{
    BgpDumpParser, PassiveDnsAggregator, PassiveDnsRecord, parse_ct_csv,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_bgp_csv_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("bgp_csv_parse");

    for size in [100, 1_000, 10_000, 100_000] {
        // Generate synthetic BGP CSV data
        let mut csv_data = String::from("prefix,asn,as_path,origin_asn\n");
        for i in 0..size {
            let octets = [(i >> 24) as u8, (i >> 16) as u8, (i >> 8) as u8, i as u8];
            let asn = (i % 65535) + 1;
            csv_data.push_str(&format!(
                "{}.{}.{}.0/24,{},{},{}\n",
                octets[0],
                octets[1],
                octets[2],
                asn,
                asn,
                (asn + 1) % 65535 + 1
            ));
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &csv_data, |b, data| {
            b.iter(|| BgpDumpParser::parse_csv_str(data).unwrap());
        });
    }

    group.finish();
}

fn bench_ct_csv_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("ct_csv_parse");

    for size in [100, 1_000, 10_000] {
        let mut csv_data = String::from("issuer_ca_id,issuer_name,common_name,name_value\n");
        for i in 0..size {
            csv_data.push_str(&format!(
                r#"12345,"Let's Encrypt",example{i}.com,"example{i}.com\nwww.example{i}.com""#
            ));
            csv_data.push('\n');
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &csv_data, |b, data| {
            b.iter(|| parse_ct_csv(data.as_bytes()).unwrap());
        });
    }

    group.finish();
}

fn bench_passive_dns_ingest(c: &mut Criterion) {
    let mut group = c.benchmark_group("passive_dns_ingest");

    for size in [1_000, 10_000, 100_000] {
        let records: Vec<PassiveDnsRecord> = (0..size)
            .map(|i| PassiveDnsRecord {
                domain: format!("host{i}.example.com"),
                ip_address: format!(
                    "{}.{}.{}.{}",
                    (i >> 24) & 0xFF,
                    (i >> 16) & 0xFF,
                    (i >> 8) & 0xFF,
                    i & 0xFF
                ),
                record_type: "A".to_string(),
                first_seen: "2024-01-01T00:00:00Z".to_string(),
                last_seen: "2026-08-29T00:00:00Z".to_string(),
                ttl: Some(3600),
            })
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &records, |b, records| {
            b.iter(|| {
                let mut agg = PassiveDnsAggregator::new();
                agg.ingest_batch(records.clone());
                agg
            });
        });
    }

    group.finish();
}

fn bench_historical_replay(c: &mut Criterion) {
    let mut group = c.benchmark_group("historical_replay");
    group.sample_size(20);

    for dns_size in [10_000, 100_000, 1_000_000] {
        // 30-day synthetic passive DNS corpus + 100K BGP prefixes.
        let bgp_csv = generate_bgp_csv(100_000);
        let records: Vec<PassiveDnsRecord> = (0..dns_size)
            .map(|i| PassiveDnsRecord {
                domain: format!("host{}.example.com", i % 100_000),
                ip_address: format!(
                    "{}.{}.{}.{}",
                    10 + ((i >> 24) & 0xFF),
                    (i >> 16) & 0xFF,
                    (i >> 8) & 0xFF,
                    i & 0xFF
                ),
                record_type: "A".to_string(),
                first_seen: "2026-08-01T00:00:00Z".to_string(),
                last_seen: "2026-08-31T00:00:00Z".to_string(),
                ttl: Some(3600),
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("replay", dns_size),
            &(records, bgp_csv),
            |b, (records, bgp_csv)| {
                b.iter(|| {
                    let mut engine = HistoricalReplayEngine::new(ReplayConfig::default());
                    engine.load_bgp_dump(bgp_csv.as_bytes()).unwrap();
                    engine.ingest_dns_batch(records.clone());
                    engine.replay().len()
                });
            },
        );
    }

    group.finish();
}

fn generate_bgp_csv(size: usize) -> String {
    let mut csv_data = String::from("prefix,asn,as_path,origin_asn\n");
    for i in 0..size {
        let octets = [(i >> 24) as u8, (i >> 16) as u8, (i >> 8) as u8, i as u8];
        let asn = (i % 65535) + 1;
        csv_data.push_str(&format!(
            "{}.{}.{}.0/24,{},{},{}\n",
            octets[0],
            octets[1],
            octets[2],
            asn,
            asn,
            (asn + 1) % 65535 + 1
        ));
    }
    csv_data
}

criterion_group!(
    benches,
    bench_bgp_csv_parse,
    bench_ct_csv_parse,
    bench_passive_dns_ingest,
    bench_historical_replay,
);
criterion_main!(benches);
