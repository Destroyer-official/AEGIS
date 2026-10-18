//! Tests for cyclic group permutation (traversal)
//! Covers edge cases:
//! - Wrap-around behavior
//! - Zero handling
//! - Identity properties
//! - Full period coverage
//! - Shard distribution

use aegis_traversal::{
    is_generator, CyclicPermutation, PermutationShard, Shard, ShardSet, DEFAULT_GENERATOR_G32,
    DEFAULT_PRIME_P32, MAX_IPV4, NUM_SHARDS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_permutation_formula() {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let current = perm.current();
        let expected_next =
            ((current as u128 * DEFAULT_GENERATOR_G32 as u128) % DEFAULT_PRIME_P32 as u128) as u64;
        let actual_next = perm.next().unwrap();
        assert_eq!(actual_next, expected_next);
    }

    #[test]
    fn test_permutation_never_zero() {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        assert_ne!(perm.current(), 0);
        for _ in 0..100 {
            let next = perm.next().unwrap();
            assert_ne!(next, 0);
        }
    }

    #[test]
    fn test_permutation_steps_increment() {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        assert_eq!(perm.steps(), 0);
        for i in 1..100 {
            perm.next().unwrap();
            assert_eq!(perm.steps(), i);
        }
    }

    #[test]
    fn test_permutation_wraparound_to_start() {
        let p = 17u64;
        let g = 3u64;
        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();

        for _ in 0..(p - 1) {
            let _ = perm.next().unwrap();
        }

        assert_eq!(perm.steps(), p - 1);
        assert!(perm.is_complete());
        assert_eq!(perm.next(), None);
    }

    #[test]
    fn test_permutation_reset_returns_to_start() {
        let mut perm =
            CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 42).unwrap();
        let start = perm.current();

        for _ in 0..50 {
            perm.next().unwrap();
        }

        perm.reset();

        assert_eq!(perm.current(), start);
        assert_eq!(perm.steps(), 0);
        assert!(!perm.is_complete());
    }

    #[test]
    fn test_permutation_full_period_covers_all_nonzero() {
        let p = 17u64;
        let g = 3u64;

        let mut perm = CyclicPermutation::new(p, g, 1).unwrap();
        let mut seen = HashSet::new();
        seen.insert(perm.current());

        while let Some(v) = perm.next() {
            seen.insert(v);
        }

        assert_eq!(seen.len(), (p - 1) as usize);

        for v in 1..p {
            assert!(seen.contains(&v), "missing value {}", v);
        }
    }

    #[test]
    fn test_next_ipv4_never_returns_zero_or_overflow() {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();

        if perm.current() <= MAX_IPV4 && perm.current() != 0 {
            assert!(perm.current() <= MAX_IPV4);
            assert_ne!(perm.current(), 0);
        }

        for _ in 0..200 {
            if let Some(ip) = perm.next_ipv4() {
                assert!(u64::from(ip) <= MAX_IPV4);
                assert_ne!(ip, 0);
            } else {
                break;
            }
        }
    }

    #[test]
    fn test_next_ipv4_filters_values_above_max_ipv4() {
        let mut perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut count = 0;

        while let Some(ip) = perm.next_ipv4() {
            assert!(u64::from(ip) <= MAX_IPV4);
            assert_ne!(ip, 0);
            count += 1;
            if count > 1000 {
                break;
            }
        }

        assert!(count > 0);
    }

    #[test]
    fn test_is_generator_rejects_zero_and_geq_p() {
        let p = DEFAULT_PRIME_P32;
        assert!(!is_generator(0, p));
        assert!(!is_generator(p, p));
        assert!(!is_generator(p + 1, p));
    }

    #[test]
    fn test_is_generator_rejects_non_prime_modulus() {
        assert!(!is_generator(3, 8));
        assert!(!is_generator(1, 8));
    }

    #[test]
    fn test_known_generators_work() {
        assert!(is_generator(DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32));
        assert!(is_generator(2, 4294967357u64));
        assert!(is_generator(3, 7));
    }

    #[test]
    fn test_shard_creation_all_distinct_seeds() {
        let set = ShardSet::new_default(NUM_SHARDS).unwrap();

        let mut seeds = HashSet::new();
        for shard in set.shards() {
            let seed = shard.permutation().current();
            assert!(seeds.insert(seed), "duplicate seed {}", seed);
        }

        assert_eq!(seeds.len(), NUM_SHARDS);
    }

    #[test]
    fn test_shard_ids_are_correct() {
        let set = ShardSet::new_default(NUM_SHARDS).unwrap();

        for (i, shard) in set.shards().iter().enumerate() {
            assert_eq!(shard.id(), i);
        }
    }

    #[test]
    fn test_shard_send_recv_increments_counters() {
        let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();

        let sent = shard.send();
        assert_eq!(sent, 1);

        let recv = shard.recv();
        assert_eq!(recv, 1);
    }

    #[test]
    fn test_shard_set_send_all_returns_count() {
        let mut set = ShardSet::new_default(NUM_SHARDS).unwrap();

        let sent = set.send_all();
        assert_eq!(sent, NUM_SHARDS);

        let recv = set.recv_all();
        assert_eq!(recv, NUM_SHARDS);
    }

    #[test]
    fn test_shard_distinct_targets_across_shards() {
        let mut set = ShardSet::new_default(NUM_SHARDS).unwrap();
        let mut seen = HashSet::new();

        for shard in set.shards_mut() {
            for _ in 0..10 {
                if let Some(ip) = shard.next_target() {
                    assert!(seen.insert(ip), "duplicate IP {} across shards", ip);
                }
            }
        }

        assert_eq!(seen.len(), NUM_SHARDS * 10);
    }

    #[test]
    fn test_shard_next_target_matches_permutation() {
        let mut shard = PermutationShard::new_default(0, NUM_SHARDS).unwrap();

        for _ in 0..100 {
            let target = shard.next_target().unwrap();
            let perm_val = shard.permutation().current();

            assert_eq!(target as u64, perm_val);

            shard.permutation_mut().next_ipv4().unwrap();
        }
    }

    #[test]
    fn test_iterator_yields_p_minus_1_values() {
        let perm = CyclicPermutation::new(17, 3, 1).unwrap();
        let count = perm.clone().count();

        // Iterator yields next() values including wrap-around = p-1 values
        assert_eq!(count, 16); // p-1 = 16
    }

    #[test]
    fn test_collect_all_ipv4_includes_valid_only() {
        let perm = CyclicPermutation::new(17, 3, 1).unwrap();
        let all = perm.collect_all_ipv4();

        // Should only include values <= MAX_IPV4 and != 0
        for ip in &all {
            assert!(u64::from(*ip) <= MAX_IPV4);
            assert_ne!(*ip, 0);
        }

        // For p=17, all 1..16 should be included exactly once (all <= MAX_IPV4)
        assert_eq!(all.len(), 16);
    }

    #[test]
    fn test_rejects_non_prime() {
        assert!(CyclicPermutation::new(8, 3, 1).is_err());
        assert!(CyclicPermutation::new(15, 2, 1).is_err());
    }

    #[test]
    fn test_rejects_bad_generator() {
        assert!(CyclicPermutation::new(7, 2, 1).is_err());
        assert!(CyclicPermutation::new(7, 4, 1).is_err());
    }

    #[test]
    fn test_rejects_zero_seed() {
        assert!(CyclicPermutation::new(7, 3, 0).is_err());
    }

    #[test]
    fn test_rejects_seed_geq_p() {
        assert!(CyclicPermutation::new(7, 3, 7).is_err());
        assert!(CyclicPermutation::new(7, 3, 8).is_err());
    }

    #[test]
    fn test_shard_rejects_invalid_id() {
        assert!(PermutationShard::new(
            NUM_SHARDS,
            NUM_SHARDS,
            DEFAULT_PRIME_P32,
            DEFAULT_GENERATOR_G32,
            1
        )
        .is_err());
    }
}
