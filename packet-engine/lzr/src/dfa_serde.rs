use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crate::{LzrEngine, LzrError};

/// Serializable snapshot of the LZR engine's compiled DFA state.
///
/// Stores the raw patterns and metadata needed to rebuild the Aho-Corasick
/// automaton at startup. Binary format via `bincode` for fast serialization.
///
/// **Stability**: Unstable (internal format). Do not persist across versions.
#[derive(Serialize, Deserialize)]
pub struct DfaSnapshot {
    /// AC-tier signatures (category, protocol, literal pattern, priority).
    pub ac_sigs: Vec<(String, String, Vec<u8>, u32)>,
    /// Regex-tier signatures (category, protocol, regex string, priority).
    pub regex_sigs: Vec<(String, String, String, u32)>,
    /// Total signature count from original CSV (before AC/regex split).
    pub total_count: usize,
    /// Version marker for forward compatibility.
    pub version: u32,
}

impl DfaSnapshot {
    const CURRENT_VERSION: u32 = 1;
}

/// Serialize the engine's compiled state to a writer.
///
/// Captures:
/// - AC-tier patterns + metadata
/// - Regex-tier patterns + metadata
///
/// The Aho-Corasick automaton itself is NOT serialized — it is rebuilt
/// from the patterns on load (takes <5ms for 445 signatures).
///
/// # Errors
/// Returns `LzrError` on I/O or serialization failure.
pub fn serialize_dfa<W: Write>(engine: &LzrEngine, writer: &mut W) -> Result<(), LzrError> {
    let snapshot = DfaSnapshot {
        ac_sigs: engine
            .ac_sigs
            .iter()
            .map(|s| {
                (
                    s.category.clone(),
                    s.protocol.clone(),
                    s.pattern.clone(),
                    s.priority,
                )
            })
            .collect(),
        regex_sigs: engine
            .regex_sigs
            .iter()
            .map(|s| {
                (
                    s.category.clone(),
                    s.protocol.clone(),
                    s.regex.to_string(),
                    s.priority,
                )
            })
            .collect(),
        version: DfaSnapshot::CURRENT_VERSION,
        total_count: engine.total_count,
    };

    bincode::serialize_into(writer, &snapshot)
        .map_err(|e| LzrError::IoError(std::io::Error::other(e)))?;

    Ok(())
}

/// Deserialize engine state from a reader and rebuild the Aho-Corasick automaton.
///
/// Returns the rebuilt engine and the time taken for deserialization + rebuild.
///
/// # Errors
/// Returns `LzrError` on I/O, deserialization, or regex compilation failure.
pub fn deserialize_dfa<R: Read>(reader: &mut R) -> Result<(LzrEngine, Duration), LzrError> {
    let start = Instant::now();

    let snapshot: DfaSnapshot = bincode::deserialize_from(reader)
        .map_err(|e| LzrError::IoError(std::io::Error::other(e)))?;

    if snapshot.version != DfaSnapshot::CURRENT_VERSION {
        return Err(LzrError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "DFA snapshot version mismatch: expected {}, got {}",
                DfaSnapshot::CURRENT_VERSION,
                snapshot.version
            ),
        )));
    }

    let engine = LzrEngine::with_signatures_from_snapshot(&snapshot)?;
    let elapsed = start.elapsed();

    Ok((engine, elapsed))
}

/// Serialize engine to a byte vector.
pub fn serialize_dfa_bytes(engine: &LzrEngine) -> Result<Vec<u8>, LzrError> {
    let mut buf = Vec::with_capacity(64 * 1024);
    serialize_dfa(engine, &mut buf)?;
    Ok(buf)
}

/// Deserialize engine from a byte slice.
pub fn deserialize_dfa_bytes(data: &[u8]) -> Result<(LzrEngine, Duration), LzrError> {
    let mut cursor = std::io::Cursor::new(data);
    deserialize_dfa(&mut cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AcSignature, RegexSignature};
    use std::io::Cursor;

    #[test]
    fn test_serialize_roundtrip() {
        let engine = LzrEngine::new();
        let mut buf = Vec::new();
        serialize_dfa(&engine, &mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let (engine2, elapsed) = deserialize_dfa(&mut cursor).unwrap();

        assert_eq!(engine.signature_count(), engine2.signature_count());
        let max_ms = if cfg!(debug_assertions) { 500 } else { 50 };
        assert!(
            elapsed.as_millis() < max_ms,
            "DFA load took {:?}, target <{}ms",
            elapsed,
            max_ms
        );

        // Verify classification works
        let result = engine2.classify(b"HTTP/1.1 200 OK\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "HTTP/1.1");
    }

    #[test]
    fn test_serialize_bytes_roundtrip() {
        let engine = LzrEngine::new();
        let bytes = serialize_dfa_bytes(&engine).unwrap();

        // Binary should be reasonable size (< 1MB for 445 signatures)
        assert!(
            bytes.len() < 1_000_000,
            "DFA binary too large: {} bytes",
            bytes.len()
        );

        let (engine2, elapsed) = deserialize_dfa_bytes(&bytes).unwrap();
        assert_eq!(engine.signature_count(), engine2.signature_count());
        let max_ms = if cfg!(debug_assertions) { 500 } else { 50 };
        assert!(
            elapsed.as_millis() < max_ms,
            "DFA load took {:?}, target <{}ms",
            elapsed,
            max_ms
        );
    }

    #[test]
    fn test_dfa_load_time_under_50ms() {
        let engine = LzrEngine::new();
        let bytes = serialize_dfa_bytes(&engine).unwrap();

        // Measure load time over multiple iterations
        let iterations = 10;
        let mut total = std::time::Duration::ZERO;
        for _ in 0..iterations {
            let start = Instant::now();
            let (e, _) = deserialize_dfa_bytes(&bytes).unwrap();
            total += start.elapsed();
            assert_eq!(e.signature_count(), engine.signature_count());
        }
        let avg = total / iterations;

        // Debug builds are slower; target <50ms in release mode
        let max_ms = if cfg!(debug_assertions) { 500 } else { 50 };
        assert!(
            avg.as_millis() < max_ms,
            "Average DFA load time: {:?}, target <{}ms (debug={})",
            avg,
            max_ms,
            cfg!(debug_assertions)
        );
    }

    #[test]
    fn test_dfa_memory_under_100mb() {
        let engine = LzrEngine::new();
        let bytes = serialize_dfa_bytes(&engine).unwrap();

        // The serialized binary should be < 100MB
        assert!(
            bytes.len() < 100 * 1024 * 1024,
            "DFA binary {} bytes exceeds 100MB",
            bytes.len()
        );

        // Estimate in-memory size: patterns + regex strings
        let pattern_mem: usize = engine
            .ac_sigs
            .iter()
            .map(|s| s.pattern.len() + std::mem::size_of::<AcSignature>())
            .sum();
        let regex_mem: usize = engine
            .regex_sigs
            .iter()
            .map(|s| s.regex.as_str().len() + std::mem::size_of::<RegexSignature>())
            .sum();
        let total_mem = pattern_mem + regex_mem;

        assert!(
            total_mem < 100 * 1024 * 1024,
            "In-memory DFA usage {} bytes exceeds 100MB",
            total_mem
        );
    }

    #[test]
    fn test_version_mismatch_rejected() {
        use super::DfaSnapshot;
        use std::io::Cursor;

        let snapshot = DfaSnapshot {
            ac_sigs: vec![],
            regex_sigs: vec![],
            total_count: 0,
            version: 999, // Wrong version
        };
        let mut buf = Vec::new();
        bincode::serialize_into(&mut buf, &snapshot).unwrap();

        let mut cursor = Cursor::new(&buf);
        let result = deserialize_dfa(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_protocols_classifiable_after_deserialize() {
        let engine = LzrEngine::new();
        let bytes = serialize_dfa_bytes(&engine).unwrap();
        let (engine2, _) = deserialize_dfa_bytes(&bytes).unwrap();

        let test_cases: Vec<(&[u8], &str)> = vec![
            (b"HTTP/1.1 200 OK\r\n", "HTTP/1.1"),
            (b"SSH-2.0-OpenSSH_8.9\r\n", "SSH"),
            (b"220 mail.example.com ESMTP\r\n", "SMTP"),
            (b"+OK POP3 ready\r\n", "POP3"),
            (b"* OK [CAPABILITY IMAP4rev1] Server ready\r\n", "IMAP"),
            (b"FATAL database does not exist\r\n", "PostgreSQL"),
        ];

        for (payload, expected_proto) in test_cases {
            let result = engine2.classify(payload);
            assert!(
                result.matched && result.protocol == expected_proto,
                "Failed for {}: got {:?}",
                expected_proto,
                result
            );
        }
    }
}
