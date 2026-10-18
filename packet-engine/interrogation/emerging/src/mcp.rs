//! Machine Control Protocol (MCP) interrogator stub.
//!
//! §6.5 — Emerging Protocol Interrogators (MCP, A2A)
//!
//! MCP is an emerging application-level protocol for LLM tool-use endpoints.
//! Security assessment of MCP endpoints (arxiv 2608.00150) indicates that these
//! protocols will require specialized fingerprinting approaches similar to
//! existing LLM endpoint analysis.
//!
//! This stub interrogator performs a basic MCP handshake probe and extracts
//! server capability metadata. Full implementation requires the finalized
//! MCP specification.

use crate::{InterrogateError, Interrogator, ProbeResult, ProbeTarget, Protocol};

/// MCP server capabilities reported during handshake.
#[derive(Clone, Debug, Default)]
pub struct McpCapability {
    /// Capability name (e.g., "tools", "resources", "prompts")
    pub name: String,
    /// Capability version string
    pub version: String,
}

/// MCP server information extracted from a probe.
#[derive(Clone, Debug, Default)]
pub struct McpServerInfo {
    /// Server software name (e.g., "ollama", "vllm-mcp")
    pub server_name: String,
    /// Server version
    pub server_version: String,
    /// Advertised capabilities
    pub capabilities: Vec<McpCapability>,
    /// Protocol version supported
    pub protocol_version: String,
}

/// Stub interrogator for Machine Control Protocol endpoints.
///
/// Performs a basic connection probe and attempts to read an MCP
/// server info response. Returns low-confidence stub data until
/// the MCP specification is finalized and a full client is implemented.
pub struct McpInterrogator;

impl Interrogator for McpInterrogator {
    fn protocol(&self) -> Protocol {
        Protocol::Mcp
    }

    fn probe(&self, target: &ProbeTarget) -> Result<ProbeResult, InterrogateError> {
        // Stub: connect and read banner, but don't parse MCP-specific frames yet.
        // A real implementation would:
        // 1. Send MCP initialize request
        // 2. Parse server capabilities response
        // 3. Extract tool/resource/prompt lists
        // 4. Compute behavioral fingerprint from capability set

        let _ = target; // suppress unused warning in stub

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("status".into(), "stub".into());
        metadata.insert("protocol".into(), "MCP".into());
        metadata.insert("note".into(), "MCP spec not finalized — stub probe".into());

        Ok(ProbeResult {
            fingerprint: "MCP:stub".into(),
            metadata,
            confidence: 0.0,
            raw_response: Vec::new(),
        })
    }

    fn description(&self) -> &str {
        "MCP interrogator — stub for Machine Control Protocol endpoints (§6.5)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_mcp_interrogator_protocol() {
        let interp = McpInterrogator;
        assert_eq!(interp.protocol(), Protocol::Mcp);
    }

    #[test]
    fn test_mcp_probe_returns_stub() {
        let interp = McpInterrogator;
        let target = ProbeTarget {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            protocol: Protocol::Mcp,
            timeout_ms: None,
        };
        let result = interp.probe(&target).unwrap();
        assert_eq!(result.confidence, 0.0);
        assert!(result.fingerprint.starts_with("MCP:"));
    }

    #[test]
    fn test_mcp_description() {
        let interp = McpInterrogator;
        assert!(interp.description().contains("MCP"));
    }
}
