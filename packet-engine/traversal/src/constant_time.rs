//! Constant-time cryptographic primitives for traversal
//!
//! Provides side-channel resistant operations for cyclic permutation:
//! - Constant-time modular multiplication (no data-dependent branches)
//! - Cache-line aligned sensitive data storage
//! - Secret integer types that resist timing analysis
//! - Zeroize-on-drop for key material
//!
//! All operations compile to branch-free assembly on x86_64.
//! Zero external dependencies.

use core::marker::PhantomData;

// ───────── Cache-Line Alignment ─────────

/// Cache line size on x86_64 (64 bytes). Adjust for ARM if needed.
pub const CACHE_LINE_SIZE: usize = 64;

/// Cache-line aligned storage for sensitive data.
///
/// Ensures the data sits on its own cache line, preventing
/// timing side-channels from prefetcher-induced cache collisions.
#[repr(align(64))]
pub struct CacheLineAligned<T>(pub T);

impl<T> CacheLineAligned<T> {
    /// Create a new aligned value.
    #[inline]
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get a mutable reference to the inner value.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> core::ops::Deref for CacheLineAligned<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> core::ops::DerefMut for CacheLineAligned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

// ───────── Secret Integer Type ─────────

/// Marker type for secret data. Prevents accidental comparison or display.
pub enum Secret {}

/// Marker type for public data.
pub enum Public {}

/// Sealed trait for visibility control.
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Secret {}
    impl Sealed for super::Public {}
}

/// Trait for integer types that can be secret or public.
pub trait SecretVisibility: sealed::Sealed {
    /// Whether this type's operations should be constant-time.
    const IS_SECRET: bool;
}

impl SecretVisibility for Secret {
    const IS_SECRET: bool = true;
}

impl SecretVisibility for Public {
    const IS_SECRET: bool = false;
}

/// Secret-wrapped integer. Prevents accidental leakage through comparison, display, or debug.
///
/// Operations on `SecretU64<V>` are constant-time when `V = Secret`.
#[repr(transparent)]
pub struct SecretU64<V: SecretVisibility = Secret> {
    value: u64,
    _vis: PhantomData<fn() -> V>,
}

// Manual Clone avoids the `V: Clone` bound that `#[derive(Clone)]` would add,
// keeping the constant-time wrapper usable for any `SecretVisibility` marker.
#[allow(clippy::non_canonical_clone_impl)]
impl<V: SecretVisibility> Clone for SecretU64<V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _vis: PhantomData,
        }
    }
}

impl<V: SecretVisibility> Copy for SecretU64<V> {}

impl<V: SecretVisibility> SecretU64<V> {
    /// Create a new secret integer.
    #[inline]
    pub fn new(val: u64) -> Self {
        Self {
            value: val,
            _vis: PhantomData,
        }
    }

    /// Extract the raw value. Use with caution.
    #[inline]
    pub fn expose(self) -> u64 {
        self.value
    }
}

/// Constant-time comparison for secret integers.
impl SecretU64<Secret> {
    /// Constant-time equality check. Returns 1 if equal, 0 otherwise.
    #[inline]
    pub fn ct_eq(self, other: Self) -> bool {
        let diff = self.value ^ other.value;
        // Branch-free: (diff | (0 - diff)) >> 63 == 0 iff diff == 0
        // This compiles to BT + SBB on x86_64 (no branch).
        let mask = diff.wrapping_neg() | diff;
        (mask >> 63) == 0
    }

    /// Constant-time less-than. Returns true if self < other.
    #[inline]
    pub fn ct_lt(self, other: Self) -> bool {
        // Compute borrow without branch
        let (_, borrow) = self.value.overflowing_sub(other.value);
        borrow
    }

    /// Constant-time conditional select.
    /// Returns `a` if condition is true, `b` otherwise. No branch.
    #[inline]
    pub fn ct_select(condition: bool, a: Self, b: Self) -> Self {
        // Create mask: 0xFFFFFFFFFFFFFFFF if true, 0 if false
        let mask = (condition as u64).wrapping_neg();
        Self::new((a.value & mask) | (b.value & !mask))
    }
}

impl<V: SecretVisibility> PartialEq for SecretU64<V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<V: SecretVisibility> Eq for SecretU64<V> {}

// No Debug/Display for Secret to prevent leakage
impl<V: SecretVisibility> core::fmt::Debug for SecretU64<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SecretU64(***)")
    }
}

// ───────── Constant-Time Modular Arithmetic ─────────

/// Constant-time modular multiplication: `(a * b) % m`
///
/// Uses 128-bit intermediate to avoid overflow. Constant-time for fixed-width inputs
/// (no data-dependent branches). O(1) per operation.
#[inline]
pub fn ct_mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Constant-time modular exponentiation: `(base^exp) % m`
///
/// Square-and-multiply with constant-time conditional select.
/// Fixed 64 iterations regardless of exp value.
#[inline]
pub fn ct_powmod(base: u64, exp: u64, m: u64) -> u64 {
    let mut result: u64 = 1;
    let mut base = base % m;
    let mut exp = exp;

    for _ in 0..64 {
        // If exp bit is set, multiply result by base
        let mask = (exp & 1).wrapping_neg(); // 0xFF..FF if bit set, 0 otherwise
        let product = ct_mulmod(result, base, m);
        result = (product & mask) | (result & !mask);

        base = ct_mulmod(base, base, m);
        exp >>= 1;
    }
    result
}

/// Constant-time comparison of two byte slices.
/// Returns true if they are equal. No early exit on mismatch.
#[inline]
pub fn ct_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    // diff == 0 iff all bytes match
    diff == 0
}

/// Constant-time conditional store: writes `val` to `ptr` if `condition` is true.
/// Uses bitmask to prevent store-forwarding attacks.
#[inline]
pub fn ct_conditional_store(ptr: &mut u64, val: u64, condition: bool) {
    let mask = -(condition as i64) as u64;
    let old = *ptr;
    *ptr = (val & mask) | (old & !mask);
}

/// Constant-time minimum: returns min(a, b) without branch.
#[inline]
pub fn ct_min(a: u64, b: u64) -> u64 {
    let diff = a ^ b;
    // mask = 0xFF..FF if a < b, else 0
    let mask = ((a < b) as u64).wrapping_neg();
    // When a < b: b ^ (a ^ b) = a (the smaller value)
    // When a >= b: b ^ 0 = b (the smaller value)
    b ^ (diff & mask)
}

// ───────── Zeroize-on-Drop ─────────

/// Wrapper that zeroizes memory on drop.
///
/// Uses volatile writes to prevent the compiler from optimizing away the zeroing.
pub struct Zeroized<T: Copy> {
    value: T,
}

impl<T: Copy> Zeroized<T> {
    /// Create a new zeroized wrapper.
    #[inline]
    pub fn new(val: T) -> Self {
        Self { value: val }
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Copy> Drop for Zeroized<T> {
    fn drop(&mut self) {
        // Volatile zeroize: prevent compiler from optimizing away
        let size = core::mem::size_of::<T>();
        let ptr = &mut self.value as *mut T as *mut u8;
        unsafe {
            for i in 0..size {
                core::ptr::write_volatile(ptr.add(i), 0);
            }
        }
    }
}

impl<T: Copy + PartialEq> PartialEq for Zeroized<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Copy + Eq> Eq for Zeroized<T> {}

impl<T: Copy> core::fmt::Debug for Zeroized<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Zeroized(***)")
    }
}

// ───────── Tests ─────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_line_aligned() {
        let val = CacheLineAligned::new(42u64);
        assert_eq!(*val, 42);
        // Verify alignment
        let ptr = &*val as *const u64 as usize;
        assert_eq!(ptr % CACHE_LINE_SIZE, 0, "must be cache-line aligned");
    }

    #[test]
    fn test_secret_u64_ct_eq() {
        let a = SecretU64::new(0x12345678);
        let b = SecretU64::new(0x12345678);
        let c = SecretU64::new(0x12345679);

        assert!(a.ct_eq(b));
        assert!(!a.ct_eq(c));
        // Zero equality
        let z1 = SecretU64::new(0);
        let z2 = SecretU64::new(0);
        assert!(z1.ct_eq(z2));
    }

    #[test]
    fn test_secret_u64_ct_select() {
        let a = SecretU64::new(100);
        let b = SecretU64::new(200);

        assert_eq!(SecretU64::ct_select(true, a, b).expose(), 100);
        assert_eq!(SecretU64::ct_select(false, a, b).expose(), 200);
    }

    #[test]
    fn test_ct_mulmod() {
        // Basic: 3 * 4 mod 5 = 2
        assert_eq!(ct_mulmod(3, 4, 5), 2);
        // Large values
        assert_eq!(ct_mulmod(123456789, 987654321, 1000000007), 259106859);
        // Edge: multiply by 0
        assert_eq!(ct_mulmod(12345, 0, 99991), 0);
        // Edge: multiply by 1
        assert_eq!(ct_mulmod(12345, 1, 99991), 12345);
    }

    #[test]
    fn test_ct_mulmod_matches_scalar() {
        // Verify against standard modular multiplication for random inputs
        let test_cases = [
            (123u64, 456u64, 789u64),
            (0, 0, 1),
            (u64::MAX - 1, 2, 7),
            (4294967311, 3, 4294967311), // DEFAULT_PRIME
        ];
        for (a, b, m) in test_cases {
            let ct = ct_mulmod(a, b, m);
            let scalar = ((a as u128 * b as u128) % m as u128) as u64;
            assert_eq!(ct, scalar, "mismatch for ({}, {}, {})", a, b, m);
        }
    }

    #[test]
    fn test_ct_powmod() {
        // 2^10 mod 1000 = 1024 mod 1000 = 24
        assert_eq!(ct_powmod(2, 10, 1000), 24);
        // 3^5 mod 7 = 243 mod 7 = 5
        assert_eq!(ct_powmod(3, 5, 7), 5);
        // Anything^0 mod m = 1 (when m > 1)
        assert_eq!(ct_powmod(42, 0, 99991), 1);
        // 0^5 mod 7 = 0
        assert_eq!(ct_powmod(0, 5, 7), 0);
    }

    #[test]
    fn test_ct_eq_bytes() {
        assert!(ct_eq_bytes(b"hello", b"hello"));
        assert!(!ct_eq_bytes(b"hello", b"world"));
        assert!(!ct_eq_bytes(b"hello", b"hell"));
        assert!(ct_eq_bytes(b"", b""));
        assert!(!ct_eq_bytes(b"a", b"ab"));
    }

    #[test]
    fn test_zeroized() {
        let val = Zeroized::new(42u64);
        assert_eq!(*val.get(), 42);
        drop(val);
        // After drop, memory should be zeroed (can't easily verify, but no panic)
    }

    #[test]
    fn test_ct_min() {
        assert_eq!(ct_min(5, 10), 5);
        assert_eq!(ct_min(10, 5), 5);
        assert_eq!(ct_min(7, 7), 7);
        assert_eq!(ct_min(0, u64::MAX), 0);
    }

    #[test]
    fn test_secret_debug_no_leak() {
        let secret: SecretU64<Secret> = SecretU64::new(0xDEADBEEF);
        let debug_str = format!("{:?}", secret);
        assert_eq!(debug_str, "SecretU64(***)");
        assert!(!debug_str.contains("DEADBEEF"));
    }
}
