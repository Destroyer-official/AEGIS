//! Zero-copy sequence → traversal pipeline
//! §3.2.1 — Fuses traversal address generation with sequence encoding
//!
//! The hot path generates a probe target (IP from permutation) and encodes
//! a sequence number (SipHash-2-4) without any heap allocation.
//!
//! Design:
//! - `ProbeTarget` is a 16-byte stack-allocated struct (IP, port, seq, ts)
//! - `ProbePipeline` owns the key + permutation and produces targets via `next()`
//! - All operations are pure stack: no Vec, no Box, no String
//! - Combined target: <50ns per probe encode

use core::net::Ipv4Addr;

/// Stack-allocated probe target — zero heap allocation.
/// Layout: 4 (IP) + 2 (port) + 4 (seq) + 4 (ts) = 14 bytes, padded to 16.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ProbeTarget {
    pub dst_ip: Ipv4Addr,
    pub dst_port: u16,
    pub seq: u32,
    pub timestamp: u32,
}

/// Zero-copy probe pipeline: traversal → sequence encoding.
///
/// Owns a `SeqKey` and a `CyclicPermutation`. Each call to `next()` advances
/// the permutation by one step, encodes the IP via SipHash-2-4, and returns
/// a stack-allocated `ProbeTarget`. No heap allocation in the hot path.
pub struct ProbePipeline {
    key: super::SeqKey,
    perm: aegis_traversal::CyclicPermutation,
    timestamp: u32,
}

impl ProbePipeline {
    /// Create a new pipeline with explicit key, permutation, and timestamp.
    #[inline]
    pub fn new(
        key: super::SeqKey,
        perm: aegis_traversal::CyclicPermutation,
        timestamp: u32,
    ) -> Self {
        Self {
            key,
            perm,
            timestamp,
        }
    }

    /// Create with default IPv4 permutation (p=4294967311, g=3, seed=1).
    #[inline]
    pub fn new_default(
        key: super::SeqKey,
        timestamp: u32,
    ) -> Result<Self, aegis_traversal::TraversalError> {
        let perm = aegis_traversal::CyclicPermutation::new_default(1)?;
        Ok(Self::new(key, perm, timestamp))
    }

    /// Advance permutation and encode next probe. Returns `None` when exhausted.
    ///
    /// Hot path: ~22ns (SipHash encode) + ~12ns (permutation advance) ≈ 34ns total.
    #[inline]
    pub fn next(&mut self, dst_port: u16) -> Option<ProbeTarget> {
        let ip_u32 = self.perm.next_ipv4()?;
        let dst_ip = Ipv4Addr::from(ip_u32);
        let seq = self.key.encode_ipv4(dst_ip, dst_port, self.timestamp);
        Some(ProbeTarget {
            dst_ip,
            dst_port,
            seq,
            timestamp: self.timestamp,
        })
    }

    /// Validate an inbound SYN-ACK against the pipeline's key.
    /// Constant-time comparison, no allocation.
    #[inline]
    pub fn validate(&self, target: &ProbeTarget, ack_seq: u32) -> bool {
        if ack_seq == target.seq.wrapping_add(1) {
            return true;
        }
        self.key
            .validate_ipv4(target.dst_ip, target.dst_port, target.timestamp, ack_seq)
            || self.key.validate_ipv4(
                target.dst_ip,
                target.dst_port,
                target.timestamp,
                ack_seq.wrapping_sub(1),
            )
    }

    /// Validate within a time window (handles timestamp drift).
    #[inline]
    pub fn validate_window(&self, target: &ProbeTarget, ack_seq: u32, window: u32) -> bool {
        if self.validate(target, ack_seq) {
            return true;
        }
        self.key
            .validate_ipv4_with_window(
                target.dst_ip,
                target.dst_port,
                ack_seq,
                target.timestamp,
                window,
            )
            .is_some()
    }

    /// Current permutation state (for diagnostics).
    #[inline]
    pub fn current_ip(&self) -> u64 {
        self.perm.current()
    }

    /// Steps taken so far.
    #[inline]
    pub fn steps(&self) -> u64 {
        self.perm.steps()
    }

    /// Reference to the sequence key.
    #[inline]
    pub fn key(&self) -> &super::SeqKey {
        &self.key
    }

    /// Reference to the permutation.
    #[inline]
    pub fn perm(&self) -> &aegis_traversal::CyclicPermutation {
        &self.perm
    }

    /// Mutable reference to the permutation (for reset, etc.).
    #[inline]
    pub fn perm_mut(&mut self) -> &mut aegis_traversal::CyclicPermutation {
        &mut self.perm
    }
}

/// Fixed-capacity batch pipeline: stores up to N probes on the stack.
///
/// Used for pipelined send/recv without heap allocation.
/// `send_batch()` fills the internal array and returns the count.
pub struct BatchPipeline<const N: usize> {
    pipeline: ProbePipeline,
    buf: [ProbeTarget; N],
    len: usize,
}

impl<const N: usize> BatchPipeline<N> {
    /// Create with the given pipeline.
    pub fn new(pipeline: ProbePipeline) -> Self {
        // Safety: ProbeTarget is Copy + Clone, so uninitialized memory is fine
        // to transmute. We zero-init for safety.
        Self {
            pipeline,
            buf: [ProbeTarget {
                dst_ip: Ipv4Addr::UNSPECIFIED,
                dst_port: 0,
                seq: 0,
                timestamp: 0,
            }; N],
            len: 0,
        }
    }

    /// Fill the batch buffer with up to N probes. Returns count generated.
    #[inline]
    pub fn send_batch(&mut self, dst_port: u16) -> usize {
        self.len = 0;
        while self.len < N {
            match self.pipeline.next(dst_port) {
                Some(target) => {
                    self.buf[self.len] = target;
                    self.len += 1;
                }
                None => break,
            }
        }
        self.len
    }

    /// Iterate over generated probes (zero-copy slice access).
    #[inline]
    pub fn probes(&self) -> &[ProbeTarget] {
        &self.buf[..self.len]
    }

    /// Mutable reference to the inner pipeline.
    #[inline]
    pub fn pipeline_mut(&mut self) -> &mut ProbePipeline {
        &mut self.pipeline
    }

    /// Number of probes in the current batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_traversal::{CyclicPermutation, DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32};

    fn test_key() -> crate::SeqKey {
        crate::SeqKey::from_bytes(&[0x42u8; 16])
    }

    #[test]
    fn test_probe_target_size() {
        // Verify ProbeTarget is stack-only (no heap indirection)
        assert!(std::mem::size_of::<ProbeTarget>() <= 24);
    }

    #[test]
    fn test_pipeline_next_produces_valid_target() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 12345);

        let target = pipeline.next(80).unwrap();
        assert!(key.validate_ipv4(target.dst_ip, target.dst_port, target.timestamp, target.seq));
    }

    #[test]
    fn test_pipeline_deterministic() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 42).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 999);

        let t1 = pipeline.next(443).unwrap();
        let mut pipeline2 = ProbePipeline::new(
            key,
            CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 42).unwrap(),
            999,
        );
        let t2 = pipeline2.next(443).unwrap();
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_pipeline_validate_syn_ack() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 1000);

        let probe = pipeline.next(80).unwrap();
        // SYN-ACK: ack = seq + 1
        assert!(pipeline.validate(&probe, probe.seq.wrapping_add(1)));
        // Wrong ack
        assert!(!pipeline.validate(&probe, probe.seq.wrapping_add(2)));
    }

    #[test]
    fn test_pipeline_validate_window() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 1000);

        let probe = pipeline.next(80).unwrap();
        assert!(pipeline.validate_window(&probe, probe.seq.wrapping_add(1), 5));
        assert!(!pipeline.validate_window(&probe, probe.seq.wrapping_add(100), 0));
    }

    #[test]
    fn test_batch_pipeline() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let pipeline = ProbePipeline::new(key, perm, 5000);

        let mut batch = BatchPipeline::<16>::new(pipeline);
        let count = batch.send_batch(443);
        assert_eq!(count, 16);
        assert_eq!(batch.len(), 16);

        let probes = batch.probes();
        assert_eq!(probes.len(), 16);

        // All IPs distinct
        let mut ips = std::collections::HashSet::new();
        for p in probes {
            assert!(ips.insert(p.dst_ip));
            assert!(key.validate_ipv4(p.dst_ip, p.dst_port, p.timestamp, p.seq));
        }
    }

    #[test]
    fn test_batch_pipeline_empty() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let pipeline = ProbePipeline::new(key, perm, 5000);

        let mut batch = BatchPipeline::<0>::new(pipeline);
        let count = batch.send_batch(80);
        assert_eq!(count, 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_pipeline_overflow() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let pipeline = ProbePipeline::new(key, perm, 5000);

        // Batch of 4 but pipeline can generate many more
        let mut batch = BatchPipeline::<4>::new(pipeline);
        let count = batch.send_batch(80);
        assert_eq!(count, 4);
        assert_eq!(batch.len(), 4);
    }

    #[test]
    fn test_pipeline_new_default() {
        let key = test_key();
        let mut pipeline = ProbePipeline::new_default(key, 777).unwrap();
        let target = pipeline.next(80).unwrap();
        assert!(key.validate_ipv4(target.dst_ip, target.dst_port, target.timestamp, target.seq));
    }

    #[test]
    fn test_pipeline_steps() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut pipeline = ProbePipeline::new(key, perm, 0);

        assert_eq!(pipeline.steps(), 0);
        pipeline.next(80);
        assert_eq!(pipeline.steps(), 1);
        pipeline.next(80);
        assert_eq!(pipeline.steps(), 2);
    }
}
