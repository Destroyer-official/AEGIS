//! Fuzz target for A2A Agent Card parsing.
//!
//! Tests parsing of agent card JSON with malformed fields, oversized
//! payloads, and invalid structures.

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct A2AAgentCardInput {
    name: String,
    description: String,
    version: String,
    protocol_version: String,
    endpoint_url: String,
    skills: Vec<A2ASkillInput>,
}

#[derive(Arbitrary, Debug)]
struct A2ASkillInput {
    id: String,
    name: String,
    description: String,
    mime_types: Vec<String>,
}

fuzz_target!(|input: A2AAgentCardInput| {
    let _ = parse_a2a_agent_card(&input);
});

fn parse_a2a_agent_card(input: &A2AAgentCardInput) -> Result<(), String> {
    // Validate required fields
    if input.name.is_empty() {
        return Err("empty agent name".into());
    }
    if input.name.len() > 200 {
        return Err("agent name too long".into());
    }
    
    if input.description.len() > 10_000 {
        return Err("description too long".into());
    }
    
    // Validate version format (semver-like)
    if input.version.is_empty() || input.version.len() > 50 {
        return Err("invalid version".into());
    }
    
    // Validate protocol version
    if input.protocol_version.is_empty() || input.protocol_version.len() > 50 {
        return Err("invalid protocol version".into());
    }
    
    // Validate endpoint URL
    if input.endpoint_url.is_empty() {
        return Err("empty endpoint URL".into());
    }
    if input.endpoint_url.len() > 2000 {
        return Err("endpoint URL too long".into());
    }
    // Basic URL validation
    if !(input.endpoint_url.starts_with("http://") || input.endpoint_url.starts_with("https://")) {
        return Err("endpoint URL must be HTTP/HTTPS".into());
    }
    
    // Validate skills
    for skill in &input.skills {
        let _ = parse_a2a_skill(skill)?;
    }
    
    Ok(())
}

fn parse_a2a_skill(input: &A2ASkillInput) -> Result<(), String> {
    if input.id.is_empty() {
        return Err("empty skill id".into());
    }
    if input.id.len() > 200 {
        return Err("skill id too long".into());
    }
    if !input.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err("invalid skill id characters".into());
    }
    
    if input.name.is_empty() {
        return Err("empty skill name".into());
    }
    if input.name.len() > 200 {
        return Err("skill name too long".into());
    }
    
    if input.description.len() > 5_000 {
        return Err("skill description too long".into());
    }
    
    // Validate MIME types
    for mime in &input.mime_types {
        if mime.is_empty() || mime.len() > 100 {
            return Err("invalid MIME type".into());
        }
        if !mime.contains('/') {
            return Err("MIME type must contain '/'".into());
        }
    }
    
    Ok(())
}

// Raw bytes fuzz target
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    if let Ok(json_str) = std::str::from_utf8(data) {
        if json_str.contains("agent") || json_str.contains("AgentCard") {
            let _ = validate_agent_card_json(json_str);
        }
    }
});

fn validate_agent_card_json(json: &str) -> Result<(), String> {
    if json.len() > 100_000 {
        return Err("agent card too large".into());
    }
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err("not a JSON object".into());
    }
    // Check for required fields
    if !json.contains("name") || !json.contains("version") {
        return Err("missing required agent card fields".into());
    }
    Ok(())
}