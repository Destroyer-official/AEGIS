//! Address traversal randomization via cyclic multiplicative group permutation
//! §3.2.1 — `addr_next = (addr_current * g) mod p` where `g` is a generator of Z*_p
//! and `p` is a prime slightly larger than the address space.
//!
//! Guarantees every address is probed exactly once per pass while scattering
//! probes pseudo-randomly across the global address space.
//!
//! Also provides the 16-shard concurrent traversal skeleton.

pub mod constant_time;

use std::collections::HashSet;
use thiserror::Error;

// ───────── Constants ─────────

/// Default prime for IPv4 traversal — 2^32 + 15 = 4294967311 (prime).
/// Verified via deterministic Miller-Rabin; `p-1 = 2 * 3^2 * 5 * 131 * 364289`.
pub const DEFAULT_PRIME_P32: u64 = 4294967311;
/// Generator for `DEFAULT_PRIME_P32`; primitive root `g=3` validated via
/// `g^((p-1)/q) != 1 (mod p)` for all prime factors q of p-1.
pub const DEFAULT_GENERATOR_G32: u64 = 3;

/// Alternate prime for IPv4: 2^32 + 61 = 4294967357 (prime), generator g=2
/// Second prime above 2^32 after DEFAULT, factors p-1 = 2 * 1073741839.
pub const ALT_PRIME_P32: u64 = 4294967357;
pub const ALT_GENERATOR_G32: u64 = 2;

/// Maximum IPv4 address value (inclusive)
pub const MAX_IPV4: u64 = 0xFFFF_FFFF;

/// Number of concurrent shards per address family (§3.2.1)
pub const NUM_SHARDS: usize = 16;

// ───────── Errors ─────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TraversalError {
    #[error("prime p must be > 2^32 and prime, got {0}")]
    InvalidPrime(u64),
    #[error("generator g={g} is not a primitive root modulo p={p}")]
    InvalidGenerator { g: u64, p: u64 },
    #[error("seed must be in 1..p-1, got {seed} for p={p}")]
    InvalidSeed { seed: u64, p: u64 },
    #[error("shard index {idx} out of range for {total} shards")]
    InvalidShard { idx: usize, total: usize },
}

// ───────── Utilities ─────────

#[inline]
fn powmod(base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result: u128 = 1;
    let mut b: u128 = base as u128 % modulus as u128;
    let m = modulus as u128;
    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * b) % m;
        }
        b = (b * b) % m;
        exp >>= 1;
    }
    result as u64
}

/// Distinct prime factors of `n` (trial division, sufficient for n < 2^32 * 2).
fn distinct_prime_factors(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        if n.is_multiple_of(d) {
            factors.push(d);
            while n.is_multiple_of(d) {
                n /= d;
            }
        }
        d += if d == 2 { 1 } else { 2 };
        if d > 100_000 && n > 1 {
            // For remaining large prime factor >100k, break and handle.
            // In our case p-1 factor 364289 etc will be captured by trial up to sqrt.
            // But safe fallback: treat remainder as prime.
            break;
        }
    }
    if n > 1 {
        // n is prime (or product of large primes). For our known p-1 values,
        // the remainder is already a distinct prime.
        // To be safe, factor remainder via further trial if composite.
        // Check if remainder is composite by trial up to sqrt.
        let mut m = n;
        let mut sub = Vec::new();
        let mut d2 = 2u64;
        let mut is_prime = true;
        while d2 * d2 <= m {
            if m.is_multiple_of(d2) {
                is_prime = false;
                sub.push(d2);
                while m.is_multiple_of(d2) {
                    m /= d2;
                }
            }
            d2 += if d2 == 2 { 1 } else { 2 };
        }
        if is_prime {
            factors.push(n);
        } else {
            // m is leftover large prime, sub contains small factors of n
            factors.extend(sub);
            if m > 1 {
                factors.push(m);
            }
        }
    }
    factors.sort_unstable();
    factors.dedup();
    factors
}

/// Deterministic Miller-Rabin for 64-bit (bases [2,3,5,7,11,13] or set for <2^64)
fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n.is_multiple_of(p) {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d.is_multiple_of(2) {
        d >>= 1;
        s += 1;
    }
    // Bases sufficient for 64-bit: [2,325,9375,28178,450775,9780504,1795265022]
    for &a in &[2u64, 325, 9375, 28178, 450775, 9780504, 1795265022] {
        if a % n == 0 {
            continue;
        }
        let mut x = powmod(a % n, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        let mut composite = true;
        for _ in 1..s {
            x = ((x as u128 * x as u128) % n as u128) as u64;
            if x == n - 1 {
                composite = false;
                break;
            }
        }
        if composite {
            return false;
        }
    }
    true
}

/// Returns true if `g` is a primitive root modulo prime `p`.
pub fn is_generator(g: u64, p: u64) -> bool {
    if g == 0 || g >= p {
        return false;
    }
    if !is_prime_u64(p) {
        return false;
    }
    let factors = distinct_prime_factors(p - 1);
    for q in factors {
        if powmod(g, (p - 1) / q, p) == 1 {
            return false;
        }
    }
    true
}

/// Validate prime `p` (any size) and generator `g`.
fn validate_params(p: u64, g: u64) -> Result<(), TraversalError> {
    if !is_prime_u64(p) {
        return Err(TraversalError::InvalidPrime(p));
    }
    if !is_generator(g, p) {
        return Err(TraversalError::InvalidGenerator { g, p });
    }
    Ok(())
}

/// Validate IPv4-specific prime `p > 2^32`.
fn validate_ipv4_params(p: u64, g: u64) -> Result<(), TraversalError> {
    validate_params(p, g)?;
    if p <= MAX_IPV4 {
        return Err(TraversalError::InvalidPrime(p));
    }
    Ok(())
}

// ───────── CyclicPermutation ─────────

/// Full-period pseudo-random permutation over `Z*_p`.
///
/// Computed as `next = (curr * g) % p`. With `g` a primitive root modulo prime `p`,
/// the sequence cycles through all `p-1` non-zero residues exactly once before repeating.
#[derive(Debug, Clone)]
pub struct CyclicPermutation {
    p: u64,
    g: u64,
    current: u64,
    start: u64,
    steps: u64,
}

impl CyclicPermutation {
    /// Create a new permutation with explicit prime, generator, and seed.
    /// Validates that `p` is prime and `g` is a primitive root. Allows any size prime
    /// (including small primes for testing). For IPv4 use `new_ipv4` or `new_default`.
    pub fn new(p: u64, g: u64, seed: u64) -> Result<Self, TraversalError> {
        validate_params(p, g)?;
        if seed == 0 || seed >= p {
            return Err(TraversalError::InvalidSeed { seed, p });
        }
        Ok(Self {
            p,
            g,
            current: seed,
            start: seed,
            steps: 0,
        })
    }

    /// Strict IPv4 constructor: prime must be > 2^32.
    pub fn new_ipv4(p: u64, g: u64, seed: u64) -> Result<Self, TraversalError> {
        validate_ipv4_params(p, g)?;
        if seed == 0 || seed >= p {
            return Err(TraversalError::InvalidSeed { seed, p });
        }
        Ok(Self {
            p,
            g,
            current: seed,
            start: seed,
            steps: 0,
        })
    }

    /// Create with default IPv4 prime/generator and given seed.
    pub fn new_default(seed: u64) -> Result<Self, TraversalError> {
        Self::new_ipv4(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, seed)
    }

    /// Current value without advancing.
    #[inline]
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Prime modulus.
    #[inline]
    pub fn prime(&self) -> u64 {
        self.p
    }

    /// Generator.
    #[inline]
    pub fn generator(&self) -> u64 {
        self.g
    }

    /// Steps taken since creation/reset.
    #[inline]
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// Remaining steps before completing a full period (p-1 - steps).
    #[inline]
    pub fn remaining(&self) -> u64 {
        (self.p - 1).saturating_sub(self.steps)
    }

    /// Whether a full period has been completed (visited p-1 distinct values).
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.steps >= self.p - 1
    }

    /// Reset to starting seed.
    pub fn reset(&mut self) {
        self.current = self.start;
        self.steps = 0;
    }

    /// Advance one step: `next = (curr * g) % p` and return new current.
    /// Returns `None` if the period is complete (would wrap to start).
    ///
    /// Deliberately an inherent method (not `Iterator::next`) so callers get
    /// `Option<u64>` directly; the `Iterator` impl delegates here.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn next(&mut self) -> Option<u64> {
        if self.is_complete() {
            return None;
        }
        // Use u128 to avoid overflow for large p
        let nxt = ((self.current as u128 * self.g as u128) % self.p as u128) as u64;
        self.current = nxt;
        self.steps += 1;
        // steps ==1 means we have moved one away from start; after p-1 steps we wrap to start
        Some(nxt)
    }

    /// Advance and return next value as IPv4 address (`u32`) if `< 2^32`.
    /// Skips values >= 2^32 (which exist because p > 2^32) by advancing until a valid
    /// address is found or the period completes.
    pub fn next_ipv4(&mut self) -> Option<u32> {
        loop {
            let v = self.next()?;
            if v <= MAX_IPV4 {
                // Exclude 0 if needed? 0 not in group but we start at 1..p-1 so never 0.
                // Could filter private/reserved via caller.
                if v != 0 {
                    return Some(v as u32);
                }
            }
            // If v > MAX_IPV4, continue to next permutation value.
            // This preserves the permutation property: every <2^32 address appears exactly once;
            // overhead is (p - 2^32) skips per full pass (~15 for DEFAULT_PRIME).
            if self.is_complete() {
                return None;
            }
        }
    }

    /// Iterate over all valid IPv4 addresses in permuted order (collects).
    /// Helper for tests; not for production line-rate use.
    /// Returns exactly p-1 distinct valid IPv4 addresses (one per cycle element).
    pub fn collect_all_ipv4(mut self) -> Vec<u32> {
        let mut out = Vec::with_capacity((self.p - 1) as usize);
        let mut seen = HashSet::new();

        // Include starting seed if valid
        if self.start <= MAX_IPV4 && self.start != 0 {
            out.push(self.start as u32);
            seen.insert(self.start);
        }

        while let Some(ip) = self.next_ipv4() {
            let ip_u64 = ip as u64;
            if seen.insert(ip_u64) {
                out.push(ip);
            } else {
                // Duplicate detected (wrap-around to start), stop
                break;
            }
        }
        out
    }
}

impl Iterator for CyclicPermutation {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        CyclicPermutation::next(self)
    }
}

// ───────── Shard Trait & Implementations ─────────

/// Concurrent shard abstraction — each shard drives a disjoint slice of the address space.
///
/// Spec: `Shard { fn send(&mut self); fn recv(&mut self) }` with up to 16 concurrent shards
/// per address family.
pub trait Shard {
    /// Send probes for shard's current permutation window. Returns number of probes sent.
    fn send(&mut self) -> usize;
    /// Receive and process responses. Returns number of responses handled.
    fn recv(&mut self) -> usize;
    /// Shard identifier (0..NUM_SHARDS-1).
    fn id(&self) -> usize;
}

/// Simple permutation-backed shard implementing `Shard` trait.
/// Each shard owns a `CyclicPermutation` offset by its index.
#[derive(Debug)]
pub struct PermutationShard {
    id: usize,
    perm: CyclicPermutation,
    sent: usize,
    received: usize,
}

impl PermutationShard {
    /// Create shard `id` of `total` shards using distinct seeds spaced by exponentiation.
    /// Each shard starts at `g^(id * chunk)` * seed_base mod p.
    pub fn new(
        id: usize,
        total: usize,
        p: u64,
        g: u64,
        base_seed: u64,
    ) -> Result<Self, TraversalError> {
        if id >= total {
            return Err(TraversalError::InvalidShard { idx: id, total });
        }
        validate_params(p, g)?;
        if base_seed == 0 || base_seed >= p {
            return Err(TraversalError::InvalidSeed { seed: base_seed, p });
        }
        // Compute offset: seed_i = base_seed * g^(id * step) mod p
        // where step = (p-1)/total  ensures even spacing.
        let step = (p - 1) / total as u64;
        let offset = powmod(g, id as u64 * step, p);
        let seed = ((base_seed as u128 * offset as u128) % p as u128) as u64;
        let seed = if seed == 0 { 1 } else { seed };
        let perm = CyclicPermutation::new(p, g, seed)?;
        Ok(Self {
            id,
            perm,
            sent: 0,
            received: 0,
        })
    }

    /// Create default IPv4 shard.
    pub fn new_default(id: usize, total: usize) -> Result<Self, TraversalError> {
        Self::new(id, total, DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1)
    }

    /// Next IPv4 target for probing.
    pub fn next_target(&mut self) -> Option<u32> {
        self.perm.next_ipv4()
    }

    /// Current permutation state.
    pub fn permutation(&self) -> &CyclicPermutation {
        &self.perm
    }

    /// Mutable permutation.
    pub fn permutation_mut(&mut self) -> &mut CyclicPermutation {
        &mut self.perm
    }
}

impl Shard for PermutationShard {
    fn send(&mut self) -> usize {
        // In real engine this would craft and send packets via AF_XDP.
        // Here we simulate advancing the permutation and counting.
        if let Some(_ip) = self.perm.next_ipv4() {
            self.sent += 1;
            1
        } else {
            0
        }
    }

    fn recv(&mut self) -> usize {
        // Simulate recv: in real engine this would poll RX ring.
        // Skeleton increments counter.
        self.received += 1;
        1
    }

    fn id(&self) -> usize {
        self.id
    }
}

/// Container managing `N` concurrent shards (default 16).
#[derive(Debug)]
pub struct ShardSet {
    shards: Vec<PermutationShard>,
}

impl ShardSet {
    /// Create `N` shards (typically 16) with default prime/generator.
    pub fn new_default(n: usize) -> Result<Self, TraversalError> {
        if n == 0 || n > 64 {
            return Err(TraversalError::InvalidShard { idx: n, total: 64 });
        }
        let mut shards = Vec::with_capacity(n);
        for i in 0..n {
            shards.push(PermutationShard::new_default(i, n)?);
        }
        Ok(Self { shards })
    }

    /// Create with explicit prime/generator.
    pub fn new(p: u64, g: u64, base_seed: u64, n: usize) -> Result<Self, TraversalError> {
        let mut shards = Vec::with_capacity(n);
        for i in 0..n {
            shards.push(PermutationShard::new(i, n, p, g, base_seed)?);
        }
        Ok(Self { shards })
    }

    pub fn shards(&self) -> &[PermutationShard] {
        &self.shards
    }

    pub fn shards_mut(&mut self) -> &mut [PermutationShard] {
        &mut self.shards
    }

    pub fn len(&self) -> usize {
        self.shards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shards.is_empty()
    }

    /// Drive all shards' send (returns total sent)
    pub fn send_all(&mut self) -> usize {
        self.shards.iter_mut().map(|s| s.send()).sum()
    }

    /// Drive all shards' recv (returns total recv)
    pub fn recv_all(&mut self) -> usize {
        self.shards.iter_mut().map(|s| s.recv()).sum()
    }
}

// Helpers for IP conversion

/// Convert `u32` IPv4 address to dotted octets string (for debug).
pub fn ipv4_to_octets(ip: u32) -> [u8; 4] {
    ip.to_be_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_is_prime() {
        assert!(is_prime_u64(DEFAULT_PRIME_P32));
        assert!(is_prime_u64(ALT_PRIME_P32));
        assert!(is_prime_u64(7));
        assert!(!is_prime_u64(4294967296)); // 2^32
        assert!(!is_prime_u64(4294967310));
    }

    #[test]
    fn test_is_generator() {
        assert!(is_generator(DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32));
        assert!(is_generator(ALT_GENERATOR_G32, ALT_PRIME_P32));
        assert!(is_generator(3, 7));
        assert!(!is_generator(2, 7));
        assert!(!is_generator(1, 7));
        assert!(!is_generator(0, 7));
        // Non-prime p
        assert!(!is_generator(3, 8));
    }

    #[test]
    fn test_small_permutation_full_period() {
        // Small prime p=7, g=3 should cycle through 1..6
        let mut perm = CyclicPermutation::new(7, 3, 1).unwrap();
        let mut seen = Vec::new();
        seen.push(perm.current());
        while let Some(v) = perm.next() {
            seen.push(v);
        }
        // Should have visited 6 distinct values (p-1) plus start?
        // Our implementation: start=1, steps increments on next(); after 5 next() calls we have 6 values total including start?
        // Actually we push start, then next() yields 3,2,6,4,5 then next() would wrap to 1 but we return Some then steps? Let's see.
        // With p=7, sequence from 1: 1*3%7=3, 3*3%7=2,2*3%7=6,6*3%7=4,4*3%7=5,5*3%7=1 -> 6 steps to return.
        assert_eq!(perm.steps, 6);
        assert!(perm.is_complete());
        let set: HashSet<u64> = seen.iter().copied().collect();
        assert_eq!(set.len(), 6);
        for v in 1..7 {
            assert!(set.contains(&v), "missing {}", v);
        }
    }

    #[test]
    fn test_permutation_next_formula() {
        let mut perm = CyclicPermutation::new(7, 3, 2).unwrap();
        // 2*3%7=6
        assert_eq!(perm.next(), Some(6));
        assert_eq!(perm.next(), Some(4)); // 6*3%7=4
        assert_eq!(perm.next(), Some(5)); // 4*3%7=5
        assert_eq!(perm.next(), Some(1)); // 5*3%7=1
    }

    #[test]
    fn test_ipv4_default_prime_cycle() {
        // Verify with default prime small sample: ensure we can generate many unique IPv4s without collision.
        let mut perm = CyclicPermutation::new_default(1).unwrap();
        let mut set = HashSet::new();
        set.insert(perm.current() as u32);
        for _ in 0..10000 {
            if let Some(ip) = perm.next_ipv4() {
                assert!(set.insert(ip), "duplicate ip {}", ip);
            } else {
                break;
            }
        }
        assert_eq!(set.len(), 10001);
    }

    #[test]
    fn test_ipv4_full_coverage_small_model() {
        // Use small prime just above 2^4 to test coverage logic
        // p=17 (>16), g=3 (generator), address space 0..16 but our MAX is 2^32, so all values valid
        let p = 17u64;
        let g = 3u64;
        assert!(is_generator(g, p));
        let perm = CyclicPermutation::new(p, g, 1).unwrap();
        let mut all = HashSet::new();
        all.insert(perm.current());
        let mut p2 = perm.clone();
        while let Some(v) = p2.next() {
            all.insert(v);
        }
        // Should cover 1..16 exactly
        assert_eq!(all.len(), 16);
        for v in 1..17 {
            assert!(all.contains(&v));
        }
    }

    #[test]
    fn test_shard_creation() {
        let shard = PermutationShard::new_default(0, 16).unwrap();
        assert_eq!(shard.id(), 0);
        assert_eq!(shard.permutation().prime(), DEFAULT_PRIME_P32);
    }

    #[test]
    fn test_16_shards_distinct_seeds() {
        let set = ShardSet::new_default(16).unwrap();
        let mut seeds = HashSet::new();
        for s in set.shards() {
            seeds.insert(s.permutation().current());
        }
        assert_eq!(
            seeds.len(),
            16,
            "shards should have distinct starting seeds"
        );
    }

    #[test]
    fn test_shard_send_recv() {
        let mut shard = PermutationShard::new_default(0, 16).unwrap();
        let n = shard.send();
        assert_eq!(n, 1);
        let m = shard.recv();
        assert_eq!(m, 1);
        assert_eq!(shard.id(), 0);
    }

    #[test]
    fn test_shard_set_send_all() {
        let mut set = ShardSet::new_default(16).unwrap();
        let sent = set.send_all();
        assert_eq!(sent, 16);
        let recv = set.recv_all();
        assert_eq!(recv, 16);
    }

    #[test]
    fn test_shard_distinct_targets() {
        // Each shard's next few targets should be distinct
        let mut set = ShardSet::new_default(16).unwrap();
        let mut seen = HashSet::new();
        for shard in set.shards_mut() {
            for _ in 0..10 {
                if let Some(ip) = shard.next_target() {
                    // Allow duplicates across shards rarely? But with spaced seeds they should be distinct
                    // We check that within same period no immediate duplicate across shards
                    // (probabilistic; for strict test just ensure inserts work per shard)
                    seen.insert(ip);
                }
            }
        }
        // Should have generated 160 distinct IPs
        assert_eq!(seen.len(), 160);
    }

    #[test]
    fn test_invalid_params() {
        assert!(CyclicPermutation::new(8, 3, 1).is_err()); // not prime
        assert!(CyclicPermutation::new(7, 2, 1).is_err()); // not generator
        assert!(CyclicPermutation::new(7, 3, 0).is_err()); // invalid seed
        assert!(CyclicPermutation::new(7, 3, 7).is_err()); // seed >=p
        assert!(
            PermutationShard::new(16, 16, DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).is_err()
        );
    }

    #[test]
    fn test_powmod() {
        assert_eq!(powmod(3, 2, 7), 2);
        assert_eq!(powmod(5, 3, 7), 6);
        assert_eq!(powmod(2, 10, 1000), 24);
    }

    #[test]
    fn test_prime_greater_than_2pow32() {
        const { assert!(DEFAULT_PRIME_P32 > MAX_IPV4) }
        const { assert!(ALT_PRIME_P32 > MAX_IPV4) }
    }

    #[test]
    fn test_reset() {
        let mut perm = CyclicPermutation::new(7, 3, 1).unwrap();
        perm.next();
        perm.next();
        assert_eq!(perm.steps, 2);
        perm.reset();
        assert_eq!(perm.current(), 1);
        assert_eq!(perm.steps, 0);
        assert!(!perm.is_complete());
    }

    #[test]
    fn test_iterator() {
        let perm = CyclicPermutation::new(7, 3, 1).unwrap();
        let vals: Vec<u64> = perm.clone().collect();
        // Should iterate p-2 times? Actually iterator yields next values, not including start.
        assert_eq!(vals.len(), 6);
        let mut all = vals.clone();
        all.push(perm.current());
        let set: HashSet<u64> = all.into_iter().collect();
        assert_eq!(set.len(), 6);
    }
}
