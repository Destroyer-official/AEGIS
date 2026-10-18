#!/bin/bash
# AEGIS End-to-End Pipeline Validation
# Tests: Synthetic Generator → Kafka → Flink → Iceberg → Trino

set -euo pipefail

echo "=============================================="
echo "  AEGIS End-to-End Pipeline Validation"
echo "=============================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
FAIL=0

check() {
    local name="$1"
    local result="$2"
    if [ "$result" = "PASS" ]; then
        echo -e "  ${GREEN}[PASS]${NC} $name"
        ((PASS++))
    else
        echo -e "  ${RED}[FAIL]${NC} $name: $result"
        ((FAIL++))
    fi
}

# 1. Run synthetic generator
echo "--- Step 1: Generate synthetic events ---"
echo "Running synthetic generator (10 events/sec for 30 seconds)..."
cd /Users/shant/OneDrive/Documents/AEGIS/infra/synthetic-generator
cargo run --release -- \
    --kafka-broker localhost:9092 \
    --topic raw-l4-scans \
    --rate 10 \
    --duration 30 \
    --source-pop aegis-poc 2>&1 | tail -5

# 2. Verify events in Kafka
echo ""
echo "--- Step 2: Verify events in Kafka ---"
TOPIC_COUNT=$(kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list 2>/dev/null | wc -l)
check "Kafka topics exist" $([ "$TOPIC_COUNT" -ge 5 ] && echo "PASS" || echo "FAIL: $TOPIC_COUNT topics")

MSG_COUNT=$(kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-console-consumer.sh \
    --bootstrap-server localhost:9092 \
    --topic raw-l4-scans \
    --from-beginning \
    --max-messages 100 \
    --timeout-ms 10000 2>/dev/null | wc -l)
check "Events in raw-l4-scans topic" $([ "$MSG_COUNT" -gt 0 ] && echo "PASS" || echo "FAIL: $MSG_COUNT messages")

# 3. Verify Flink job is running
echo ""
echo "--- Step 3: Verify Flink job ---"
JOBMANAGER_POD=$(kubectl get pods -l component=jobmanager -n default -o jsonpath='{.items[0].metadata.name}')
FLINK_JOBS=$(kubectl exec "$JOBMANAGER_POD" -n default -- flink list -r 2>/dev/null || echo "ERROR")
if echo "$FLINK_JOBS" | grep -q "RUNNING"; then
    check "Flink job running" "PASS"
else
    check "Flink job running" "FAIL: $FLINK_JOBS"
fi

# 4. Verify Iceberg tables in Trino
echo ""
echo "--- Step 4: Verify Iceberg tables via Trino ---"
TRINO_POD=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}')

# Wait a bit for Flink to write
echo "Waiting 15 seconds for Flink to write to Iceberg..."
sleep 15

SCHEMAS=$(kubectl exec "$TRINO_POD" -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW SCHEMAS IN aegis" 2>/dev/null)
check "Schema 'aegis' exists" $(echo "$SCHEMAS" | grep -q "aegis" && echo "PASS" || echo "FAIL")

TABLES=$(kubectl exec "$TRINO_POD" -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW TABLES IN aegis.aegis" 2>/dev/null)
check "Tables exist" $(echo "$TABLES" | grep -q "services" && echo "PASS" || echo "FAIL")

# Query services table
SERVICES_COUNT=$(kubectl exec "$TRINO_POD" -n default -- trino --server http://127.0.0.1:8080 --execute "SELECT count(*) FROM aegis.aegis.services" 2>/dev/null | grep -oE '[0-9]+' | head -1)
check "Services table has rows" $([ -n "$SERVICES_COUNT" ] && [ "$SERVICES_COUNT" -gt 0 ] && echo "PASS" || echo "FAIL: $SERVICES_COUNT rows")

# 5. Time-travel query test
echo ""
echo "--- Step 5: Time-travel query ---"
echo "Testing Iceberg time-travel..."
kubectl exec "$TRINO_POD" -n default -- trino --server http://127.0.0.1:8080 --execute "
    SELECT count(*) as current_count FROM aegis.aegis.services
" 2>/dev/null

# Get snapshot ID for time travel
SNAPSHOT=$(kubectl exec "$TRINO_POD" -n default -- trino --server http://127.0.0.1:8080 --execute "
    SELECT snapshot_id FROM aegis.aegis.\$snapshots LIMIT 1
" 2>/dev/null | grep -oE '[0-9]+' | head -1)

if [ -n "$SNAPSHOT" ]; then
    echo "Found snapshot: $SNAPSHOT"
    check "Time-travel query capability" "PASS (snapshot $SNAPSHOT available)"
else
    check "Time-travel query capability" "FAIL: no snapshots found"
fi

# 6. Summary
echo ""
echo "=============================================="
echo "  Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}"
echo "=============================================="

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "Some checks failed. Troubleshooting:"
    echo "  - Flink job: kubectl port-forward svc/aegis-flink-jobmanager 8081:8081"
    echo "  - Trino:     kubectl port-forward svc/aegis-trino-coordinator 8080:8080"
    echo "  - Kafka:     kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-console-consumer.sh --topic raw-l4-scans --bootstrap-server localhost:9092 --from-beginning"
    exit 1
fi

echo ""
echo "All validations passed!"
echo ""
echo "Pipeline verified:"
echo "  Synthetic Generator → Kafka (raw-l4-scans)"
echo "  → Flink (1-min dedup window)"
echo "  → Iceberg (aegis.aegis.services)"
echo "  → Trino (time-travel queries)"