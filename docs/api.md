# AEGIS API Reference

**Base URL:** `https://api.aegis.example.com` (production) / `http://localhost:3000` (local)  
**Protocol:** HTTPS (TLS 1.3)  
**Authentication:** JWT Bearer Token  
**Rate Limit:** Token bucket (configurable per tenant)

---

## 1. Authentication

### 1.1 Login
```http
POST /auth/login
Content-Type: application/json

{
  "username": "analyst@company.com",
  "password": "secure-password"
}
```

**Response (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600
}
```

**Error (401):**
```json
{ "error": "Invalid credentials", "code": 401 }
```

### 1.2 Token Usage
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 1.3 Roles & Permissions

| Role | Query | Admin Actions | User Mgmt | Rate Limit Config |
|------|-------|---------------|-----------|-------------------|
| Viewer | ✅ (read-only) | ❌ | ❌ | ❌ |
| Analyst | ✅ | ❌ | ❌ | ❌ |
| Admin | ✅ | ✅ | ✅ | ✅ |

---

## 2. Health Check

```http
GET /health
```

**Response (200):**
```json
{
  "status": "ok",
  "version": "1.2.3",
  "timestamp": "2026-08-30T12:00:00Z"
}
```

---

## 3. Query API (Analyst+)

### 3.1 Execute SQL Query
```http
POST /query
Authorization: Bearer <token>
Content-Type: application/json

{
  "query": "SELECT ip, port, protocol, ja4 FROM services WHERE status = 'active' LIMIT 100",
  "catalog": "aegis",
  "schema": "aegis"
}
```

**Response (200):**
```json
{
  "columns": [
    {"name": "ip", "data_type": "varchar"},
    {"name": "port", "data_type": "integer"},
    {"name": "protocol", "data_type": "varchar"},
    {"name": "ja4", "data_type": "varchar"}
  ],
  "rows": [
    ["192.0.2.1", 443, "HTTPS", "t13d1516h2_8daaf6152771_e5627efa2ab1"],
    ["192.0.2.2", 22, "SSH", "t13d1516h2_8daaf6152771_e5627efa2ab1"]
  ],
  "total_rows": 12450,
  "query_id": "20260830_120000_00001_abc123",
  "stats": {
    "state": "FINISHED",
    "queued": false,
    "rows": 100,
    "processed_rows": 12450
  }
}
```

### 3.2 Time-Travel Query
```sql
SELECT * FROM hosts
FOR SYSTEM_TIME AS OF TIMESTAMP '2026-08-01 00:00:00'
WHERE ip = '192.0.2.1';
```

### 3.3 Common Query Patterns

**Active services with JA4:**
```sql
SELECT s.host_ip, s.port, s.protocol, s.ja4, s.status
FROM services s
WHERE s.status = 'active' AND s.ja4 IS NOT NULL
ORDER BY s.last_seen DESC
LIMIT 100;
```

**Certificate analysis:**
```sql
SELECT c.sha256_fingerprint, c.subject_cn, c.issuer_cn,
       c.not_before, c.not_after, c.is_expired
FROM certificates c
WHERE c.is_expired = false
ORDER BY c.not_after ASC
LIMIT 50;
```

**Vulnerable services:**
```sql
SELECT h.ip, s.port, s.protocol, v.cve, v.cvss
FROM hosts h
JOIN services s ON h.ip = s.host_ip
JOIN LATERAL (
  SELECT unnest(s.cves) AS cve, unnest(s.cvss_scores) AS cvss
) v ON true
WHERE v.cvss >= 7.0
ORDER BY v.cvss DESC;
```

**Asset clustering by TLS serial:**
```sql
SELECT c.serial_number, array_agg(DISTINCT s.host_ip) AS ips
FROM certificates c
JOIN services s ON c.sha256_fingerprint = s.cert_sha256
GROUP BY c.serial_number
HAVING cardinality(array_agg(DISTINCT s.host_ip)) > 1;
```

---

## 4. Rate Limit Status

```http
GET /rate-limit/status
Authorization: Bearer <token>
```

**Response (200):**
```json
{
  "key": "analyst@company.com",
  "allowed": true,
  "remaining": 947
}
```

---

## 5. Admin Endpoints (Admin Only)

### 5.1 Audit Actions
```http
POST /admin/action
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "action": "export",
  "tenant_id": "tenant-a"  // optional
}
```

**Actions:** `export`, `verify`, `key`, `key_hex`

**Response (200):**
```json
{
  "valid": true,
  "count": 12450
}
```

### 5.2 Tenant Management
```http
POST /admin/users
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "action": "create",
  "config": {
    "id": "tenant-new",
    "name": "New Customer",
    "max_rate_rps": 200,
    "max_scans_per_day": 100,
    "max_queries_per_hour": 5000,
    "data_retention_days": 180,
    "enabled": true
  }
}
```

**Actions:** `list`, `create`, `update`, `delete`, `quota`, `apikey_create`, `apikey_list`, `apikey_revoke`

---

## 6. WebSocket Streaming

### 6.1 Query Streaming
```javascript
const ws = new WebSocket('ws://localhost:3000/ws/query', ['bearer', '<token>']);
ws.onmessage = (event) => {
  const chunk = JSON.parse(event.data);
  console.log('Rows:', chunk.rows, 'Last:', chunk.is_last);
};

ws.send(JSON.stringify({
  sql: "SELECT * FROM services WHERE status = 'active'",
  batch_size: 1000
}));
```

**Chunk Response:**
```json
{
  "columns": [{"name": "host_ip", "data_type": "varchar"}, ...],
  "rows": [["192.0.2.1", 443, "HTTPS", ...], ...],
  "is_last": false,
  "query_id": "20260830_120000_00001_abc123"
}
```

---

## 7. gRPC API (Internal)

### 7.1 Trino Query Service
```protobuf
service QueryService {
  rpc ExecuteQuery(QueryRequest) returns (QueryResponse);
  rpc StreamQuery(StreamRequest) returns (stream StreamChunk);
}

message QueryRequest {
  string sql = 1;
  string catalog = 2;
  string schema = 3;
}

message QueryResponse {
  repeated Column columns = 1;
  repeated Row rows = 2;
  int64 total_rows = 3;
  string query_id = 4;
}

message StreamRequest {
  string sql = 1;
  int32 batch_size = 2;
}

message StreamChunk {
  repeated Row rows = 1;
  bool is_last = 2;
  string query_id = 3;
}
```

---

## 8. Error Codes

| Code | Meaning | Resolution |
|------|---------|------------|
| 400 | Bad Request | Check JSON syntax, required fields |
| 401 | Unauthorized | Token expired or invalid |
| 403 | Forbidden | Insufficient role/permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Duplicate tenant/API key |
| 422 | Unprocessable | Validation failed |
| 429 | Rate Limited | Wait for `Retry-After` header |
| 500 | Internal Error | Check logs, retry |
| 503 | Unavailable | Service starting/maintenance |

---

## 9. Rate Limiting

| Tier | Requests/sec | Burst | Quota Type |
|------|--------------|-------|------------|
| Viewer | 10 | 20 | Per user |
| Analyst | 100 | 200 | Per user |
| Admin | 1000 | 2000 | Per user |
| Scan Quota | Configurable | Per tenant/day | Tenant config |
| Query Quota | Configurable | Per tenant/hour | Tenant config |

**Headers:**
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1693411200
Retry-After: 30
```

---

## 10. Pagination

All list endpoints support cursor pagination:
```http
GET /admin/users?action=list&limit=50&cursor=eyJpZCI6InRlbmFudC1hIn0=
```

**Response:**
```json
{
  "items": [...],
  "next_cursor": "eyJpZCI6InRlbmFudC1iIn0=",
  "has_more": true
}
```

---

## 11. OpenAPI Specification

Full OpenAPI 3.1 spec available at:
- **File:** `gateway/openapi.yaml`
- **Swagger UI:** `https://api.aegis.example.com/swagger-ui` (when deployed)

---

## 12. SDKs

| Language | Package | Install |
|----------|---------|---------|
| Go | `github.com/aegis-scanner/aegis-go` | `go get github.com/aegis-scanner/aegis-go` |
| Python | `aegis-sdk` | `pip install aegis-sdk` |
| TypeScript | `@aegis/scanner` | `npm install @aegis/scanner` |

---

## 13. Example: Complete Workflow

```python
from aegis import AegisClient

client = AegisClient(
    base_url="https://api.aegis.example.com",
    username="analyst@company.com",
    password="secure-password"
)

# Query vulnerable services
results = client.query("""
    SELECT h.ip, s.port, s.protocol, v.cve, v.cvss
    FROM hosts h
    JOIN services s ON h.ip = s.host_ip
    JOIN LATERAL (
      SELECT unnest(s.cves) AS cve, unnest(s.cvss_scores) AS cvss
    ) v ON true
    WHERE v.cvss >= 7.0
    ORDER BY v.cvss DESC
    LIMIT 100
""")

for row in results:
    print(f"{row['ip']}:{row['port']} {row['protocol']} - {row['cve']} (CVSS {row['cvss']})")

# Export audit log
audit = client.admin_action("export", tenant_id="tenant-a")
print(f"Exported {len(audit)} audit entries")
```

---

## 14. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-08-30 | Initial production release |
| 1.1.0 | 2026-09-15 | Added VistaScan, LLM fingerprinting, audit API |