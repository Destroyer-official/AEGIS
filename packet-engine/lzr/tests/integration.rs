use aegis_lzr::LzrEngine;

#[test]
fn integration_http_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "HTTP/1.1");
    assert_eq!(result.category, "web");
}

#[test]
fn integration_https_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"HTTPS 1.1 200 OK\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "HTTPS");
}

#[test]
fn integration_ssh_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "SSH");
    assert_eq!(result.category, "remote");
}

#[test]
fn integration_smtp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"220 mail.example.com ESMTP Postfix (Ubuntu)\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "SMTP");
    assert_eq!(result.category, "mail");
}

#[test]
fn integration_pop3_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"+OK POP3 ready\r\n");
    assert!(result.matched);
    assert_eq!(result.category, "mail");
}

#[test]
fn integration_imap_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"* OK IMAP4rev1 ready\r\n");
    assert!(result.matched);
    assert_eq!(result.category, "mail");
}

#[test]
fn integration_mysql_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[
        0x00, 0x00, 0x00, 0x00, 0x0a, 0x38, 0x2e, 0x30, 0x2e, 0x33, 0x33, 0x00,
    ];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "MySQL");
    assert_eq!(result.category, "database");
}

#[test]
fn integration_postgresql_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"FATAL database \"test\" does not exist\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "PostgreSQL");
    assert_eq!(result.category, "database");
}

#[test]
fn integration_redis_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"-ERR unknown command\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "Redis");
    assert_eq!(result.category, "database");
}

#[test]
fn integration_mongodb_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[
        0x89, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xdd, 0x07, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "MongoDB");
    assert_eq!(result.category, "database");
}

#[test]
fn integration_elasticsearch_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"name": "node-1", "cluster_name": "test"}"#);
    assert!(result.matched);
    assert_eq!(result.category, "database");
}

#[test]
fn integration_ollama_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"version": "0.1.14"}"#);
    assert!(result.matched);
    assert_eq!(result.protocol, "Ollama");
    assert_eq!(result.category, "ai");
}

#[test]
fn integration_vllm_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"object": "list", "data": []}"#);
    assert!(result.matched);
    assert_eq!(result.protocol, "vLLM");
    assert_eq!(result.category, "ai");
}

#[test]
fn integration_openai_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"object": "chat.completion", "id": "chatcmpl-abc123"}"#);
    assert!(result.matched);
    assert_eq!(result.category, "ai");
}

#[test]
fn integration_triton_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"ready": true}"#);
    assert!(result.matched);
    assert_eq!(result.category, "ai");
}

#[test]
fn integration_comfyui_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(br#"{"prompt_id": "abc123"}"#);
    assert!(result.matched);
    assert_eq!(result.protocol, "ComfyUI");
    assert_eq!(result.category, "ai");
}

#[test]
fn integration_grpc_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
    assert!(result.matched);
    assert!(result.protocol.contains("gRPC") || result.protocol.contains("HTTP/2"));
}

#[test]
fn integration_websocket_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\r\n");
    assert!(result.matched);
    assert_eq!(result.category, "web");
}

#[test]
fn integration_rtsp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"RTSP/1.0 200 OK\r\nCSeq: 1\r\n\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "RTSP");
    assert_eq!(result.category, "streaming");
}

#[test]
fn integration_rtmp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.category, "streaming");
}

#[test]
fn integration_modbus_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x01, 0x03];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "Modbus");
    assert_eq!(result.category, "iot");
}

#[test]
fn integration_bacnet_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x81, 0x0b, 0x00, 0x05, 0x01];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "BACnet");
    assert_eq!(result.category, "iot");
}

#[test]
fn integration_opcua_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x4f, 0x50, 0x4e, 0x00, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "OPC-UA");
    assert_eq!(result.category, "iot");
}

#[test]
fn integration_rdp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[
        0x03, 0x00, 0x00, 0x13, 0x0e, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00,
        0x03, 0x00, 0x00, 0x00,
    ];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.category, "remote");
}

#[test]
fn integration_vnc_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"RFB 003.008\n");
    assert!(result.matched);
    assert_eq!(result.category, "remote");
}

#[test]
fn integration_sip_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"SIP/2.0 200 OK\r\n\r\n");
    assert!(result.matched);
    assert_eq!(result.protocol, "SIP");
    assert_eq!(result.category, "sip");
}

#[test]
fn integration_dns_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[
        0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "DNS");
    assert_eq!(result.category, "dns");
}

#[test]
fn integration_ntp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x24, 0x01, 0x06, 0xe8];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "NTP");
    assert_eq!(result.category, "ntp");
}

#[test]
fn integration_snmp_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[
        0x30, 0x26, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63,
    ];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.category, "snmp");
}

#[test]
fn integration_ike_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "IKE");
    assert_eq!(result.category, "vpn");
}

#[test]
fn integration_openvpn_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x00, 0x0e, 0x38, 0xde, 0xcc, 0xb4];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "OpenVPN");
    assert_eq!(result.category, "vpn");
}

#[test]
fn integration_wireguard_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x01, 0x00, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "WireGuard");
    assert_eq!(result.category, "vpn");
}

#[test]
fn integration_quic_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0xc0, 0x00, 0x00, 0x00, 0x01];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "QUIC-Initial");
    assert_eq!(result.category, "quic");
}

#[test]
fn integration_dtls_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x16, 0xfe, 0xfd, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "DTLS-1.0");
    assert_eq!(result.category, "tls");
}

#[test]
fn integration_stun_detection() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let payload: &[u8] = &[0x01, 0x01, 0x00, 0x00];
    let result = engine.classify(payload);
    assert!(result.matched);
    assert_eq!(result.protocol, "WebRTC-STUN");
    assert_eq!(result.category, "streaming");
}

#[test]
fn integration_classify_all_returns_multiple() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let matches = engine.classify_all(b"HTTP/1.1 200 OK\r\n");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].protocol, "HTTP/1.1");
    assert!(matches[0].priority >= 90);
}

#[test]
fn integration_unknown_payload() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"RANDOM UNKNOWN DATA THAT MATCHES NOTHING");
    assert!(!result.matched);
    assert_eq!(result.protocol, "unknown");
    assert_eq!(result.category, "unknown");
}

#[test]
fn integration_signature_count() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    assert!(engine.signature_count() >= 200);
}

#[test]
fn integration_empty_payload() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let result = engine.classify(b"");
    assert!(!result.matched);
}

#[test]
fn integration_priority_ordering() {
    let engine = LzrEngine::from_embedded_csv().unwrap();
    let matches = engine.classify_all(b"SSH-2.0-OpenSSH_9.0\r\n");
    assert!(!matches.is_empty());
    let ssh_9 = matches.iter().find(|m| m.protocol == "SSH-OpenSSH-9");
    let ssh_generic = matches.iter().find(|m| m.protocol == "SSH");
    if let (Some(s9), Some(sg)) = (ssh_9, ssh_generic) {
        assert!(s9.priority >= sg.priority);
    }
}
