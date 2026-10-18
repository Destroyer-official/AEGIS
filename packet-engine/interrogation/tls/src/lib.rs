//! Zero-copy TLS Client Hello / Server Hello parser and JA4+ fingerprint suite.
//!
//! §6.1 — TLS Fingerprinting — JA4 / JA4+
//!
//! This crate provides:
//! - Zero-copy parsing of TLS 1.0–1.3 Client Hello and Server Hello records
//! - Full JA4+ fingerprint computation: JA4, JA4S, JA4H, JA4T, JA4TS, JA4L, JA4X, JA4SSH, JA4D
//! - GREASE value detection and exclusion
//! - ALPN extraction for protocol identification
//!
//! All parsers operate on `&[u8]` slices with zero allocations in the hot path.

pub mod alpn_policy;
pub mod client_hello;
pub mod ct_monitor;
pub mod grease;
pub mod ja4;
pub mod ja4d;
pub mod ja4h;
pub mod ja4l;
pub mod ja4s;
pub mod ja4ssh;
pub mod ja4t;
pub mod ja4ts;
pub mod ja4x;
pub mod rate_limit;
pub mod record;
pub mod server_hello;

pub use alpn_policy::{AlpnPolicy, AlpnPolicyEngine, PolicyAction};
pub use client_hello::{parse_client_hello, ClientHello};
pub use ct_monitor::{CtAlert, CtLogEntry, CtMonitor, MerkleTreeHead, WatchEntry};
pub use ja4::Ja4Fingerprint;
pub use rate_limit::{RateLimitConfig, RateLimitResult, RateLimiter};
pub use record::{parse_tls_record, ContentType, ProtocolVersion, TlsRecord};
pub use server_hello::{parse_server_hello, ServerHello};
