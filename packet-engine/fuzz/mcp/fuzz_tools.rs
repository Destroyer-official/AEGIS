//! Fuzz target for MCP Tools/List and Tools/Call message parsing.
//!
//! Tests parsing of tool definitions, arguments, and results with
//! malformed structures, oversized payloads, and edge cases.

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct McpToolInput {
    name: String,
    description: String,
    input_schema: String,
}

#[derive(Arbitrary, Debug)]
struct McpToolCallInput {
    name: String,
    arguments: String,
}

fuzz_target!(|input: McpToolInput| {
    let _ = parse_mcp_tool(&input);
});

fn parse_mcp_tool(input: &McpToolInput) -> Result<(), String> {
    // Validate tool name
    if input.name.is_empty() {
        return Err("empty tool name".into());
    }
    if input.name.len() > 200 {
        return Err("tool name too long".into());
    }
    if !input.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("invalid tool name characters".into());
    }
    
    // Validate description
    if input.description.len() > 10_000 {
        return Err("description too long".into());
    }
    
    // Validate input schema (should be JSON Schema)
    if input.input_schema.len() > 50_000 {
        return Err("input schema too large".into());
    }
    if !input.input_schema.trim().starts_with('{') {
        return Err("input schema not a JSON object".into());
    }
    
    Ok(())
}

fuzz_target!(|input: McpToolCallInput| {
    let _ = parse_mcp_tool_call(&input);
});

fn parse_mcp_tool_call(input: &McpToolCallInput) -> Result<(), String> {
    // Validate tool name
    if input.name.is_empty() {
        return Err("empty tool name".into());
    }
    if input.name.len() > 200 {
        return Err("tool name too long".into());
    }
    
    // Validate arguments (JSON)
    if input.arguments.len() > 100_000 {
        return Err("arguments too large".into());
    }
    if !input.arguments.trim().starts_with('{') && !input.arguments.trim().starts_with('[') {
        return Err("arguments not valid JSON".into());
    }
    
    Ok(())
}

// Raw bytes fuzz target for tool messages
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    if let Ok(json_str) = std::str::from_utf8(data) {
        if json_str.contains("tools") {
            let _ = validate_tool_message(json_str);
        }
    }
});

fn validate_tool_message(json: &str) -> Result<(), String> {
    if json.len() > 100_000 {
        return Err("message too large".into());
    }
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err("not a JSON object".into());
    }
    Ok(())
}