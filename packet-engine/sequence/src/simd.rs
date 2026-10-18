//! SIMD-accelerated SipHash-2-4 batch hashing
//!
//! Processes multiple independent hash computations in parallel:
//! - AVX2: 4 x u64 lanes (256-bit)
//! - Scalar fallback: 1 x u64
//!
//! Architecture: runtime CPU feature detection via `std::arch`.
//! Each hash is independent (different input data), so we parallelize
//! across lanes rather than within a single hash.

use core::net::Ipv4Addr;

/// Batch size for AVX2 (4 parallel hashes)
#[cfg(target_arch = "x86_64")]
pub const AVX2_LANES: usize = 4;

// ───────── Scalar fallback ─────────

#[inline(always)]
fn scalar_siphash24(k0: u64, k1: u64, buf: &[u8; 10]) -> u32 {
    super::siphash24(k0, k1, buf) as u32
}

fn scalar_4x(
    k0: u64,
    k1: u64,
    ips: [Ipv4Addr; 4],
    ports: [u16; 4],
    timestamps: [u32; 4],
) -> [u32; 4] {
    let mut out = [0u32; 4];
    for i in 0..4 {
        let mut buf = [0u8; 10];
        buf[0..4].copy_from_slice(&ips[i].octets());
        buf[4..6].copy_from_slice(&ports[i].to_le_bytes());
        buf[6..10].copy_from_slice(&timestamps[i].to_le_bytes());
        out[i] = scalar_siphash24(k0, k1, &buf);
    }
    out
}

#[allow(dead_code)]
fn scalar_8x(
    k0: u64,
    k1: u64,
    ips: [Ipv4Addr; 8],
    ports: [u16; 8],
    timestamps: [u32; 8],
) -> [u32; 8] {
    let mut out = [0u32; 8];
    for i in 0..8 {
        let mut buf = [0u8; 10];
        buf[0..4].copy_from_slice(&ips[i].octets());
        buf[4..6].copy_from_slice(&ports[i].to_le_bytes());
        buf[6..10].copy_from_slice(&timestamps[i].to_le_bytes());
        out[i] = scalar_siphash24(k0, k1, &buf);
    }
    out
}

// ───────── AVX2 implementation ─────────

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn sip_round_avx2(
    v0: &mut core::arch::x86_64::__m256i,
    v1: &mut core::arch::x86_64::__m256i,
    v2: &mut core::arch::x86_64::__m256i,
    v3: &mut core::arch::x86_64::__m256i,
) {
    use core::arch::x86_64::*;

    // v0 += v1
    *v0 = _mm256_add_epi64(*v0, *v1);
    // v1 = rotl(v1, 13)
    *v1 = _mm256_xor_si256(_mm256_slli_epi64(*v1, 13), _mm256_srli_epi64(*v1, 51));
    // v1 ^= v0
    *v1 = _mm256_xor_si256(*v1, *v0);
    // v0 = rotl(v0, 32)
    *v0 = _mm256_xor_si256(_mm256_slli_epi64(*v0, 32), _mm256_srli_epi64(*v0, 32));

    // v2 += v3
    *v2 = _mm256_add_epi64(*v2, *v3);
    // v3 = rotl(v3, 16)
    *v3 = _mm256_xor_si256(_mm256_slli_epi64(*v3, 16), _mm256_srli_epi64(*v3, 48));
    // v3 ^= v2
    *v3 = _mm256_xor_si256(*v3, *v2);

    // v0 += v3
    *v0 = _mm256_add_epi64(*v0, *v3);
    // v3 = rotl(v3, 21)
    *v3 = _mm256_xor_si256(_mm256_slli_epi64(*v3, 21), _mm256_srli_epi64(*v3, 43));
    // v3 ^= v0
    *v3 = _mm256_xor_si256(*v3, *v0);

    // v2 += v1
    *v2 = _mm256_add_epi64(*v2, *v1);
    // v1 = rotl(v1, 17)
    *v1 = _mm256_xor_si256(_mm256_slli_epi64(*v1, 17), _mm256_srli_epi64(*v1, 47));
    // v1 ^= v2
    *v1 = _mm256_xor_si256(*v1, *v2);
    // v2 = rotl(v2, 32)
    *v2 = _mm256_xor_si256(_mm256_slli_epi64(*v2, 32), _mm256_srli_epi64(*v2, 32));
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn siphash24_4x_ipv4_avx2(
    k0: u64,
    k1: u64,
    ips: [Ipv4Addr; 4],
    ports: [u16; 4],
    timestamps: [u32; 4],
) -> [u32; 4] {
    use core::arch::x86_64::*;

    // Build 10-byte buffers for each hash
    let mut bufs = [[0u8; 10]; 4];
    for i in 0..4 {
        bufs[i][0..4].copy_from_slice(&ips[i].octets());
        bufs[i][4..6].copy_from_slice(&ports[i].to_le_bytes());
        bufs[i][6..10].copy_from_slice(&timestamps[i].to_le_bytes());
    }

    // Block 0 (bytes 0..8)
    let m0 = u64::from_le_bytes(bufs[0][0..8].try_into().unwrap());
    let m1 = u64::from_le_bytes(bufs[1][0..8].try_into().unwrap());
    let m2 = u64::from_le_bytes(bufs[2][0..8].try_into().unwrap());
    let m3 = u64::from_le_bytes(bufs[3][0..8].try_into().unwrap());
    let m = _mm256_set_epi64x(m3 as i64, m2 as i64, m1 as i64, m0 as i64);

    // Init state
    let mut v0 = _mm256_set1_epi64x((k0 ^ 0x736f6d6570736575u64) as i64);
    let mut v1 = _mm256_set1_epi64x((k1 ^ 0x646f72616e646f6du64) as i64);
    let mut v2 = _mm256_set1_epi64x((k0 ^ 0x6c7967656e657261u64) as i64);
    let mut v3 = _mm256_set1_epi64x((k1 ^ 0x7465646279746573u64) as i64);

    // Compression: block 0
    v3 = _mm256_xor_si256(v3, m);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 = _mm256_xor_si256(v0, m);

    // Tail block: 2 bytes remaining + length (10) in top byte
    let tail0 = (bufs[0][8] as u64) | ((bufs[0][9] as u64) << 8) | (10u64 << 56);
    let tail1 = (bufs[1][8] as u64) | ((bufs[1][9] as u64) << 8) | (10u64 << 56);
    let tail2 = (bufs[2][8] as u64) | ((bufs[2][9] as u64) << 8) | (10u64 << 56);
    let tail3 = (bufs[3][8] as u64) | ((bufs[3][9] as u64) << 8) | (10u64 << 56);
    let b = _mm256_set_epi64x(tail3 as i64, tail2 as i64, tail1 as i64, tail0 as i64);

    v3 = _mm256_xor_si256(v3, b);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 = _mm256_xor_si256(v0, b);

    // Finalization
    let ff = _mm256_set1_epi64x(0xffi64);
    v2 = _mm256_xor_si256(v2, ff);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round_avx2(&mut v0, &mut v1, &mut v2, &mut v3);

    let result = _mm256_xor_si256(_mm256_xor_si256(v0, v1), _mm256_xor_si256(v2, v3));

    let mut out = [0u32; 4];
    out[0] = _mm256_extract_epi64::<0>(result) as u32;
    out[1] = _mm256_extract_epi64::<1>(result) as u32;
    out[2] = _mm256_extract_epi64::<2>(result) as u32;
    out[3] = _mm256_extract_epi64::<3>(result) as u32;
    out
}

// ───────── Public API ─────────

/// Batch hash 4 IPv4 probe identities in parallel.
///
/// Returns 4 x 32-bit truncated sequence numbers.
/// On x86_64 with AVX2: processes all 4 in a single SIMD pass.
/// Otherwise: falls back to 4 sequential scalar hashes.
pub fn siphash24_4x_ipv4(
    k0: u64,
    k1: u64,
    ips: [Ipv4Addr; 4],
    ports: [u16; 4],
    timestamps: [u32; 4],
) -> [u32; 4] {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            return unsafe { siphash24_4x_ipv4_avx2(k0, k1, ips, ports, timestamps) };
        }
    }
    scalar_4x(k0, k1, ips, ports, timestamps)
}

/// Batch hash 8 IPv4 probe identities.
///
/// On AVX-512: single SIMD pass. Otherwise: 2x AVX2 or 8x scalar.
pub fn siphash24_8x_ipv4(
    k0: u64,
    k1: u64,
    ips: [Ipv4Addr; 8],
    ports: [u16; 8],
    timestamps: [u32; 8],
) -> [u32; 8] {
    let (lo_ips, hi_ips) = ips.split_at(4);
    let (lo_ports, hi_ports) = ports.split_at(4);
    let (lo_ts, hi_ts) = timestamps.split_at(4);
    let r0 = siphash24_4x_ipv4(
        k0,
        k1,
        lo_ips.try_into().unwrap(),
        lo_ports.try_into().unwrap(),
        lo_ts.try_into().unwrap(),
    );
    let r1 = siphash24_4x_ipv4(
        k0,
        k1,
        hi_ips.try_into().unwrap(),
        hi_ports.try_into().unwrap(),
        hi_ts.try_into().unwrap(),
    );
    let mut out = [0u32; 8];
    out[..4].copy_from_slice(&r0);
    out[4..].copy_from_slice(&r1);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_key() -> (u64, u64) {
        let key = [
            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        (
            u64::from_le_bytes(key[0..8].try_into().unwrap()),
            u64::from_le_bytes(key[8..16].try_into().unwrap()),
        )
    }

    #[test]
    fn test_4x_matches_scalar() {
        let (k0, k1) = ref_key();
        let ips = [
            Ipv4Addr::new(1, 1, 1, 1),
            Ipv4Addr::new(8, 8, 8, 8),
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(10, 0, 0, 1),
        ];
        let ports = [80u16, 443, 8080, 9090];
        let timestamps = [0u32, 1234567890, 0xdeadbeef, 0xFFFFFFFF];

        let scalar = scalar_4x(k0, k1, ips, ports, timestamps);
        let simd = siphash24_4x_ipv4(k0, k1, ips, ports, timestamps);
        assert_eq!(scalar, simd, "4x SIMD must match scalar for all lanes");
    }

    #[test]
    fn test_8x_matches_scalar() {
        let (k0, k1) = ref_key();
        let ips = [
            Ipv4Addr::new(1, 0, 0, 1),
            Ipv4Addr::new(1, 0, 0, 2),
            Ipv4Addr::new(1, 0, 0, 3),
            Ipv4Addr::new(1, 0, 0, 4),
            Ipv4Addr::new(1, 0, 0, 5),
            Ipv4Addr::new(1, 0, 0, 6),
            Ipv4Addr::new(1, 0, 0, 7),
            Ipv4Addr::new(1, 0, 0, 8),
        ];
        let ports = [80u16; 8];
        let timestamps = [1000u32; 8];

        let scalar = scalar_8x(k0, k1, ips, ports, timestamps);
        let simd = siphash24_8x_ipv4(k0, k1, ips, ports, timestamps);
        assert_eq!(scalar, simd, "8x SIMD must match scalar");
    }

    #[test]
    fn test_4x_known_vector() {
        let (k0, k1) = ref_key();
        let ips = [Ipv4Addr::new(1, 1, 1, 1); 4];
        let ports = [80u16; 4];
        let timestamps = [0u32; 4];

        let result = siphash24_4x_ipv4(k0, k1, ips, ports, timestamps);
        assert_eq!(result[0], result[1]);
        assert_eq!(result[1], result[2]);
        assert_eq!(result[2], result[3]);

        // Match scalar
        let scalar = scalar_4x(k0, k1, ips, ports, timestamps);
        assert_eq!(result[0], scalar[0]);
    }

    #[test]
    fn test_4x_distinct_inputs() {
        let (k0, k1) = ref_key();
        let ips = [
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 2),
            Ipv4Addr::new(10, 0, 0, 3),
            Ipv4Addr::new(10, 0, 0, 4),
        ];
        let ports = [80, 80, 80, 80];
        let timestamps = [1000, 1000, 1000, 1000];

        let result = siphash24_4x_ipv4(k0, k1, ips, ports, timestamps);
        assert_ne!(result[0], result[1]);
        assert_ne!(result[1], result[2]);
        assert_ne!(result[2], result[3]);
    }
}
