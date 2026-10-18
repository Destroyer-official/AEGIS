//! JA4L — Light distance and latency estimation (§6.1).
//!
//! JA4L format: `{light_distance}_{latency}`
//!
//! Estimates geographic proximity and network latency between scanner and target
//! using ICMP/TCP timestamp analysis.

/// JA4L light distance and latency fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4lFingerprint {
    pub full: String,
    /// Estimated light-distance in kilometers (0 if unknown)
    pub light_distance_km: u32,
    /// Round-trip latency in milliseconds (0 if unknown)
    pub latency_ms: u32,
}

impl Ja4lFingerprint {
    /// Compute JA4L fingerprint from latency measurements.
    ///
    /// `light_distance_km` is computed from the minimum RTT observed across
    /// multiple probes, assuming signal propagation at 2/3 speed of light.
    pub fn compute(light_distance_km: u32, latency_ms: u32) -> Self {
        let full = format!("{}_{:06}", light_distance_km, latency_ms);

        Ja4lFingerprint {
            full,
            light_distance_km,
            latency_ms,
        }
    }

    /// Estimate light distance from minimum RTT (in microseconds).
    ///
    /// Assumes signal propagation at approximately 2/3 speed of light in vacuum.
    /// speed_of_light_km_per_us = 299.792 km/ms
    /// fiber_speed = 2/3 * 299.792 ≈ 199.86 km/ms
    /// distance = (min_rtt_ms / 2) * fiber_speed
    pub fn estimate_distance_from_rtt(min_rtt_micros: u64) -> u32 {
        // Convert microseconds to milliseconds (floating point for precision)
        let rtt_ms = min_rtt_micros as f64 / 1000.0;
        // 2/3 speed of light in fiber (km/ms)
        let fiber_speed = 199.861;
        let distance = (rtt_ms / 2.0) * fiber_speed;
        distance as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4l_format() {
        let ja4l = Ja4lFingerprint::compute(150, 3);
        assert_eq!(ja4l.full, "150_000003");
    }

    #[test]
    fn test_ja4l_distance_estimation() {
        // 1ms RTT → ~100 km
        let dist = Ja4lFingerprint::estimate_distance_from_rtt(1000);
        assert!((95..105).contains(&dist), "Expected ~100km, got {}", dist);

        // 10ms RTT → ~1000 km
        let dist = Ja4lFingerprint::estimate_distance_from_rtt(10000);
        assert!(
            (950..1050).contains(&dist),
            "Expected ~1000km, got {}",
            dist
        );
    }
}
