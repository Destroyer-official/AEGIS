//! Wiring between AF_XDP backend and Dev 2's packet core (§3.2.1)
//!
//! Demonstrates:
//! - Secret-keyed sequence number encoding (`SeqKey::encode_ipv4` → TCP seq)
//! - Address traversal randomization (`CyclicPermutation::next_ipv4`)
//! - Pipelined send/recv with recv-key registration before send (race-free)
//! - 16-shard concurrent skeleton

use std::collections::HashSet;
use std::net::Ipv4Addr;

use aegis_sequence::SeqKey;
use aegis_traversal::{CyclicPermutation, PermutationShard, ShardSet};

/// Probe target with zero-state encoded sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeTarget {
    pub dst_ip: Ipv4Addr,
    pub dst_port: u16,
    pub seq: u32,
    pub timestamp: u32,
}

/// Stateless probe encoder: combines traversal + sequencing.
#[derive(Debug)]
pub struct ProbeEncoder {
    seq_key: SeqKey,
    perm: CyclicPermutation,
    timestamp: u32,
}

impl ProbeEncoder {
    /// Create with secret key, permutation, and initial timestamp.
    pub fn new(seq_key: SeqKey, perm: CyclicPermutation, timestamp: u32) -> Self {
        Self {
            seq_key,
            perm,
            timestamp,
        }
    }

    /// Create with default traversal (p=4294967311, g=3, seed=1)
    pub fn new_default(
        seq_key: SeqKey,
        timestamp: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let perm = CyclicPermutation::new_default(1)?;
        Ok(Self::new(seq_key, perm, timestamp))
    }

    /// Next probe: traverse to next IP and encode seq = SipHash(key, ip, port, ts)
    /// Returns None when permutation is exhausted.
    pub fn next_probe(&mut self, dst_port: u16) -> Option<ProbeTarget> {
        let ip_u32 = self.perm.next_ipv4()?;
        let dst_ip = Ipv4Addr::from(ip_u32);
        let seq = self.seq_key.encode_ipv4(dst_ip, dst_port, self.timestamp);
        Some(ProbeTarget {
            dst_ip,
            dst_port,
            seq,
            timestamp: self.timestamp,
        })
    }

    /// Validate an inbound SYN-ACK: check seq matches expected for (ip,port,ts)
    /// Constant-time comparison.
    pub fn validate_response(
        &self,
        src_ip: Ipv4Addr,
        src_port: u16,
        ack_seq: u32,
        timestamp: u32,
    ) -> bool {
        self.seq_key
            .validate_ipv4(src_ip, src_port, timestamp, ack_seq)
            || self
                .seq_key
                .validate_ipv4(src_ip, src_port, timestamp, ack_seq.wrapping_sub(1))
    }

    /// Validate within window (handles timestamp drift)
    pub fn validate_with_window(
        &self,
        src_ip: Ipv4Addr,
        src_port: u16,
        ack_seq: u32,
        window: u32,
    ) -> Option<u32> {
        if let Some(ts) = self.seq_key.validate_ipv4_with_window(
            src_ip,
            src_port,
            ack_seq,
            self.timestamp,
            window,
        ) {
            return Some(ts);
        }
        let ack_orig = ack_seq.wrapping_sub(1);
        self.seq_key
            .validate_ipv4_with_window(src_ip, src_port, ack_orig, self.timestamp, window)
    }

    pub fn perm_mut(&mut self) -> &mut CyclicPermutation {
        &mut self.perm
    }

    pub fn seq_key(&self) -> &SeqKey {
        &self.seq_key
    }

    pub fn set_timestamp(&mut self, ts: u32) {
        self.timestamp = ts;
    }
}

/// Pipelined send/recv tracker demonstrating "recv-key registration before send"
/// to eliminate the `reply landed before we knew to listen` race.
///
/// In real engine, the RX thread registers expected seq keys into a filter
/// (eBPF map or userspace HashSet) before the TX thread sends the probe.
#[derive(Debug)]
pub struct PipelinedTracker {
    seq_key: SeqKey,
    /// Set of pending expected sequence numbers (for demo; real engine uses zero-state)
    pending: HashSet<u32>,
}

impl PipelinedTracker {
    pub fn new(seq_key: SeqKey) -> Self {
        Self {
            seq_key,
            pending: HashSet::new(),
        }
    }

    /// Register expected response *before* sending probe (race-free).
    pub fn register_before_send(&mut self, dst_ip: Ipv4Addr, dst_port: u16, timestamp: u32) -> u32 {
        let seq = self.seq_key.encode_ipv4(dst_ip, dst_port, timestamp);
        self.pending.insert(seq);
        self.pending.insert(seq.wrapping_add(1));
        seq
    }

    /// Check inbound packet's ack/seq against pending set (constant-time fallback to SipHash validate)
    pub fn is_expected(
        &self,
        src_ip: Ipv4Addr,
        src_port: u16,
        ack_seq: u32,
        timestamp: u32,
    ) -> bool {
        if self.pending.contains(&ack_seq) {
            return self
                .seq_key
                .validate_ipv4(src_ip, src_port, timestamp, ack_seq)
                || self.seq_key.validate_ipv4(
                    src_ip,
                    src_port,
                    timestamp,
                    ack_seq.wrapping_sub(1),
                );
        }
        false
    }

    /// Remove completed (called after recv)
    pub fn complete(&mut self, seq: u32) {
        self.pending.remove(&seq);
        self.pending.remove(&seq.wrapping_add(1));
        self.pending.remove(&seq.wrapping_sub(1));
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// 16-shard engine skeleton that owns shards and seq keys.
/// Each shard implements `Shard { fn send(); fn recv(); }` and
/// internally advances its own permutation and seq encoding.
pub struct ShardedProbeEngine {
    shards: ShardSet,
    seq_key: SeqKey,
    timestamp: u32,
    dst_port: u16,
}

impl ShardedProbeEngine {
    /// Create 16 shards with default prime/generator.
    pub fn new_default(
        seq_key: SeqKey,
        timestamp: u32,
        dst_port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let shards = ShardSet::new_default(16)?;
        Ok(Self {
            shards,
            seq_key,
            timestamp,
            dst_port,
        })
    }

    pub fn shards(&self) -> &[PermutationShard] {
        self.shards.shards()
    }

    pub fn shards_mut(&mut self) -> &mut [PermutationShard] {
        self.shards.shards_mut()
    }

    /// Drive one send round across all shards: each shard advances permutation and encodes seq.
    /// Returns vector of probes (for demo).
    pub fn send_all(&mut self) -> Vec<ProbeTarget> {
        let mut probes = Vec::with_capacity(self.shards.len());
        for shard in self.shards.shards_mut() {
            if let Some(ip_u32) = shard.next_target() {
                let dst_ip = Ipv4Addr::from(ip_u32);
                let seq = self
                    .seq_key
                    .encode_ipv4(dst_ip, self.dst_port, self.timestamp);
                probes.push(ProbeTarget {
                    dst_ip,
                    dst_port: self.dst_port,
                    seq,
                    timestamp: self.timestamp,
                });
            }
        }
        probes
    }

    /// Drive recv (stub).
    pub fn recv_all(&mut self) -> usize {
        self.shards.recv_all()
    }
}

// ───────── AF_XDP wiring helpers ─────────

/// Build a minimal TCP SYN packet bytes with encoded seq.
/// For wire demonstration only—real implementation would use pnet raw sockets.
pub fn build_syn_packet(
    dst_ip: Ipv4Addr,
    dst_port: u16,
    src_ip: Ipv4Addr,
    src_port: u16,
    seq: u32,
) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(40);
    pkt.extend_from_slice(&[0x45, 0x00, 0x00, 0x28]);
    pkt.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x40, 0x06]);
    pkt.extend_from_slice(&[0x00, 0x00]);
    pkt.extend_from_slice(&src_ip.octets());
    pkt.extend_from_slice(&dst_ip.octets());
    pkt.extend_from_slice(&src_port.to_be_bytes());
    pkt.extend_from_slice(&dst_port.to_be_bytes());
    pkt.extend_from_slice(&seq.to_be_bytes());
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt.extend_from_slice(&[0x50, 0x02, 0xFF, 0xFF]);
    pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    pkt
}

/// Validate inbound SYN-ACK's ack number against expected seq.
/// TCP SYN-ACK: ack = seq + 1
pub fn validate_syn_ack(seq_key: &SeqKey, probe: &ProbeTarget, ack: u32) -> bool {
    if ack == probe.seq.wrapping_add(1) {
        return seq_key.validate_ipv4(probe.dst_ip, probe.dst_port, probe.timestamp, probe.seq);
    }
    seq_key.validate_ipv4(probe.dst_ip, probe.dst_port, probe.timestamp, ack)
        || seq_key.validate_ipv4(
            probe.dst_ip,
            probe.dst_port,
            probe.timestamp,
            ack.wrapping_sub(1),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_traversal::{DEFAULT_GENERATOR_G32, DEFAULT_PRIME_P32};

    fn test_key() -> SeqKey {
        SeqKey::from_bytes(&[0x11u8; 16])
    }

    #[test]
    fn test_probe_encoder_next() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 1).unwrap();
        let mut enc = ProbeEncoder::new(key, perm, 12345);
        let p1 = enc.next_probe(80).unwrap();
        let p2 = enc.next_probe(80).unwrap();
        assert_ne!(p1.dst_ip, p2.dst_ip);
        assert!(enc.validate_response(p1.dst_ip, p1.dst_port, p1.seq, p1.timestamp));
        assert!(enc.validate_response(
            p1.dst_ip,
            p1.dst_port,
            p1.seq.wrapping_add(1),
            p1.timestamp
        ));
    }

    #[test]
    fn test_pipelined_tracker_register_before_send() {
        let key = test_key();
        let mut tracker = PipelinedTracker::new(key);
        let ip = Ipv4Addr::new(1, 2, 3, 4);
        let seq = tracker.register_before_send(ip, 80, 1000);
        assert_eq!(tracker.pending_count(), 2);
        assert!(tracker.is_expected(ip, 80, seq, 1000));
        assert!(tracker.is_expected(ip, 80, seq.wrapping_add(1), 1000));
        assert!(!tracker.is_expected(ip, 80, seq.wrapping_add(2), 1000));
        assert!(!tracker.is_expected(Ipv4Addr::new(1, 2, 3, 5), 80, seq, 1000));
        tracker.complete(seq);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_sharded_engine() {
        let key = test_key();
        let mut engine = ShardedProbeEngine::new_default(key, 999, 443).unwrap();
        let probes = engine.send_all();
        assert_eq!(probes.len(), 16);
        let mut ips = std::collections::HashSet::new();
        for p in probes {
            assert!(ips.insert(p.dst_ip), "duplicate ip across shards");
            assert!(key.validate_ipv4(p.dst_ip, p.dst_port, p.timestamp, p.seq));
        }
        let recv = engine.recv_all();
        assert_eq!(recv, 16);
    }

    #[test]
    fn test_build_syn_packet_embeds_seq() {
        let key = test_key();
        let dst = Ipv4Addr::new(8, 8, 8, 8);
        let seq = key.encode_ipv4(dst, 80, 0);
        let pkt = build_syn_packet(dst, 80, Ipv4Addr::new(192, 168, 1, 1), 12345, seq);
        let embedded = u32::from_be_bytes(pkt[24..28].try_into().unwrap());
        assert_eq!(embedded, seq);
    }

    #[test]
    fn test_validate_syn_ack() {
        let key = test_key();
        let perm = CyclicPermutation::new(DEFAULT_PRIME_P32, DEFAULT_GENERATOR_G32, 42).unwrap();
        let mut enc = ProbeEncoder::new(key, perm, 555);
        let probe = enc.next_probe(8080).unwrap();
        assert!(validate_syn_ack(&key, &probe, probe.seq.wrapping_add(1)));
        assert!(!validate_syn_ack(&key, &probe, probe.seq.wrapping_add(2)));
        let mut fake = probe;
        fake.dst_ip = Ipv4Addr::new(9, 9, 9, 9);
        assert!(!validate_syn_ack(&key, &fake, probe.seq.wrapping_add(1)));
    }

    #[test]
    fn test_wire_integration_loop() {
        let key = SeqKey::from_bytes(&[0xAA; 16]);
        let mut enc = ProbeEncoder::new_default(key, 1000).unwrap();
        let mut tracker = PipelinedTracker::new(key);
        for _ in 0..100 {
            let probe = enc.next_probe(80).unwrap();
            let seq = tracker.register_before_send(probe.dst_ip, probe.dst_port, probe.timestamp);
            assert_eq!(seq, probe.seq);
            let ack = probe.seq.wrapping_add(1);
            assert!(tracker.is_expected(probe.dst_ip, probe.dst_port, ack, probe.timestamp));
            assert!(enc.validate_response(probe.dst_ip, probe.dst_port, ack, probe.timestamp));
        }
    }
}
