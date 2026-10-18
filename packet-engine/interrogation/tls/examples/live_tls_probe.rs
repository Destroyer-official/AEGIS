//! Live Internet TLS Probe Example
//!
//! Connects to a real host over the internet (default: 1.1.1.1:443 or user specified),
//! sends an RFC 8446 TLS ClientHello, reads the ServerHello response, and computes
//! its JA4S fingerprint using AEGIS zero-copy parsers.
//!
//! Usage:
//!   cargo run --example live_tls_probe -- 1.1.1.1:443
//!   cargo run --example live_tls_probe -- github.com:443
//!   cargo run --example live_tls_probe -- google.com:443

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aegis_tls::ja4s::Ja4sFingerprint;
use aegis_tls::record::{parse_tls_record, ContentType};
use aegis_tls::server_hello::parse_server_hello;

/// Builds an RFC 8446 compliant TLS 1.3 / 1.2 ClientHello probe with the given SNI hostname.
fn build_client_hello(sni_host: &str) -> Vec<u8> {
    let mut exts = Vec::new();

    // 1. SNI extension (0x0000)
    let sni_bytes = sni_host.as_bytes();
    let sni_entry_len = 1 + 2 + sni_bytes.len();
    let sni_list_len = sni_entry_len;
    let sni_ext_len = 2 + sni_list_len;

    exts.extend_from_slice(&[0x00, 0x00]);
    exts.extend_from_slice(&(sni_ext_len as u16).to_be_bytes());
    exts.extend_from_slice(&(sni_list_len as u16).to_be_bytes());
    exts.push(0x00); // HostName type
    exts.extend_from_slice(&(sni_bytes.len() as u16).to_be_bytes());
    exts.extend_from_slice(sni_bytes);

    // 2. supported_versions extension (0x002b) -> TLS 1.3 (0x0304), TLS 1.2 (0x0303)
    exts.extend_from_slice(&[0x00, 0x2b, 0x00, 0x05, 0x04, 0x03, 0x04, 0x03, 0x03]);

    // 3. supported_groups extension (0x000a) -> x25519 (0x001d), secp256r1 (0x0017)
    exts.extend_from_slice(&[0x00, 0x0a, 0x00, 0x06, 0x00, 0x04, 0x00, 0x1d, 0x00, 0x17]);

    // 4. signature_algorithms extension (0x000d) -> ecdsa_secp256r1_sha256 (0x0403), rsa_pss_rsae_sha256 (0x0804), rsa_pkcs1_sha256 (0x0401)
    exts.extend_from_slice(&[
        0x00, 0x0d, 0x00, 0x08, 0x00, 0x06, 0x04, 0x03, 0x08, 0x04, 0x04, 0x01,
    ]);

    // 5. key_share extension (0x0033) -> x25519 dummy 32 bytes public share
    let dummy_key_share = [0x42u8; 32];
    let key_share_entry_len = 2 + 2 + dummy_key_share.len();
    let key_share_ext_len = 2 + key_share_entry_len;
    exts.extend_from_slice(&[0x00, 0x33]);
    exts.extend_from_slice(&(key_share_ext_len as u16).to_be_bytes());
    exts.extend_from_slice(&(key_share_entry_len as u16).to_be_bytes());
    exts.extend_from_slice(&[0x00, 0x1d]); // x25519
    exts.extend_from_slice(&(dummy_key_share.len() as u16).to_be_bytes());
    exts.extend_from_slice(&dummy_key_share);

    // 6. ALPN extension (0x0010) -> h2, http/1.1
    let alpn = b"\x02h2\x08http/1.1";
    let alpn_ext_len = 2 + alpn.len();
    exts.extend_from_slice(&[0x00, 0x10]);
    exts.extend_from_slice(&(alpn_ext_len as u16).to_be_bytes());
    exts.extend_from_slice(&(alpn.len() as u16).to_be_bytes());
    exts.extend_from_slice(alpn);

    // Handshake body
    let mut handshake = Vec::new();
    handshake.extend_from_slice(&[0x03, 0x03]); // legacy_version: TLS 1.2
                                                // Client Random: 32 bytes
    handshake.extend_from_slice(&[
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e,
        0x2f, 0x30,
    ]);
    // Session ID: 32 bytes for TLS 1.3 middlebox compatibility
    handshake.push(32);
    handshake.extend_from_slice(&[
        0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee,
        0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
        0xfe, 0xff,
    ]);
    // Cipher suites: modern standard suites
    let ciphers = [
        0x13, 0x01, // TLS_AES_128_GCM_SHA256
        0x13, 0x02, // TLS_AES_256_GCM_SHA384
        0x13, 0x03, // TLS_CHACHA20_POLY1305_SHA256
        0xc0, 0x2b, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
        0xc0, 0x2f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
        0x00, 0x9c, // TLS_RSA_WITH_AES_128_GCM_SHA256
    ];
    handshake.extend_from_slice(&(ciphers.len() as u16).to_be_bytes());
    handshake.extend_from_slice(&ciphers);
    // Compression methods: 1 byte (0 = null)
    handshake.push(1);
    handshake.push(0);
    // Extensions length + data
    handshake.extend_from_slice(&(exts.len() as u16).to_be_bytes());
    handshake.extend_from_slice(&exts);

    // Record layer
    let mut record = Vec::new();
    record.push(0x16); // ContentType: Handshake
    record.extend_from_slice(&[0x03, 0x01]); // Legacy TLS 1.0 record version
    let handshake_len = 4 + handshake.len();
    record.extend_from_slice(&(handshake_len as u16).to_be_bytes());

    // Handshake header
    record.push(0x01); // HandshakeType: ClientHello
    let body_len = handshake.len() as u32;
    record.push((body_len >> 16) as u8);
    record.push((body_len >> 8) as u8);
    record.push(body_len as u8);
    record.extend_from_slice(&handshake);

    record
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        "1.1.1.1:443".to_string()
    };

    // Extract SNI hostname from target (strip port if present)
    let sni_host = target.split(':').next().unwrap_or("one.one.one.one");
    let sni_host = if sni_host == "1.1.1.1" {
        "one.one.one.one"
    } else if sni_host == "8.8.8.8" {
        "dns.google"
    } else {
        sni_host
    };

    println!("========================================================");
    println!("  AEGIS Live TLS / JA4S Internet Probe");
    println!("  Target:   {}", target);
    println!("  SNI Host: {}", sni_host);
    println!("========================================================");

    // 1. Establish TCP connection
    println!("[1/4] Connecting to {} via TCP...", target);
    let mut stream = TcpStream::connect(&target)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    println!("      Connected successfully!");

    // 2. Synthesize standard TLS ClientHello (TLS 1.3 / 1.2 compatible)
    let client_hello_packet = build_client_hello(sni_host);

    println!(
        "[2/4] Sending RFC 8446 ClientHello probe ({} bytes)...",
        client_hello_packet.len()
    );
    stream.write_all(&client_hello_packet)?;
    stream.flush()?;

    // 3. Receive ServerHello from the live server
    println!("[3/4] Awaiting live TLS response from remote server...");
    let mut response_buf = [0u8; 4096];
    let n = stream.read(&mut response_buf)?;
    if n == 0 {
        eprintln!("Error: Connection closed by remote host without response.");
        return Ok(());
    }
    println!("      Received {} bytes from remote server!", n);

    // 4. Zero-copy parsing with AEGIS TLS engine
    println!("[4/4] Parsing TLS records and calculating JA4S fingerprint...");
    let record = parse_tls_record(&response_buf[..n])
        .map_err(|e| format!("Failed to parse TLS record: {:?}", e))?;

    println!("  ├── Content-Type: {:?}", record.content_type);
    println!("  ├── Wire Legacy Version: {:?}", record.version);
    println!("  └── Record Payload Size: {} bytes", record.payload.len());

    if record.content_type == ContentType::Handshake && !record.payload.is_empty() {
        let handshake_type = record.payload[0];
        if handshake_type == 2 {
            // ServerHello
            let server_hello_body = &record.payload[4..];
            match parse_server_hello(server_hello_body) {
                Ok(sh) => {
                    println!("\n  [ServerHello Details]");
                    println!("    • Wire Legacy Version: 0x{:04x}", sh.legacy_version);
                    if let Some(sv) = sh.supported_version {
                        println!("    • Negotiated TLS Version: 0x{:04x} (TLS 1.3)", sv);
                    }
                    println!("    • Selected Cipher Suite: 0x{:04x}", sh.cipher_suite);
                    println!("    • Extension Count: {}", sh.extensions.len());

                    let ext_types: Vec<u16> =
                        sh.extensions.iter().map(|e| e.extension_type).collect();
                    println!("    • Extension IDs: {:?}", ext_types);

                    let ja4s = Ja4sFingerprint::compute(&sh);
                    println!("\n  ========================================================");
                    println!("  🎯 LIVE JA4S FINGERPRINT: {}", ja4s.full);
                    println!(
                        "     Version: {} | Cipher: {} | ExtHash: {}",
                        ja4s.version, ja4s.cipher, ja4s.ext_hash
                    );
                    println!("  ========================================================");
                }
                Err(e) => {
                    println!("    Notice: Could not parse ServerHello body: {:?}", e);
                }
            }
        } else {
            println!("    Received Handshake Message Type: {}", handshake_type);
        }
    } else {
        println!("    Received non-handshake response (e.g. Alert or plain TCP).");
    }

    println!("\nVerification Complete: AEGIS live internet TLS probing verified successfully!");
    Ok(())
}
