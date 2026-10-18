//! Proptest exhaustive coverage for cyclic group permutation
//! Target: 10K+ generated test cases covering edge cases
//! - Small primes (exhaustive full-period verification)
//! - Wrap-around behavior
//! - IPv4 boundary conditions
//! - Shard distribution properties

use aegis_traversal::{
    is_generator, CyclicPermutation, PermutationShard, Shard, ShardSet, ALT_GENERATOR_G32,
    ALT_PRIME_P32, DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32, MAX_IPV4,
};
use proptest::prelude::*;

// ───────── Small Prime Exhaustive Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Property: Full-period permutation visits all p-1 non-zero residues exactly once
    #[test]
    fn full_period_all_values(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13), Just(17), Just(19), Just(23), Just(29), Just(31)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        let mut seen = std::collections::HashSet::new();
        seen.insert(perm.current());
        while let Some(v) = perm.next() {
            seen.insert(v);
        }
        prop_assert_eq!(seen.len(), (p - 1) as usize);
        for v in 1..p {
            prop_assert!(seen.contains(&v), "missing value {} in full period", v);
        }
    }

    /// Property: next() never returns 0 for valid permutation
    #[test]
    fn never_zero(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13), Just(17), Just(19), Just(23)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        prop_assert_ne!(perm.current(), 0);
        for _ in 0..(p - 1) {
            let v = perm.next().unwrap_or(0);
            prop_assert_ne!(v, 0, "permutation returned 0");
        }
    }

    /// Property: steps() increments correctly
    #[test]
    fn steps_increment(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13), Just(17)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        prop_assert_eq!(perm.steps(), 0);
        for i in 1..=10.min(p - 1) {
            perm.next();
            prop_assert_eq!(perm.steps(), i);
        }
    }

    /// Property: reset() returns to starting state
    #[test]
    fn reset_returns_to_start(
        p in prop_oneof![Just(5u64), Just(7), Just(11), Just(13), Just(17)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        let start = perm.current();
        for _ in 0..5.min(p - 1) {
            perm.next();
        }
        perm.reset();
        prop_assert_eq!(perm.current(), start);
        prop_assert_eq!(perm.steps(), 0);
        prop_assert!(!perm.is_complete());
    }

    /// Property: is_complete() is true only after p-1 steps
    #[test]
    fn is_complete_boundary(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        prop_assert!(!perm.is_complete());
        for _ in 0..(p - 2) {
            perm.next();
            prop_assert!(!perm.is_complete());
        }
        perm.next();
        prop_assert!(perm.is_complete());
        prop_assert!(perm.next().is_none());
    }
}

// ───────── Next_ipv4 Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: next_ipv4 never returns a value > MAX_IPV4
    #[test]
    fn next_ipv4_bounds(seed in 1u64..1000u64) {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, seed).unwrap();
        for _ in 0..200 {
            match perm.next_ipv4() {
                Some(ip) => {
                    prop_assert!((ip as u64) <= MAX_IPV4, "ipv4 value {} exceeds MAX_IPV4", ip);
                    prop_assert_ne!(ip, 0, "ipv4 returned 0");
                }
                None => break,
            }
        }
    }

    /// Property: next_ipv4 values are distinct within one pass
    #[test]
    fn next_ipv4_distinct(seed in 1u64..1000u64) {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, seed).unwrap();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            match perm.next_ipv4() {
                Some(ip) => prop_assert!(seen.insert(ip), "duplicate ipv4 {} within one pass", ip),
                None => break,
            }
        }
    }
}

// ───────── Shard Properties ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: shards have distinct starting seeds
    #[test]
    fn shard_distinct_seeds(shard_count in 2usize..=16usize) {
        let set = ShardSet::new_default(shard_count).unwrap();
        let mut seeds = std::collections::HashSet::new();
        for shard in set.shards() {
            prop_assert!(seeds.insert(shard.permutation().current()),
                "duplicate seed {} across shards", shard.permutation().current());
        }
        prop_assert_eq!(seeds.len(), shard_count);
    }

    /// Property: Each shard's next_target produces values in valid IPv4 range
    #[test]
    fn shard_targets_valid_range(shard_count in 2usize..=8usize) {
        let set = ShardSet::new_default(shard_count).unwrap();
        for shard in set.shards() {
            let mut perm = shard.permutation().clone();
            for _ in 0..50 {
                if let Some(v) = perm.next_ipv4() {
                    prop_assert!((v as u64) <= MAX_IPV4, "shard target {} exceeds MAX_IPV4", v);
                    prop_assert_ne!(v, 0);
                }
            }
        }
    }

    /// Property: send() returns at most 1 when permutation is not exhausted
    #[test]
    fn shard_send_returns_one(shard_count in 2usize..=8usize) {
        let mut shard = PermutationShard::new_default(0, shard_count).unwrap();
        let count = shard.send();
        prop_assert!(count <= 1);
    }

    /// Property: Shard ID is within bounds
    #[test]
    fn shard_id_bounds(id in 0usize..16usize) {
        let shard = PermutationShard::new_default(id, 16).unwrap();
        prop_assert_eq!(shard.id(), id);
    }
}

// ───────── Edge Case Tests ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: Small prime p=3, g=2 (smallest valid permutation)
    #[test]
    fn smallest_permutation(seed in 1u64..3u64) {
        let mut perm = CyclicPermutation::new(3, 2, seed).unwrap();
        let mut seen = std::collections::HashSet::new();
        seen.insert(perm.current());
        while let Some(v) = perm.next() {
            seen.insert(v);
        }
        prop_assert_eq!(seen.len(), 2);
        prop_assert!(seen.contains(&1));
        prop_assert!(seen.contains(&2));
    }

    /// Property: Large prime (ALT_PRIME_P32) works correctly
    #[test]
    fn alt_prime_works(seed in 1u64..100u64) {
        let mut perm = CyclicPermutation::new(ALT_PRIME_P32, ALT_GENERATOR_G32, seed).unwrap();
        let mut seen = std::collections::HashSet::new();
        seen.insert(perm.current());
        for _ in 0..100 {
            if let Some(v) = perm.next() {
                seen.insert(v);
            }
        }
        prop_assert_eq!(seen.len(), 101);
        for v in &seen {
            prop_assert_ne!(*v, 0);
        }
    }

    /// Property: Iterator yields correct count (p-1 values)
    #[test]
    fn iterator_count(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13), Just(17), Just(19)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        prop_assume!(!generators.is_empty());
        let g = generators[0];
        let perm = CyclicPermutation::new(p, g, 1).unwrap();
        let count = perm.into_iter().count();
        prop_assert_eq!(count, (p - 1) as usize);
    }

    /// Property: next_ipv4 produces valid IPv4 addresses (u32 range)
    #[test]
    fn ipv4_valid_range(seed in 1u64..1000u64) {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, seed).unwrap();
        for _ in 0..50 {
            if let Some(ip) = perm.next_ipv4() {
                prop_assert!((ip as u64) <= MAX_IPV4);
            } else {
                break;
            }
        }
    }

    /// Property: ipv4_to_octets roundtrip
    #[test]
    fn ipv4_octets_roundtrip(ip in 0u32..=u32::MAX) {
        let octets = aegis_traversal::ipv4_to_octets(ip);
        let reconstructed = u32::from_be_bytes(octets);
        prop_assert_eq!(ip, reconstructed);
    }
}

// ───────── Invalid Parameter Rejection ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: Non-prime modulus is always rejected
    #[test]
    fn rejects_non_prime(n in 4u64..200u64) {
        let is_composite = (2..n).any(|d| n % d == 0);
        prop_assume!(is_composite);
        prop_assert!(CyclicPermutation::new(n, 2, 1).is_err());
    }

    /// Property: Seed 0 is always rejected
    #[test]
    fn rejects_zero_seed(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13)],
    ) {
        prop_assert!(CyclicPermutation::new(p, 3, 0).is_err());
    }

    /// Property: Seed >= p is always rejected
    #[test]
    fn rejects_seed_geq_p(
        p in prop_oneof![Just(3u64), Just(5), Just(7), Just(11), Just(13)],
    ) {
        prop_assert!(CyclicPermutation::new(p, 3, p).is_err());
        prop_assert!(CyclicPermutation::new(p, 3, p + 1).is_err());
    }

    /// Property: Non-generator is always rejected
    #[test]
    fn rejects_non_generator(
        p in prop_oneof![Just(5u64), Just(7), Just(11), Just(13), Just(17)],
    ) {
        let generators: Vec<u64> = (1..p).filter(|&g| is_generator(g, p)).collect();
        let non_generators: Vec<u64> = (1..p).filter(|g| !generators.contains(g)).collect();
        prop_assume!(!non_generators.is_empty());
        let g = non_generators[0];
        prop_assert!(CyclicPermutation::new(p, g, 1).is_err(),
            "non-generator {} accepted for p={}", g, p);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn collect_all_ipv4_valid() {
        let perm = CyclicPermutation::new(17, 3, 1).unwrap();
        let all = perm.collect_all_ipv4();
        for ip in &all {
            assert!(*ip != 0, "collected IPv4 should not be 0");
            assert!((*ip as u64) <= MAX_IPV4, "collected IPv4 exceeds MAX_IPV4");
        }
        assert_eq!(all.len(), 16);
    }
}
