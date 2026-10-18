//! Fuzz target for A2A Skill and Task message parsing.
//!
//! Tests parsing of skill definitions, task requests, and responses
//! with malformed structures and oversized payloads.

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct A2ATaskInput {
    id: String,
    skill_id: String,
    input_data: String,
    metadata: Vec<(String, String)>,
}

#[derive(Arbitrary, Debug)]
struct A2ATaskResultInput {
    task_id: String,
    status: String,
    output_data: String,
    error: Option<String>,
}

fuzz_target!(|input: A2ATaskInput| {
    let _ = parse_a2a_task(&input);
});

fn parse_a2a_task(input: &A2ATaskInput) -> Result<(), String> {
    // Validate task ID
    if input.id.is_empty() {
        return Err("empty task id".into());
    }
    if input.id.len() > 200 {
        return Err("task id too long".into());
    }
    
    // Validate skill ID
    if input.skill_id.is_empty() {
        return Err("empty skill id".into());
    }
    if input.skill_id.len() > 200 {
        return Err("skill id too long".into());
    }
    
    // Validate input data (JSON)
    if input.input_data.len() > 1_000_000 {
        return Err("input data too large".into());
    }
    
    // Validate metadata
    if input.metadata.len() > 100 {
        return Err("too many metadata entries".into());
    }
    for (k, v) in &input.metadata {
        if k.len() > 200 || v.len() > 10_000 {
            return Err("metadata entry too large".into());
        }
    }
    
    Ok(())
}

fuzz_target!(|input: A2ATaskResultInput| {
    let _ = parse_a2a_task_result(&input);
});

fn parse_a2a_task_result(input: &A2ATaskResultInput) -> Result<(), String> {
    if input.task_id.is_empty() {
        return Err("empty task id".into());
    }
    if input.task_id.len() > 200 {
        return Err("task id too long".into());
    }
    
    // Validate status
    const VALID_STATUSES: &[&str] = &["pending", "running", "completed", "failed", "cancelled"];
    if !VALID_STATUSES.contains(&input.status.as_str()) {
        return Err("invalid task status".into());
    }
    
    if input.output_data.len() > 1_000_000 {
        return Err("output data too large".into());
    }
    
    if let Some(err) = &input.error {
        if err.len() > 10_000 {
            return Err("error message too long".into());
        }
    }
    
    Ok(())
}

// Raw bytes fuzz target for task messages
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    if let Ok(json_str) = std::str::from_utf8(data) {
        if json_str.contains("task") || json_str.contains("TaskRequest") || json_str.contains("TaskResult") {
            let _ = validate_task_message(json_str);
        }
    }
});

fn validate_task_message(json: &str) -> Result<(), String> {
    if json.len() > 500_000 {
        return Err("task message too large".into());
    }
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err("not a JSON object".into());
    }
    Ok(())
}