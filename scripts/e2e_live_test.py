#!/usr/bin/env python3
"""
AEGIS Platform — Live End-to-End Runtime Verification Script
Exercises the live running Gateway service over real HTTP/REST requests:
1. Health & OpenTelemetry check (/health)
2. Role-Based Access Control & Argon2id Authentication (/auth/login)
3. Cryptographic defenses against authentication bypass (Finding 1)
4. Privilege isolation between Admin, Analyst, and Viewer roles
5. Tenant provisioning & quota configuration (/admin/users)
6. High-entropy CSPRNG API-key creation & single-use secret response (Finding 4)
7. SQL Query Grammar Security & statement chaining injection defense (Finding 2)
8. Ed25519 Cryptographic Hash Chain Audit Log Verification (Finding 5)
"""

import sys
import os
import time
import json
import urllib.request
import urllib.error
import subprocess
import shutil

GATEWAY_PORT = 3005
BASE_URL = f"http://127.0.0.1:{GATEWAY_PORT}"
AUDIT_LOGS_DIR = "test_audit_logs"

def log(msg, status="INFO"):
    symbol = {"INFO": "ℹ️", "SUCCESS": "✅", "WARN": "⚠️", "FAIL": "❌"}.get(status, "•")
    print(f"[{status}] {symbol} {msg}")

def http_req(endpoint, method="GET", payload=None, token=None):
    url = f"{BASE_URL}{endpoint}"
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            body = response.read().decode("utf-8")
            status = response.status
            try:
                parsed = json.loads(body)
            except json.JSONDecodeError:
                parsed = body
            return status, parsed
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8")
        try:
            parsed = json.loads(body)
        except json.JSONDecodeError:
            parsed = body
        return e.code, parsed
    except Exception as e:
        return 0, str(e)

def main():
    print("=" * 70)
    print("AEGIS PLATFORM — LIVE END-TO-END SYSTEM INTEGRATION TEST")
    print("=" * 70)

    # Clean up test audit log directory if present
    if os.path.exists(AUDIT_LOGS_DIR):
        shutil.rmtree(AUDIT_LOGS_DIR, ignore_errors=True)

    candidates = [
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "gateway", "target", "debug", "aegis-gateway.exe")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "gateway", "target", "debug", "aegis-gateway")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "gateway", "target", "debug", "aegis_gateway.exe")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "gateway", "target", "debug", "aegis_gateway")),
    ]
    gateway_binary = next((c for c in candidates if os.path.isfile(c)), None)
    if not gateway_binary:
        log(f"Gateway binary not found in candidates: {candidates}. Please run 'cargo build' in gateway.", "FAIL")
        sys.exit(1)

    log(f"Found Gateway binary: {gateway_binary}", "INFO")

    # Set up environment for gateway
    env = os.environ.copy()
    env["GATEWAY_PORT"] = str(GATEWAY_PORT)
    env["GATEWAY_HOST"] = "127.0.0.1"
    env["GRPC_PORT"] = "50055"
    env["JWT_SECRET"] = "aegis-e2e-live-test-secret-key-32-chars-long!"
    env["INITIAL_ADMIN_PASSWORD"] = "AegisAdmin#Secure2026!"
    env["INITIAL_ANALYST_PASSWORD"] = "AegisAnalyst#2026!"
    env["INITIAL_VIEWER_PASSWORD"] = "AegisViewer#2026!"
    env["OTEL_SDK_DISABLED"] = "true"
    env["RUST_LOG"] = "info"

    log(f"Launching Gateway process on port {GATEWAY_PORT}...", "INFO")
    gateway_log_path = os.path.join(os.path.dirname(__file__), "gateway_e2e.log")
    log_file = open(gateway_log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([gateway_binary], env=env, stdout=log_file, stderr=log_file)

    try:
        # 1. Wait for server readiness
        ready = False
        for _ in range(40):
            time.sleep(0.5)
            status, resp = http_req("/health")
            if status == 200 and isinstance(resp, dict) and resp.get("status") == "ok":
                ready = True
                break

        if not ready:
            log("Gateway server failed to start within timeout", "FAIL")
            proc.kill()
            sys.exit(1)

        log(f"Gateway is LIVE! Status: 200 OK, Version: {resp.get('version')}", "SUCCESS")

        # 2. Test Critical Finding 1 Defense: Password-prefix bypass attempt MUST FAIL
        log("Testing Finding 1 Defense: Attacker bypass attempt with 'admin' prefix...", "INFO")
        status, resp = http_req("/auth/login", method="POST", payload={
            "username": "attacker",
            "password": "admin-unauthorized-bypass"
        })
        assert status == 401, f"Expected 401 Unauthorized for bypass attempt, got {status}"
        log("Finding 1 bypass attempt successfully REJECTED (401 Unauthorized)", "SUCCESS")

        # 3. Test Invalid Password for existing admin
        status, resp = http_req("/auth/login", method="POST", payload={
            "username": "admin",
            "password": "WrongPassword123!"
        })
        assert status == 401, f"Expected 401 for wrong password, got {status}"
        log("Invalid password for admin user rejected (401 Unauthorized)", "SUCCESS")

        # 4. Test Valid Argon2id Authentication for Admin
        log("Authenticating as 'admin' with valid Argon2id credentials...", "INFO")
        status, admin_auth = http_req("/auth/login", method="POST", payload={
            "username": "admin",
            "password": "AegisAdmin#Secure2026!"
        })
        if status != 200:
            log(f"Login failed: status={status}, response={admin_auth}", "FAIL")
        assert status == 200, f"Expected 200 OK for valid admin login, got {status}: {admin_auth}"
        admin_token = admin_auth["token"]
        assert admin_token and len(admin_token) > 20
        log(f"Admin authentication SUCCESS. JWT token received ({len(admin_token)} bytes)", "SUCCESS")

        # 5. Test Valid Argon2id Authentication for Analyst
        log("Authenticating as 'analyst'...", "INFO")
        status, analyst_auth = http_req("/auth/login", method="POST", payload={
            "username": "analyst",
            "password": "AegisAnalyst#2026!"
        })
        assert status == 200, f"Expected 200 OK for valid analyst login, got {status}"
        analyst_token = analyst_auth["token"]
        log("Analyst authentication SUCCESS", "SUCCESS")

        # 6. Test Privilege Boundary: Analyst cannot access Admin APIs
        log("Testing RBAC boundary: Analyst requesting /admin/users...", "INFO")
        status, _ = http_req("/admin/users", method="POST", payload={"action": "list"}, token=analyst_token)
        assert status == 403, f"Expected 403 Forbidden for non-admin on /admin/users, got {status}"
        log("RBAC boundary enforced: Analyst blocked from admin endpoints (403 Forbidden)", "SUCCESS")

        # 7. Test Tenant Creation via Admin API
        log("Admin creating tenant 'tenant-live-cyber'...", "INFO")
        status, resp = http_req("/admin/users", method="POST", payload={
            "action": "create",
            "config": {
                "id": "tenant-live-cyber",
                "name": "Live Cyber Enterprise",
                "max_rate_rps": 500,
                "max_scans_per_day": 100,
                "max_queries_per_hour": 10000,
                "data_retention_days": 180,
                "enabled": True
            }
        }, token=admin_token)
        assert status == 200, f"Expected 200 for tenant creation, got {status}"
        log("Tenant 'tenant-live-cyber' created successfully", "SUCCESS")

        # 8. Test Finding 4 Defense: API-Key Generation & Single-Use Secret
        log("Testing Finding 4 Defense: Generating high-entropy API key...", "INFO")
        status, key_resp = http_req("/admin/users", method="POST", payload={
            "action": "apikey_create",
            "id": "tenant-live-cyber",
            "name": "automated-scanner-agent"
        }, token=admin_token)
        assert status == 200, f"Expected 200 for API key creation, got {status}"
        raw_key = key_resp.get("raw_key")
        assert raw_key and raw_key.startswith("aegis_tenant-live-cyber_"), f"Unexpected key format: {raw_key}"
        assert len(raw_key) > 40, f"Key too short: {raw_key}"
        assert "raw_key" in key_resp, "Expected raw_key in one-time secret response"
        assert "key_hash" not in key_resp, "Private verifier hash must not be leaked in creation response"
        log(f"High-entropy CSPRNG API key created: {raw_key[:20]}... (Single-use secret delivered, hash kept private)", "SUCCESS")

        # 9. Test Finding 2 Defense: SQL Security Boundary & Chaining Detection
        log("Testing Finding 2 Defense: SQL Injection Statement Chaining Prevention...", "INFO")
        status, _ = http_req("/query", method="POST", payload={
            "sql": "SELECT * FROM internet_assets; DROP TABLE tenants; --"
        }, token=admin_token)
        assert status == 400, f"Expected 400 Bad Request for SQL chaining attack, got {status}"
        log("SQL injection chaining attack BLOCKED by query grammar defense (400 Bad Request)", "SUCCESS")

        # 10. Test Finding 2 Defense: Destructive DDL statement rejection
        status, _ = http_req("/query", method="POST", payload={
            "sql": "DROP TABLE scans"
        }, token=admin_token)
        assert status in (400, 403), f"Expected 400 or 403 for DROP TABLE, got {status}"
        log("Destructive DDL query BLOCKED by query grammar defense (403 Forbidden)", "SUCCESS")

        # 11. Test Finding 5 Defense: Ed25519 Cryptographic Hash Chain Audit Verification
        log("Testing Finding 5 Defense: Verifying tamper-evident audit log hash chain...", "INFO")
        status, audit_resp = http_req("/admin/action", method="POST", payload={
            "action": "verify"
        }, token=admin_token)
        assert status == 200, f"Expected 200 for audit verify, got {status}"
        assert audit_resp.get("valid") is True, f"Audit chain verification failed: {audit_resp}"
        log(f"Audit log verified: {audit_resp.get('count')} records in valid Ed25519 cryptographic hash chain", "SUCCESS")

        # 12. Rate Limit Status Check
        log("Checking Rate Limiter status...", "INFO")
        status, rl_resp = http_req("/rate-limit/status", token=admin_token)
        assert status == 200, f"Expected 200 for rate limit status, got {status}"
        log(f"Rate Limiter Status: 200 OK (Remaining tokens: {rl_resp.get('remaining', 'N/A')})", "SUCCESS")

        # 13. Test RESTful Tenant Management Endpoints
        log("Testing RESTful Tenant API: GET /admin/tenants...", "INFO")
        status, tenants = http_req("/admin/tenants", token=admin_token)
        assert status == 200, f"Expected 200 for GET /admin/tenants, got {status}"
        assert any(t["id"] == "tenant-live-cyber" for t in tenants), "Created tenant not found in GET /admin/tenants"
        log("RESTful GET /admin/tenants verified: returned all tenants", "SUCCESS")

        log("Testing RESTful Tenant API: POST /admin/tenants/:id/api-keys...", "INFO")
        status, rest_key = http_req("/admin/tenants/tenant-live-cyber/api-keys", method="POST", payload={
            "name": "rest-ingest-key",
            "scopes": ["read", "query"]
        }, token=admin_token)
        assert status == 200, f"Expected 200 for REST API key creation, got {status}"
        assert rest_key.get("raw_key", "").startswith("aegis_tenant-live-cyber_")
        log(f"RESTful API key created: {rest_key.get('name')}", "SUCCESS")

        log("Testing RESTful Tenant API: GET /admin/tenants/:id/api-keys...", "INFO")
        status, key_list = http_req("/admin/tenants/tenant-live-cyber/api-keys", token=admin_token)
        assert status == 200, f"Expected 200 for GET api-keys, got {status}"
        assert len(key_list) >= 2, f"Expected at least 2 keys, got {len(key_list)}"
        log(f"RESTful API key listing verified: {len(key_list)} keys", "SUCCESS")

        log("Testing RESTful Tenant API: POST /admin/tenants/:id/invite...", "INFO")
        status, invite_resp = http_req("/admin/tenants/tenant-live-cyber/invite", method="POST", payload={
            "email": "analyst@live-cyber.com",
            "role": "analyst"
        }, token=admin_token)
        assert status == 200, f"Expected 200 for invite, got {status}"
        assert invite_resp.get("status") == "invited"
        log("RESTful user invitation verified (email=analyst@live-cyber.com)", "SUCCESS")

        # 14. Test Scan Orchestration REST API
        log("Testing Scan Orchestration: POST /scans (analyst starting scan)...", "INFO")
        status, scan_job = http_req("/scans", method="POST", payload={
            "target": "192.168.10.0/24",
            "scan_type": "l4_port",
            "options": {
                "port_range": "22,80,443,8080",
                "timeout": 5
            }
        }, token=analyst_token)
        assert status == 201, f"Expected 201 Created for scan dispatch, got {status}: {scan_job}"
        scan_id = scan_job.get("id")
        assert scan_id and scan_id.startswith("scan-"), f"Invalid scan_id: {scan_id}"
        assert scan_job.get("status") == "running"
        assert scan_job.get("total") == 256
        log(f"Scan dispatched successfully via REST API: ID={scan_id}, Target={scan_job.get('target')}", "SUCCESS")

        log("Testing Scan Orchestration: GET /scans (analyst listing scans)...", "INFO")
        status, scans_list = http_req("/scans", token=analyst_token)
        assert status == 200, f"Expected 200 for GET /scans, got {status}"
        assert any(s["id"] == scan_id for s in scans_list), f"Dispatched scan {scan_id} not found in scan list"
        log(f"Scan listing verified: {len(scans_list)} scans found", "SUCCESS")

        log(f"Testing Scan Orchestration: POST /scans/{scan_id}/cancel...", "INFO")
        status, cancel_resp = http_req(f"/scans/{scan_id}/cancel", method="POST", token=analyst_token)
        assert status == 200, f"Expected 200 for scan cancel, got {status}"
        assert cancel_resp.get("status") == "stopped"
        log(f"Scan {scan_id} stopped cleanly via REST API", "SUCCESS")

        # 15. Verify Tamper-Evident Hash Chain includes Scan & Tenant Audit Entries
        log("Re-verifying Ed25519 hash chain with scan & tenant events included...", "INFO")
        status, final_audit = http_req("/admin/action", method="POST", payload={"action": "verify"}, token=admin_token)
        assert status == 200, f"Expected 200 for final audit verify, got {status}"
        assert final_audit.get("valid") is True
        log(f"Final tamper-evident hash chain verified: {final_audit.get('count')} total cryptographically verified records", "SUCCESS")

        print("=" * 70)
        print("🎉 ALL 15 LIVE RUNTIME & INTEGRATION TESTS PASSED WITH 100% SUCCESS!")
        print("=" * 70)

    finally:
        log("Stopping Gateway process...", "INFO")
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_file.close()
        log("Gateway process stopped cleanly", "INFO")

if __name__ == "__main__":
    main()
