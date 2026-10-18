//! gRPC / HTTP/2 ALPN negotiation, connection preface, and service interrogation (§6.3).
//!
//! This crate implements the gRPC interrogation sequence:
//! 1. TLS Client Hello with ALPN `h2` + `grpc`
//! 2. After TLS handshake completion, send HTTP/2 Connection Preface
//! 3. Send initial SETTINGS frame
//! 4. Send gRPC RPCs (Health Check, Server Reflection)

pub mod alpn;
pub mod frame;
pub mod health;
pub mod http2;
pub mod reflection;
pub mod settings;

pub use alpn::GrpcAlpn;
pub use frame::{decode_grpc_message, encode_grpc_message, DataFrame, HeadersFrame};
pub use health::{build_health_probe, HealthCheckRequest, HealthCheckResponse, ServingStatus};
pub use http2::Http2Preface;
pub use reflection::{build_reflection_probe, decode_list_services};
pub use settings::SettingsFrame;
