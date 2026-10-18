use aegis_sequence::SeqKey;
use std::net::Ipv4Addr;
use std::time::Instant;

fn main() {
    let key = SeqKey::from_bytes(&[0x42u8; 16]);
    let target = 10_000_000usize;
    let ip_base = 0xC0A80001u32;
    let port = 443u16;
    let ts = 1234567890u32;

    // Warmup
    for i in 0..1000 {
        let ip = Ipv4Addr::from(ip_base.wrapping_add(i as u32));
        let _ = key.encode_ipv4(ip, port, ts);
    }

    let start = Instant::now();
    let mut sum: u64 = 0;
    for i in 0..target {
        let ip = Ipv4Addr::from(ip_base.wrapping_add(i as u32));
        let seq = key.encode_ipv4(ip, port, ts);
        sum = sum.wrapping_add(seq as u64);
    }
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let throughput = target as f64 / secs;
    println!(
        "10M seq benchmark: {} iterations in {:.3}s => {:.2} seq/sec (sum={})",
        target, secs, throughput, sum
    );
    println!("Elapsed per seq: {:.2} ns", secs * 1e9 / target as f64);
    if throughput >= 10_000_000.0 {
        println!("PASS: >=10M seq/sec");
    } else {
        println!(
            "FAIL: below 10M seq/sec (got {:.2}M)",
            throughput / 1_000_000.0
        );
        std::process::exit(1);
    }
}
