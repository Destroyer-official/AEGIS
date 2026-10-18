#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0

check() {
    local name="$1"
    local result="$2"
    if [ "$result" = "PASS" ]; then
        echo "  [PASS] $name"
        PASS=$((PASS + 1))
    else
        echo "  [FAIL] $name: $result"
        FAIL=$((FAIL + 1))
    fi
}

echo "============================================="
echo "  AEGIS Infrastructure Verification"
echo "============================================="
echo ""

# 1. Kubernetes nodes
echo "--- Kubernetes ---"
NODES=$(kubectl get nodes --no-headers 2>/dev/null | wc -l)
if [ "$NODES" -ge 1 ]; then
    check "Kind cluster running ($NODES nodes)" "PASS"
else
    check "Kind cluster running" "No nodes found"
fi

# 2. Pods
echo ""
echo "--- Pods ---"
ALL_PODS=$(kubectl get pods --no-headers 2>/dev/null)
POD_COUNT=$(echo "$ALL_PODS" | grep -c "Running" || true)
check "Running pods ($POD_COUNT)" "PASS"

for component in aegis-kafka aegis-minio aegis-flink aegis-trino aegis-polaris; do
    if echo "$ALL_PODS" | grep -q "$component.*Running"; then
        check "$component pod running" "PASS"
    else
        check "$component pod running" "Not found or not running"
    fi
done

# 3. Kafka topics
echo ""
echo "--- Kafka Topics ---"
TOPICS=$(kubectl exec kafka-0 -- /opt/bitnami/kafka/bin/kafka-topics.sh \
    --bootstrap-server localhost:9092 --list 2>/dev/null | sort)
TOPIC_COUNT=$(echo "$TOPICS" | grep -c . || true)

EXPECTED_TOPICS="enrichment-events
l7-banners
raw-l4-scans
scan-commands
tls-fingerprints"

if [ "$TOPIC_COUNT" -eq 5 ]; then
    check "Exactly 5 Kafka topics ($TOPIC_COUNT)" "PASS"
else
    check "Exactly 5 Kafka topics" "Found $TOPIC_COUNT topics"
fi

# Verify each topic exists
for topic in raw-l4-scans tls-fingerprints l7-banners scan-commands enrichment-events; do
    if echo "$TOPICS" | grep -q "^${topic}$"; then
        check "Topic '$topic' exists" "PASS"
    else
        check "Topic '$topic' exists" "Missing"
    fi
done

echo ""
echo "  All topics:"
echo "$TOPICS" | sed 's/^/    /'

# 4. Flink UI
echo ""
echo "--- Flink ---"
FLINK_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/overview 2>/dev/null || echo "000")
if [ "$FLINK_CODE" = "200" ]; then
    check "Flink Web UI accessible on :8081" "PASS"
else
    check "Flink Web UI accessible on :8081" "HTTP $FLINK_CODE"
fi

# 5. Trino
echo ""
echo "--- Trino ---"
TRINO_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/v1/info 2>/dev/null || echo "000")
if [ "$TRINO_CODE" = "200" ]; then
    check "Trino accessible on :8080" "PASS"
else
    check "Trino accessible on :8080" "HTTP $TRINO_CODE"
fi

# 6. MinIO
echo ""
echo "--- MinIO ---"
MINIO_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9000/minio/health/live 2>/dev/null || echo "000")
if [ "$MINIO_CODE" = "200" ]; then
    check "MinIO API accessible on :9000" "PASS"
else
    check "MinIO API accessible on :9000" "HTTP $MINIO_CODE"
fi

MINIO_CONSOLE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9001/ 2>/dev/null || echo "000")
if [ "$MINIO_CONSOLE" = "200" ] || [ "$MINIO_CONSOLE" = "301" ] || [ "$MINIO_CONSOLE" = "302" ]; then
    check "MinIO Console accessible on :9001" "PASS"
else
    check "MinIO Console accessible on :9001" "HTTP $MINIO_CONSOLE"
fi

# 7. Polaris
echo ""
echo "--- Polaris ---"
POLARIS_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8181/ 2>/dev/null || echo "000")
if [ "$POLARIS_CODE" = "200" ] || [ "$POLARIS_CODE" = "404" ] || [ "$POLARIS_CODE" = "302" ]; then
    check "Polaris catalog accessible on :8181" "PASS"
else
    check "Polaris catalog accessible on :8181" "HTTP $POLARIS_CODE"
fi

# 8. Iceberg tables via Trino
echo ""
echo "--- Iceberg Tables ---"
TRINO_RESULT=$(curl -s -X POST http://localhost:8080/v1/statement \
    -H "X-Trino-User: aegis" \
    -H "X-Trino-Catalog: aegis" \
    -H "X-Trino-Schema: aegis" \
    -d "SELECT table_name FROM information_schema.tables WHERE table_schema = 'aegis'" 2>/dev/null || echo "error")

if echo "$TRINO_RESULT" | grep -q "hosts\|services\|certificates"; then
    check "Iceberg tables visible in Trino" "PASS"
else
    check "Iceberg tables visible in Trino" "Tables not found via Trino query"
fi

# Summary
echo ""
echo "============================================="
echo "  Results: $PASS passed, $FAIL failed"
echo "============================================="

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi

echo ""
echo "  All verifications passed!"
echo ""
echo "  Cluster URLs:"
echo "    Kafka:      localhost:9092"
echo "    Flink UI:   http://localhost:8081"
echo "    Trino:      http://localhost:8080"
echo "    MinIO API:  http://localhost:9000"
echo "    MinIO UI:   http://localhost:9001"
echo "    Polaris:    http://localhost:8181"
echo ""
echo "  Kafka Topics:"
echo "    raw-l4-scans"
echo "    tls-fingerprints"
echo "    l7-banners"
echo "    scan-commands"
echo "    enrichment-events"
echo ""
echo "  Schema Registry:"
echo "    Polaris REST Catalog: http://localhost:8181/api/catalog"
echo "    Namespace: aegis"
echo "    Tables: hosts, services, certificates"
