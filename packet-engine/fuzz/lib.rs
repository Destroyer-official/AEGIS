//! Fuzz target library for AEGIS interrogation protocols.
//!
//! This library exports fuzz targets for MCP and A2A protocol message parsing.
//! Run with `cargo fuzz run <target_name>` after installing cargo-fuzz:
//!   cargo install cargo-fuzz
//!   cargo fuzz run fuzz_mcp_initialize
//!   cargo fuzz run fuzz_mcp_tools
//!   cargo fuzz run fuzz_a2a_agent_card
//!   cargo fuzz run fuzz_a2a_skills

pub mod mcp {
    pub mod fuzz_initialize;
    pub mod fuzz_tools;
}

pub mod a2a {
    pub mod fuzz_agent_card;
    pub mod fuzz_skills;
}