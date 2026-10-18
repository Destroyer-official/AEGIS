//! Emerging protocol interrogators and the pluggable `Interrogator` trait.
//!
//! §3.2.2 — Phase 2: Stateful L7 Interrogation
//! §6.5 — Emerging Protocol Interrogators (MCP, A2A)
//!
//! The interrogator pool is designed as a pluggable architecture — 3 to 4 core
//! interrogators at initial deployment (HTTP/S, SSH, gRPC/TLS), with additional
//! interrogators (MCP, A2A) added as the protocol landscape evolves.
//!
//! This crate provides:
//! - The core [`Interrogator`] trait defining the probe interface
//! - Protocol enumerations and probe result types
//! - Stub implementations for MCP and A2A interrogators

pub mod a2a;
pub mod mcp;
pub mod security;

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;

// ─── Protocol Enumeration ────────────────────────────────────────────────────

/// Supported interrogation protocols.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Protocol {
    /// Machine Control Protocol (MCP) for LLM tool-use endpoints
    Mcp,
    /// Agent-to-Agent (A2A) protocol for inter-agent communication
    A2a,
    /// HTTP/HTTPS
    Http,
    /// SSH
    Ssh,
    /// TLS (JA4+ fingerprinting)
    Tls,
    /// gRPC over HTTP/2
    Grpc,
    /// QUIC / HTTP/3
    Quic,
    /// Custom protocol identified by name
    Custom(String),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mcp => write!(f, "MCP"),
            Self::A2a => write!(f, "A2A"),
            Self::Http => write!(f, "HTTP"),
            Self::Ssh => write!(f, "SSH"),
            Self::Tls => write!(f, "TLS"),
            Self::Grpc => write!(f, "gRPC"),
            Self::Quic => write!(f, "QUIC"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

// ─── Probe Target ────────────────────────────────────────────────────────────

/// A target endpoint to be probed by an interrogator.
#[derive(Clone, Debug)]
pub struct ProbeTarget {
    /// Network address of the target
    pub addr: SocketAddr,
    /// Protocol to use for interrogation
    pub protocol: Protocol,
    /// Optional timeout override in milliseconds
    pub timeout_ms: Option<u64>,
}

// ─── Probe Result ────────────────────────────────────────────────────────────

/// Structured result from an interrogation probe.
#[derive(Clone, Debug)]
pub struct ProbeResult {
    /// Protocol-specific fingerprint string
    pub fingerprint: String,
    /// Structured metadata extracted during interrogation
    pub metadata: HashMap<String, String>,
    /// Confidence that the probe correctly identified the service (0.0–1.0)
    pub confidence: f64,
    /// Raw bytes captured during the handshake (if any)
    pub raw_response: Vec<u8>,
}

impl ProbeResult {
    /// Create a low-confidence stub result.
    pub fn stub(protocol: &Protocol) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("status".into(), "stub".into());
        metadata.insert("protocol".into(), protocol.to_string());
        Self {
            fingerprint: format!("{}:stub", protocol),
            metadata,
            confidence: 0.0,
            raw_response: Vec::new(),
        }
    }
}

// ─── Interrogator Error ──────────────────────────────────────────────────────

/// Errors from interrogation probes.
#[derive(Debug, thiserror::Error)]
pub enum InterrogateError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("protocol handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// ─── Core Interrogator Trait ─────────────────────────────────────────────────

/// The core trait for all Phase 2 protocol interrogators.
///
/// Each interrogator performs a complete protocol handshake against a target
/// and extracts structured fingerprint data. Interrogators are designed to be
/// stateless and pluggable — new protocols can be added by implementing this
/// trait without modifying the scanning engine core.
///
/// # Implementors
///
/// - `mcp::McpInterrogator` — Machine Control Protocol endpoints
/// - `a2a::A2aInterrogator` — Agent-to-Agent protocol endpoints
///
/// Future interrogators (HTTP/S, SSH, gRPC, QUIC) should also implement this
/// trait to integrate with the pluggable interrogation pool.
pub trait Interrogator {
    /// The protocol this interrogator handles.
    fn protocol(&self) -> Protocol;

    /// Probe a target endpoint and return structured fingerprint data.
    ///
    /// The interrogator performs a complete protocol handshake, extracts
    /// metadata, and returns a [`ProbeResult`] with confidence score.
    /// A confidence of 0.0 indicates a stub or failed identification.
    fn probe(&self, target: &ProbeTarget) -> Result<ProbeResult, InterrogateError>;

    /// Return a human-readable description of this interrogator.
    fn description(&self) -> &str;
}

// ─── Interrogator Registry ───────────────────────────────────────────────────

/// A registry of available interrogators keyed by protocol.
pub struct InterrogatorRegistry {
    interrogators: Vec<Box<dyn Interrogator>>,
}

impl InterrogatorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            interrogators: Vec::new(),
        }
    }

    /// Register an interrogator.
    pub fn register(&mut self, interrogator: Box<dyn Interrogator>) {
        self.interrogators.push(interrogator);
    }

    /// Find an interrogator matching the given protocol.
    pub fn find(&self, protocol: &Protocol) -> Option<&dyn Interrogator> {
        self.interrogators
            .iter()
            .find(|i| i.protocol() == *protocol)
            .map(|i| i.as_ref())
    }

    /// Return the number of registered interrogators.
    pub fn len(&self) -> usize {
        self.interrogators.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.interrogators.is_empty()
    }
}

impl Default for InterrogatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Re-exports ────────────────────────────────────────────────────────────────

pub use security::{
    A2aSecuritySuite, McpSecuritySuite, SecurityCategory, SecuritySummary, SecurityTestResult,
    SecurityTestVector, Severity,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Mcp.to_string(), "MCP");
        assert_eq!(Protocol::A2a.to_string(), "A2A");
        assert_eq!(Protocol::Custom("foo".into()).to_string(), "foo");
    }

    #[test]
    fn test_probe_result_stub() {
        let result = ProbeResult::stub(&Protocol::Mcp);
        assert_eq!(result.fingerprint, "MCP:stub");
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.metadata.get("status").unwrap(), "stub");
    }

    #[test]
    fn test_registry_lifecycle() {
        let mut reg = InterrogatorRegistry::new();
        assert!(reg.is_empty());

        reg.register(Box::new(mcp::McpInterrogator));
        reg.register(Box::new(a2a::A2aInterrogator));

        assert_eq!(reg.len(), 2);
        assert!(reg.find(&Protocol::Mcp).is_some());
        assert!(reg.find(&Protocol::A2a).is_some());
        assert!(reg.find(&Protocol::Ssh).is_none());
    }
}
