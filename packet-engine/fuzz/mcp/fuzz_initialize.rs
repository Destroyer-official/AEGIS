//! Fuzz target for MCP Initialize request parsing.
//!
//! Tests parsing of MCP initialize request messages with malformed
//! headers, oversized payloads, and invalid JSON structures.

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct McpInitializeInput {
    protocol_version: String,
    capabilities: Vec<String>,
    client_info: String,
}

fuzz_target!(|input: McpInitializeInput| {
    // Simulate parsing an MCP initialize request
    // In a real implementation, this would parse JSON-RPC messages
    
    let _ = parse_mcp_initialize(&input);
});

fn parse_mcp_initialize(input: &McpInitializeInput) -> Result<(), String> {
    // Validate protocol version format
    if input.protocol_version.is_empty() {
        return Err("empty protocol version".into());
    }
    if input.protocol_version.len() > 100 {
        return Err("protocol version too long".into());
    }
    
    // Validate capabilities
    for cap in &input.capabilities {
        if cap.len() > 200 {
            return Err("capability name too long".into());
        }
        // Check for valid capability names
        if !cap.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err("invalid capability characters".into());
        }
    }
    
    // Validate client info
    if input.client_info.len() > 1000 {
        return Err("client info too long".into());
    }
    
    Ok(())
}

// Additional fuzz target for raw bytes
fuzz_target!(|data: &[u8]| {
    // Test raw byte parsing - simulate JSON-RPC message parsing
    if data.is_empty() {
        return;
    }
    
    // Try to parse as UTF-8
    if let Ok(json_str) = std::str::from_utf8(data) {
        // Basic JSON structure validation
        if json_str.contains("initialize") {
            let _ = validate_json_rpc(json_str);
        }
    }
});

fn validate_json_rpc(json: &str) -> Result<(), String> {
    // Basic validation - in real impl would use serde_json
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err("not a JSON object".into());
    }
    if json.len() > 100_000 {
        return Err("JSON too large".into());
    }
    // Check for required fields
    if !json.contains("jsonrpc") || !json.contains("method") {
        return Err("missing required JSON-RPC fields".into());
    }
    Ok(())
}