//! Property-based fuzzing for LZR signature matching using proptest.
//! Tests pathological inputs: repeating patterns, overlapping signatures, edge cases.

use aegis_lzr::LzrEngine;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,
        max_shrink_iters: 10000,
        ..ProptestConfig::default()
    })]

    /// Fuzz classify with arbitrary byte sequences
    #[test]
    fn fuzz_classify(data in prop::collection::vec(any::<u8>(), 0..2048)) {
        let engine = LzrEngine::new();
        let _ = engine.classify(&data);
    }

    /// Fuzz classify_all with arbitrary byte sequences
    #[test]
    fn fuzz_classify_all(data in prop::collection::vec(any::<u8>(), 0..2048)) {
        let engine = LzrEngine::new();
        let _ = engine.classify_all(&data);
    }

    /// Fuzz with known protocol prefixes + random suffixes
    #[test]
    fn fuzz_protocol_prefixes(
        prefix in prop::sample::select(vec![
            b"HTTP/1.1 200 OK\r\n".to_vec(),
            b"SSH-2.0-OpenSSH_9.0\r\n".to_vec(),
            b"220 mail.example.com ESMTP\r\n".to_vec(),
            b"+OK POP3 ready\r\n".to_vec(),
            b"* OK IMAP4rev1 ready\r\n".to_vec(),
            vec![0x00, 0x00, 0x00, 0x00, 0x0a, 0x38, 0x2e, 0x30, 0x2e, 0x33, 0x33, 0x00],
            br#"{"version": "0.1.14"}"#.to_vec(),
            b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec(),
        ]),
        suffix in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let mut data = prefix;
        data.extend(suffix);
        let engine = LzrEngine::new();
        let _ = engine.classify(&data);
        let _ = engine.classify_all(&data);
    }

    /// Fuzz with repeating patterns (pathological for Aho-Corasick)
    #[test]
    fn fuzz_repeating_patterns(
        pattern in prop::collection::vec(any::<u8>(), 1..32),
        repetitions in 1..100
    ) {
        let mut data = Vec::new();
        for _ in 0..repetitions {
            data.extend(&pattern);
        }
        let engine = LzrEngine::new();
        let _ = engine.classify(&data);
        let _ = engine.classify_all(&data);
    }

    /// Fuzz with overlapping signature-like patterns
    #[test]
    fn fuzz_overlapping_patterns(
        base in prop::collection::vec(any::<u8>(), 0..512),
        overlap_count in 1..20
    ) {
        // Create inputs that could trigger multiple overlapping signature matches
        let mut data = base;
        for i in 0..overlap_count {
            data.push(i as u8);
            data.extend(b"HTTP/1.1 ");
            data.push(i as u8);
            data.extend(b"SSH-2.0-");
        }
        let engine = LzrEngine::new();
        let _ = engine.classify(&data);
        let _ = engine.classify_all(&data);
    }

    /// Fuzz edge cases: empty, single byte, very large
    #[test]
    fn fuzz_edge_cases(
        case in prop::sample::select(vec![
            vec![],                              // empty
            vec![0x00],                          // single null
            vec![0xFF],                          // single 0xFF
            vec![b'H'; 1],                       // single 'H'
            vec![b'H'; 100],                     // repeating 'H'
            (0..256).map(|i| i as u8).collect(), // all byte values
        ])
    ) {
        let engine = LzrEngine::new();
        let _ = engine.classify(&case);
        let _ = engine.classify_all(&case);
    }
}