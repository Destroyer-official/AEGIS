//! QUIC protocol handling for AEGIS Phase 2 interrogation (§6.3).
//!
//! This crate implements:
//! - QUIC Initial packet construction for connection establishment
//! - Version Negotiation packet parsing
//! - Transport parameter extraction from unencrypted Initial packets
//!
//! Because QUIC encrypts transport payloads beyond initial connection packets,
//! the interrogator parses unencrypted initial connection headers to extract
//! connection IDs, transport parameters, and cryptographic versions.

pub mod initial;
pub mod transport_params;
pub mod version;

pub use initial::QuicInitialPacket;
pub use transport_params::TransportParams;
pub use version::VersionNegotiation;
