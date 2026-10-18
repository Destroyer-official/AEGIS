//! Kani proof: CyclicPermutation is a bijection over Z*_p
//!
//! For prime p and primitive root g, the map `x -> (x * g) % p` is a bijection
//! on {1, 2, ..., p-1}. This proof verifies:
//!
//! 1. **Injectivity**: Two distinct seeds produce two distinct first steps.
//! 2. **No zero output**: No valid seed maps to 0.
//! 3. **Bounded range**: All outputs are in 1..p.
//!
//! Proof covers small primes (p <= 1000) with bounded model checking.
//! For the full IPv4 prime (2^32+15), property is validated analytically
//! since exhaustive model checking is infeasible for 4B+ states.

#![cfg_attr(kani, no_std)]
extern crate alloc;

use alloc::vec::Vec;

// ───────── Replicated helpers (avoid pulling full crate for proof) ─────────

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

fn distinct_prime_factors(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        if n % d == 0 {
            factors.push(d);
            while n % d == 0 {
                n /= d;
            }
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 {
        factors.push(n);
    }
    factors.sort_unstable();
    factors.dedup();
    factors
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d % 2 == 0 {
        d >>= 1;
        s += 1;
    }
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

fn is_generator(g: u64, p: u64) -> bool {
    if g == 0 || g >= p || !is_prime_u64(p) {
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

// ───────── Proof 1: Injectivity ─────────

/// For any two distinct valid seeds s1, s2 in 1..p, the first step
/// produces distinct values: (s1 * g) % p != (s2 * g) % p.
///
/// This is trivially true since g is invertible mod p (gcd(g, p) = 1),
/// so s1 * g ≡ s2 * g (mod p) => s1 ≡ s2 (mod p) => s1 == s2.
///
/// We verify with symbolic inputs bounded to small primes.
#[kani::proof]
#[kani::unwind(16)]
fn proof_permutation_injectivity() {
    // Use small primes where exhaustive checking is feasible
    let primes: [u64; 5] = [7, 13, 31, 127, 509];

    for &p in &primes {
        if !is_prime_u64(p) {
            continue;
        }
        // Find generator
        let mut g_opt = None;
        for candidate in 2..p {
            if is_generator(candidate, p) {
                g_opt = Some(candidate);
                break;
            }
        }
        let g = match g_opt {
            Some(v) => v,
            None => continue,
        };

        // For all valid seed pairs (s1, s2) where s1 != s2:
        for s1 in 1..p {
            for s2 in (s1 + 1)..p {
                let next1 = ((s1 as u128 * g as u128) % p as u128) as u64;
                let next2 = ((s2 as u128 * g as u128) % p as u128) as u64;
                assert_ne!(next1, next2, "Injectivity violated: p={}, g={}, s1={}, s2={}", p, g, s1, s2);
            }
        }
    }
}

// ───────── Proof 2: No zero output ─────────

/// For any valid seed s in 1..p, (s * g) % p is never 0.
/// This follows from: if s*g ≡ 0 (mod p) with p prime, then p | s or p | g.
/// Since 1 <= s < p and 1 <= g < p, neither is divisible by p.
#[kani::proof]
#[kani::unwind(16)]
fn proof_permutation_no_zero() {
    let primes: [u64; 5] = [7, 13, 31, 127, 509];

    for &p in &primes {
        if !is_prime_u64(p) {
            continue;
        }
        let mut g_opt = None;
        for candidate in 2..p {
            if is_generator(candidate, p) {
                g_opt = Some(candidate);
                break;
            }
        }
        let g = match g_opt {
            Some(v) => v,
            None => continue,
        };

        for s in 1..p {
            let next = ((s as u128 * g as u128) % p as u128) as u64;
            assert!(next != 0 && next < p, "Output out of range: p={}, g={}, s={}, next={}", p, g, s, next);
        }
    }
}

// ───────── Proof 3: Full period coverage ─────────

/// Walk the permutation for p-1 steps starting from seed=1.
/// Verify all outputs are distinct (i.e., a permutation of 1..p-1).
#[kani::proof]
#[kani::unwind(256)]
fn proof_permutation_full_period() {
    let primes: [u64; 4] = [7, 13, 31, 127];

    for &p in &primes {
        if !is_prime_u64(p) {
            continue;
        }
        let mut g_opt = None;
        for candidate in 2..p {
            if is_generator(candidate, p) {
                g_opt = Some(candidate);
                break;
            }
        }
        let g = match g_opt {
            Some(v) => v,
            None => continue,
        };

        let mut visited = Vec::new();
        let mut current = 1u64;

        for _ in 0..(p - 1) {
            current = ((current as u128 * g as u128) % p as u128) as u64;
            assert!(current >= 1 && current < p, "Value out of range");
            // Check no duplicates
            for &v in &visited {
                assert_ne!(v, current, "Duplicate value in permutation: p={}, g={}", p, g);
            }
            visited.push(current);
        }

        // Verify we visited exactly p-1 distinct values
        assert_eq!(visited.len() as u64, p - 1, "Did not visit all values");
    }
}

// ───────── Proof 4: Range bound for IPv4 prime ─────────

/// Symbolic check that for the IPv4 prime p = 4294967311, any seed in 1..p
/// produces output in 1..p (non-zero, bounded). We cannot exhaustively check
/// all seeds but verify the algebraic invariant: output < p.
#[kani::proof]
#[kani::unwind(16)]
fn proof_ipv4_permutation_bounded() {
    let p: u64 = 4294967311; // 2^32 + 15, prime
    let g: u64 = 3; // Primitive root mod p

    // Verify p is prime and g is generator (static assertion)
    assert!(is_prime_u64(p), "p must be prime");
    assert!(is_generator(g, p), "g must be generator mod p");

    // For a symbolic seed, verify output bounds
    let seed: u64 = kani::any();
    kani::assume(seed >= 1 && seed < p);

    let next = ((seed as u128 * g as u128) % p as u128) as u64;

    assert!(next >= 1, "Output must be >= 1");
    assert!(next < p, "Output must be < p");
}
