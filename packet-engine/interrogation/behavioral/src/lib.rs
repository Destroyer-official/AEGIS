//! Behavioral fingerprinting suite for AEGIS Phase 2 interrogation.
//!
//! §6.4 — Behavioral Fingerprinting Suite
//!
//! This crate provides four behavioral analysis modules:
//!
//! - **SSH** (`ssh`): Key-exchange preferences, host key types, and
//!   `SSH_MSG_UNIMPLEMENTED` response framing analysis
//! - **LLM** (`llm`): Refusal-vector behavioral fingerprinting via
//!   cosine similarity over standardized prompt responses
//! - **Website Fingerprinting** (`wf`): Encrypted traffic pattern
//!   classification (packet size/timing/sequence)
//! - **Bot Score** (`bot_score`): TLS anomaly detection and automated
//!   client identification from JA4 fingerprints
//!
//! All parsers operate on `&[u8]` slices with zero allocations in the hot path.

pub mod bot_score;
pub mod cluster;
pub mod drift;
pub mod llm;
pub mod registry;
pub mod ssh;
pub mod wf;

pub use bot_score::{BotScore, TlsAnomaly};
pub use cluster::{Cluster, SshClusterer, SshFeatureVector, ThreatActorCluster};
pub use drift::{ks_two_sample, DriftAlert, DriftConfig, DriftDetector, KsTestResult};
pub use llm::{ModelFamily, RefusalVector};
pub use registry::{
    CanaryConfig, ModelRegistry, ModelStatus, ModelVersion, RegistrySummary, RoutingDecision,
};
pub use ssh::{SshBanner, SshBehavioralFingerprint, SshKexInit};
pub use wf::{WfClassifier, WfFeatureVector, WfPrediction};
