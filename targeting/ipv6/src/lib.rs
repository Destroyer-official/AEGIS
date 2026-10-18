//! 6Hit RL IPv6 target traversal
//!
//! §3.1.2 — Reinforcement learning agent for IPv6 address space exploration.
//! §3.6.1 — DQN architecture: State (prefix, hit_rate, bgp_path), Action (/64, /120), Reward.
//!
//! The IPv6 address space is partitioned via divisive hierarchical clustering on seed
//! addresses, producing a **space tree** that encodes the search frontier. The DQN agent
//! navigates this tree, selecting subranges for immediate probe dispatch and receiving
//! evaluative feedback (reward) from live scan results.
//!
//! Reference: 6Hit (Hou et al., INFOCOM 2021) — 3.5%–11.5% hit rates.

use core::net::Ipv6Addr;

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

/// Convert IPv6 address octets to u128.
fn octets_to_u128(octets: &[u8; 16]) -> u128 {
    let mut val = 0u128;
    for &b in octets {
        val = (val << 8) | b as u128;
    }
    val
}

// ─────────────────────────────────────────────────────────
// §3.6.1 — DQN State Space
// ─────────────────────────────────────────────────────────

/// Encodes the observation fed to the DQN at each decision step.
///
/// Fields map directly to the 6Hit state space formulation:
/// - **prefix**: the BGP-advertised /48 or /32 enclosing the current region
/// - **hit_rate**: rolling average of live-host discoveries per probe in this region
/// - **bgp_path**: AS-path length and origin ASN for route-level context
/// - **address_density**: fraction of seed addresses falling within this region
/// - **allocation_pattern**: observed allocation granularity (/64, /48, /32 …)
/// - **probe_budget_remaining**: how many probes can still be dispatched this round
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// BGP-advertised prefix (e.g. `2001:db8::/32`).
    pub prefix: Prefix,
    /// Rolling hit rate: live hosts found / probes sent in this region.
    pub hit_rate: f64,
    /// BGP AS-path length (number of AS hops).
    pub as_path_len: u8,
    /// Origin Autonomous System Number.
    pub origin_asn: u32,
    /// Fraction of seed addresses in this region [0.0, 1.0].
    pub address_density: f64,
    /// Dominant allocation granularity observed.
    pub allocation_pattern: AllocGranularity,
    /// Remaining probe budget for the current decision round.
    pub probe_budget_remaining: u32,
}

/// An IPv6 prefix with length (e.g. `2001:db8::/48`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Prefix {
    pub addr: Ipv6Addr,
    pub len: u8,
}

impl Prefix {
    /// Create a new prefix. Panics if `len > 128`.
    pub fn new(addr: Ipv6Addr, len: u8) -> Self {
        assert!(len <= 128, "prefix length must be <= 128");
        Self { addr, len }
    }

    /// Mask the address to its prefix length.
    pub fn masked(&self) -> Ipv6Addr {
        let mut octets = self.addr.octets();
        let full_bytes = (self.len / 8) as usize;
        let remainder = self.len % 8;
        if full_bytes < 16 && remainder > 0 {
            let mask = 0xFFu8 << (8 - remainder);
            octets[full_bytes] &= mask;
            for b in &mut octets[(full_bytes + 1)..] {
                *b = 0;
            }
        } else if full_bytes < 16 {
            for b in &mut octets[full_bytes..] {
                *b = 0;
            }
        }
        Ipv6Addr::from(octets)
    }

    /// Return the number of addresses in this prefix (as u128).
    pub fn size(&self) -> u128 {
        1u128 << (128 - self.len)
    }

    /// Convert to u128 for arithmetic.
    pub fn as_u128(&self) -> u128 {
        octets_to_u128(&self.addr.octets())
    }
}

/// Dominant allocation granularity observed in a region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AllocGranularity {
    /// Individual host addresses (/128).
    Host,
    /// /120 subnets (256 addresses).
    Slash120,
    /// /64 subnets (standard end-host).
    Slash64,
    /// /48 subnets (typical site allocation).
    Slash48,
    /// /32 subnets (typical ISP allocation).
    Slash32,
    /// Unknown or mixed.
    Unknown,
}

// ─────────────────────────────────────────────────────────
// §3.6.1 — DQN Action Space
// ─────────────────────────────────────────────────────────

/// An action the DQN agent can take: select a specific subrange to probe.
///
/// Each action maps to a concrete address range derived from the space tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Probe a /64 subnet (2^64 addresses — standard end-host block).
    ProbeSlash64(Prefix),
    /// Probe a /120 subnet (256 addresses — fine-grained scan).
    ProbeSlash120(Prefix),
    /// Skip this region entirely (no probes).
    Skip,
    /// Trigger space tree repartition for this region.
    Repartition,
}

impl Action {
    /// Return the prefix being targeted, if any.
    pub fn target_prefix(&self) -> Option<&Prefix> {
        match self {
            Action::ProbeSlash64(p) | Action::ProbeSlash120(p) => Some(p),
            Action::Skip | Action::Repartition => None,
        }
    }

    /// Estimated number of probes required for this action.
    pub fn probe_cost(&self) -> u32 {
        match self {
            Action::ProbeSlash64(_) => 1024,
            Action::ProbeSlash120(_) => 256,
            Action::Skip => 0,
            Action::Repartition => 0,
        }
    }
}

// ─────────────────────────────────────────────────────────
// §3.6.1 — DQN Reward Function
// ─────────────────────────────────────────────────────────

/// Evaluative feedback returned after executing an action.
///
/// §3.1.2: "Positive rewards are generated upon discovering live, non-alias,
/// security-relevant hosts. Negative rewards are applied when probing barren
/// subnets or alias patterns."
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reward {
    /// Raw reward value.
    pub value: f64,
    /// Breakdown of what contributed to the reward.
    pub kind: RewardKind,
}

/// Classification of the reward source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardKind {
    /// Discovered live, non-alias hosts.
    LiveHosts,
    /// Region was barren (no live hosts).
    Barren,
    /// Alias pattern detected (single router responding for entire block).
    Alias,
    /// Probe budget exhausted before meaningful result.
    BudgetExhausted,
    /// Repartition triggered (neutral reward).
    Repartition,
}

impl Reward {
    pub fn live(count: u32) -> Self {
        Self {
            value: count as f64,
            kind: RewardKind::LiveHosts,
        }
    }

    pub fn barren() -> Self {
        Self {
            value: -1.0,
            kind: RewardKind::Barren,
        }
    }

    pub fn alias() -> Self {
        Self {
            value: -1.0,
            kind: RewardKind::Alias,
        }
    }

    pub fn budget_exhausted() -> Self {
        Self {
            value: -0.5,
            kind: RewardKind::BudgetExhausted,
        }
    }

    pub fn repartition() -> Self {
        Self {
            value: 0.0,
            kind: RewardKind::Repartition,
        }
    }
}

// ─────────────────────────────────────────────────────────
// §3.1.2 — Space Tree (Divisive Hierarchical Clustering)
// ─────────────────────────────────────────────────────────

/// A node in the space tree produced by divisive hierarchical clustering
/// of seed addresses.
///
/// §3.1.2: "The IPv6 address space is divided into regions using divisive
/// hierarchical clustering on seed addresses, constructing a space tree
/// that represents a partition of the address space."
#[derive(Clone, Debug)]
pub struct SpaceTreeNode {
    /// The prefix this node represents.
    pub prefix: Prefix,
    /// Number of seed addresses falling within this prefix.
    pub seed_count: u32,
    /// Historical hit rate for this region (updated after each scan round).
    pub hit_rate: f64,
    /// Expected reward prior, derived from seed distribution.
    pub expected_reward: f64,
    /// Whether this node has been fully explored (saturated).
    pub saturated: bool,
    /// Children after clustering split (None for leaf nodes).
    pub children: Option<Vec<SpaceTreeNode>>,
}

impl SpaceTreeNode {
    /// Create a leaf node.
    pub fn leaf(prefix: Prefix, seed_count: u32) -> Self {
        Self {
            prefix,
            seed_count,
            hit_rate: 0.0,
            expected_reward: seed_count as f64,
            saturated: false,
            children: None,
        }
    }

    /// Split this node into `k` children by extending the prefix length.
    pub fn split(&mut self, k: usize) {
        let extend = (128 - self.prefix.len).min(k as u8).min(4);
        let child_len = self.prefix.len + extend;
        let step = 1u128 << (128 - child_len);
        let base = octets_to_u128(&self.prefix.masked().octets());
        let mut children = Vec::with_capacity(k);

        for i in 0..k as u128 {
            let child_addr = Ipv6Addr::from(base + i * step);
            children.push(SpaceTreeNode::leaf(Prefix::new(child_addr, child_len), 0));
        }

        self.children = Some(children);
    }

    /// Collect all leaf nodes (regions eligible for action selection).
    pub fn leaves(&self) -> Vec<&SpaceTreeNode> {
        match &self.children {
            Some(children) => children.iter().flat_map(|c| c.leaves()).collect(),
            None => vec![self],
        }
    }

    /// Find the leaf containing a given prefix.
    pub fn find_leaf(&self, prefix: &Prefix) -> Option<&SpaceTreeNode> {
        if self.prefix == *prefix && self.children.is_none() {
            return Some(self);
        }
        if let Some(children) = &self.children {
            for child in children {
                if let Some(found) = child.find_leaf(prefix) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Number of leaves in the tree.
    pub fn leaf_count(&self) -> usize {
        self.leaves().len()
    }

    /// Repartition saturated regions: split saturated leaves into new children.
    pub fn repartition(&mut self) {
        Self::repartition_recursive(self);
    }

    fn repartition_recursive(node: &mut SpaceTreeNode) {
        if let Some(children) = &mut node.children {
            for child in children.iter_mut() {
                Self::repartition_recursive(child);
            }
            // After children are processed, check if any need splitting
        } else if node.saturated && node.prefix.len < 124 {
            node.split(2);
        }
    }
}

// ─────────────────────────────────────────────────────────
// Space Tree Construction from Seeds
// ─────────────────────────────────────────────────────────

/// Build a space tree from a set of seed IPv6 addresses and a root prefix.
pub fn build_space_tree(seeds: &[Ipv6Addr], root: Prefix, split_threshold: u32) -> SpaceTreeNode {
    let mut tree = SpaceTreeNode::leaf(root, seeds.len() as u32);

    for seed in seeds {
        insert_seed(&mut tree, *seed, split_threshold);
    }

    tree
}

fn insert_seed(node: &mut SpaceTreeNode, seed: Ipv6Addr, threshold: u32) {
    if node.children.is_none() {
        node.seed_count += 1;
        if node.seed_count > threshold && node.prefix.len < 124 {
            node.split(2);
        }
        return;
    }

    if let Some(children) = &mut node.children {
        for child in children.iter_mut() {
            let child_masked = child.prefix.masked();
            let seed_masked = apply_prefix_mask(seed, child.prefix.len);
            if child_masked == seed_masked {
                insert_seed(child, seed, threshold);
                return;
            }
        }
    }
}

fn apply_prefix_mask(addr: Ipv6Addr, len: u8) -> Ipv6Addr {
    let mut octets = addr.octets();
    let full_bytes = (len / 8) as usize;
    let remainder = len % 8;
    for b in &mut octets[full_bytes..] {
        *b = 0;
    }
    if remainder > 0 && full_bytes < 16 {
        octets[full_bytes] &= 0xFF << (8 - remainder);
    }
    Ipv6Addr::from(octets)
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_masked() {
        // /48 keeps first 6 bytes: 2001:0db8:0001 → 2001:db8:1::
        let p = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0x0001, 0, 0, 0, 0, 1), 48);
        let masked = p.masked();
        assert_eq!(masked, Ipv6Addr::new(0x2001, 0xdb8, 0x0001, 0, 0, 0, 0, 0));

        // /32 keeps first 4 bytes
        let p32 = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0x0001, 0, 0, 0, 0, 1), 32);
        assert_eq!(p32.masked(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn test_prefix_size() {
        let p = Prefix::new(Ipv6Addr::UNSPECIFIED, 64);
        assert_eq!(p.size(), 1u128 << 64);
    }

    #[test]
    fn test_reward_values() {
        assert_eq!(Reward::live(5).value, 5.0);
        assert_eq!(Reward::barren().value, -1.0);
        assert_eq!(Reward::alias().value, -1.0);
        assert_eq!(Reward::repartition().value, 0.0);
    }

    #[test]
    fn test_action_probe_cost() {
        let p = Prefix::new(Ipv6Addr::LOCALHOST, 64);
        assert_eq!(Action::ProbeSlash64(p).probe_cost(), 1024);
        assert_eq!(Action::ProbeSlash120(p).probe_cost(), 256);
        assert_eq!(Action::Skip.probe_cost(), 0);
    }

    #[test]
    fn test_space_tree_leaf() {
        let p = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32);
        let node = SpaceTreeNode::leaf(p, 42);
        assert_eq!(node.seed_count, 42);
        assert!(node.children.is_none());
        assert_eq!(node.leaves().len(), 1);
    }

    #[test]
    fn test_space_tree_split() {
        let p = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32);
        let mut node = SpaceTreeNode::leaf(p, 100);
        node.split(4);
        assert!(node.children.is_some());
        let children = node.children.as_ref().unwrap();
        assert_eq!(children.len(), 4);
        for child in children {
            assert!(child.children.is_none());
        }
    }

    #[test]
    fn test_space_tree_leaves_collect() {
        let p = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32);
        let mut root = SpaceTreeNode::leaf(p, 0);
        root.split(2);
        if let Some(children) = &mut root.children {
            children[0].split(2);
        }
        let leaves = root.leaves();
        assert_eq!(leaves.len(), 3);
    }

    #[test]
    fn test_build_space_tree() {
        let seeds: Vec<Ipv6Addr> = (0..10)
            .map(|i| Ipv6Addr::new(0x2001, 0xdb8, i as u16, 0, 0, 0, 0, 1))
            .collect();
        let root = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32);
        let tree = build_space_tree(&seeds, root, 5);
        assert!(tree.leaf_count() >= 1);
    }

    #[test]
    fn test_action_target_prefix() {
        let p = Prefix::new(Ipv6Addr::LOCALHOST, 64);
        assert!(Action::ProbeSlash64(p).target_prefix().is_some());
        assert!(Action::Skip.target_prefix().is_none());
    }

    #[test]
    fn test_repartition_saturated() {
        let p = Prefix::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32);
        let mut root = SpaceTreeNode::leaf(p, 0);
        root.split(2);
        if let Some(children) = &mut root.children {
            children[0].saturated = true;
        }
        root.repartition();
        if let Some(children) = &root.children {
            assert!(children[0].children.is_some());
        }
    }

    #[test]
    fn test_octets_to_u128() {
        let addr = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        let val = octets_to_u128(&addr.octets());
        assert_eq!(val, addr.to_bits());
    }
}
