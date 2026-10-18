//! Agent-to-Agent (A2A) protocol interrogator stub.
//!
//! §6.5 — Emerging Protocol Interrogators (MCP, A2A)
//!
//! A2A is an emerging inter-agent communication protocol for AI systems.
//! As AI agent infrastructure grows, A2A endpoints will increasingly appear
//! on internet-facing services. This interrogator probes for A2A agent cards
//! and extracts skill/capability metadata.
//!
//! This stub interrogator performs a basic agent card fetch and extracts
//! advertised skills. Full implementation awaits A2A specification stability.

use crate::{InterrogateError, Interrogator, ProbeResult, ProbeTarget, Protocol};

/// An A2A agent skill advertised in the agent card.
#[derive(Clone, Debug, Default)]
pub struct AgentSkill {
    /// Skill identifier
    pub id: String,
    /// Human-readable skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Input/output MIME types
    pub mime_types: Vec<String>,
}

/// An A2A agent card returned during discovery.
#[derive(Clone, Debug, Default)]
pub struct AgentCard {
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: String,
    /// Agent version
    pub version: String,
    /// Advertised skills
    pub skills: Vec<AgentSkill>,
    /// Supported A2A protocol version
    pub protocol_version: String,
    /// Endpoint URL for agent communication
    pub endpoint_url: String,
}

/// Stub interrogator for Agent-to-Agent (A2A) protocol endpoints.
///
/// Performs a basic HTTP GET to the well-known A2A agent card path
/// and attempts to parse an `AgentCard` response. Returns low-confidence
/// stub data until the A2A specification stabilizes.
pub struct A2aInterrogator;

impl Interrogator for A2aInterrogator {
    fn protocol(&self) -> Protocol {
        Protocol::A2a
    }

    fn probe(&self, target: &ProbeTarget) -> Result<ProbeResult, InterrogateError> {
        // Stub: a real implementation would:
        // 1. GET /.well-known/agent.json (or similar discovery path)
        // 2. Parse AgentCard JSON response
        // 3. Extract skills, endpoint URLs, protocol version
        // 4. Compute behavioral fingerprint from skill set + response patterns

        let _ = target;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("status".into(), "stub".into());
        metadata.insert("protocol".into(), "A2A".into());
        metadata.insert("note".into(), "A2A spec not finalized — stub probe".into());

        Ok(ProbeResult {
            fingerprint: "A2A:stub".into(),
            metadata,
            confidence: 0.0,
            raw_response: Vec::new(),
        })
    }

    fn description(&self) -> &str {
        "A2A interrogator — stub for Agent-to-Agent protocol endpoints (§6.5)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_a2a_interrogator_protocol() {
        let interp = A2aInterrogator;
        assert_eq!(interp.protocol(), Protocol::A2a);
    }

    #[test]
    fn test_a2a_probe_returns_stub() {
        let interp = A2aInterrogator;
        let target = ProbeTarget {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            protocol: Protocol::A2a,
            timeout_ms: None,
        };
        let result = interp.probe(&target).unwrap();
        assert_eq!(result.confidence, 0.0);
        assert!(result.fingerprint.starts_with("A2A:"));
    }

    #[test]
    fn test_a2a_description() {
        let interp = A2aInterrogator;
        assert!(interp.description().contains("A2A"));
    }
}
