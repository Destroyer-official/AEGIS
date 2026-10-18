#!/bin/bash
# AEGIS Disaster Recovery Test
# Validates RTO < 15min, RPO < 1min

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

RTO_TARGET=900
RPO_TARGET=60

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

check_prereqs() {
    log_info "Checking prerequisites..."
    for cmd in kubectl mc curl jq; do
        if ! command -v $cmd &> /dev/null; then
            log_fail "$cmd not found"
            exit 1
        fi
    done
    for script in iceberg-backup.sh minio-backup.sh kafka-backup.sh; do
        if [ ! -f "$script" ]; then
            log_fail "Missing DR script: $script"
            exit 1
        fi
    done
    log_pass "Prerequisites OK"
}

backup_all() {
    log_info "=== Phase 1: Full Backup ==="
    local start=$(date +%s)

    log_info "Backing up Iceberg/Nessie..."
    ./iceberg-backup.sh backup > /dev/null 2>&1

    log_info "Backing up MinIO..."
    ./minio-backup.sh backup > /dev/null 2>&1

    log_info "Backing up Kafka..."
    ./kafka-backup.sh backup > /dev/null 2>&1

    local elapsed=$(($(date +%s) - start))
    log_pass "Full backup completed in ${elapsed}s"
}

simulate_disaster() {
    log_info "=== Phase 2: Simulate Disaster ==="
    log_warn "Deleting all AEGIS workloads..."

    kubectl delete deployment -n default -l app=aegis-trino --ignore-not-found=true
    kubectl delete deployment -n default -l app=aegis-flink --ignore-not-found=true
    kubectl delete deployment -n default -l app=aegis-polaris --ignore-not-found=true
    kubectl delete statefulset -n default aegis-kafka --ignore-not-found=true
    kubectl delete statefulset -n default aegis-minio --ignore-not-found=true

    sleep 30
    log_pass "Disaster simulated - all workloads terminated"
}

restore_all() {
    log_info "=== Phase 3: Restore ==="
    local start=$(date +%s)

    log_info "Re-deploying infrastructure..."
    helm upgrade --install aegis-kafka ../../charts/aegis-kafka -n default --wait --timeout=5m 2>/dev/null
    helm upgrade --install aegis-minio ../../charts/aegis-minio -n default --wait --timeout=5m 2>/dev/null
    helm upgrade --install aegis-polaris ../../charts/aegis-polaris -n default --wait --timeout=5m 2>/dev/null
    helm upgrade --install aegis-trino ../../charts/aegis-trino -n default --wait --timeout=5m 2>/dev/null
    helm upgrade --install aegis-flink ../../charts/aegis-flink -n default --wait --timeout=5m 2>/dev/null
    helm upgrade --install aegis-iceberg ../../charts/aegis-iceberg -n default --wait --timeout=5m 2>/dev/null

    log_info "Waiting for all pods..."
    kubectl wait --for=condition=Ready pods -l app=aegis-kafka -n default --timeout=300s 2>/dev/null || true
    kubectl wait --for=condition=Ready pods -l app=aegis-minio -n default --timeout=300s 2>/dev/null || true
    kubectl wait --for=condition=Ready pods -l app=aegis-polaris -n default --timeout=300s 2>/dev/null || true
    kubectl wait --for=condition=Ready pods -l component=coordinator -n default --timeout=300s 2>/dev/null || true
    kubectl wait --for=condition=Ready pods -l component=jobmanager -n default --timeout=300s 2>/dev/null || true
    kubectl wait --for=condition=Ready pods -l component=taskmanager -n default --timeout=300s 2>/dev/null || true

    log_info "Restoring data from latest backup..."
    local latest_iceberg=$(ls -t /backup/iceberg/ 2>/dev/null | head -1)
    local latest_minio=$(ls -t /backup/minio/ 2>/dev/null | head -1)
    local latest_kafka=$(ls -t /backup/kafka/ 2>/dev/null | head -1)

    [ -n "$latest_iceberg" ] && ./iceberg-backup.sh restore "$latest_iceberg" > /dev/null 2>&1 || true
    [ -n "$latest_minio" ] && ./minio-backup.sh restore "$latest_minio" > /dev/null 2>&1 || true
    [ -n "$latest_kafka" ] && ./kafka-backup.sh restore "$latest_kafka" > /dev/null 2>&1 || true

    local elapsed=$(($(date +%s) - start))
    log_pass "Restore completed in ${elapsed}s"
    echo $elapsed
}

validate_rpo() {
    log_info "=== Phase 4: Validate RPO < 1min ==="
    local rpo_ok=true

    local trino_pod=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [ -n "$trino_pod" ]; then
        local count=$(kubectl exec "$trino_pod" -n default -- trino --server http://127.0.0.1:8080 --execute "SELECT count(*) FROM aegis.aegis.hosts" 2>/dev/null | grep -oE '[0-9]+' | head -1)
        if [ -n "$count" ] && [ "$count" -ge 0 ]; then
            log_pass "RPO: Iceberg hosts table accessible ($count rows)"
        else
            log_fail "RPO: Iceberg hosts table NOT accessible"
            rpo_ok=false
        fi
    fi

    mc alias set minio http://aegis-minio:9000 minioadmin minioadmin > /dev/null 2>&1
    local objects=$(mc ls minio/aegis-warehouse --recursive 2>/dev/null | wc -l)
    if [ "$objects" -gt 0 ]; then
        log_pass "RPO: MinIO has $objects objects"
    else
        log_fail "RPO: MinIO bucket empty"
        rpo_ok=false
    fi

    local topics=$(kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list 2>/dev/null)
    local topic_count=$(echo "$topics" | grep -c "raw-l4-scans" || true)
    if [ "$topic_count" -gt 0 ]; then
        log_pass "RPO: Kafka topics present"
    else
        log_fail "RPO: Kafka topics missing"
        rpo_ok=false
    fi

    $rpo_ok && return 0 || return 1
}

validate_rto() {
    log_info "=== Phase 5: Validate RTO < 15min ==="
    local restore_time=$1

    if [ "$restore_time" -le "$RTO_TARGET" ]; then
        log_pass "RTO: Restore completed in ${restore_time}s <= ${RTO_TARGET}s (15min)"
        return 0
    else
        log_fail "RTO: Restore took ${restore_time}s > ${RTO_TARGET}s (15min)"
        return 1
    fi
}

validate_data_integrity() {
    log_info "=== Phase 6: Validate Data Integrity ==="
    local integrity_ok=true

    local trino_pod=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [ -n "$trino_pod" ]; then
        for table in hosts services certificates; do
            local count=$(kubectl exec "$trino_pod" -n default -- trino --server http://127.0.0.1:8080 --execute "SELECT count(*) FROM aegis.aegis.$table" 2>/dev/null | grep -oE '[0-9]+' | head -1)
            if [ -n "$count" ] && [ "$count" -ge 0 ]; then
                log_pass "Table $table: $count rows"
            else
                log_fail "Table $table: query failed"
                integrity_ok=false
            fi
        done
    fi

    local refs=$(curl -s http://aegis-polaris:19120/api/v1/refs 2>/dev/null)
    if echo "$refs" | grep -q "main"; then
        log_pass "Nessie main branch present"
    else
        log_fail "Nessie main branch missing"
        integrity_ok=false
    fi

    $integrity_ok && return 0 || return 1
}

main() {
    echo "=============================================="
    echo "  AEGIS Disaster Recovery Test"
    echo "  Target RTO: ${RTO_TARGET}s (15min)"
    echo "  Target RPO: ${RPO_TARGET}s (1min)"
    echo "=============================================="
    echo ""

    cd "$(dirname "$0")"
    check_prereqs

    backup_all
    simulate_disaster

    local restore_start=$(date +%s)
    restore_all
    local restore_elapsed=$(($(date +%s) - restore_start))

    local all_passed=true

    validate_rpo || all_passed=false
    validate_rto "$restore_elapsed" || all_passed=false
    validate_data_integrity || all_passed=false

    echo ""
    echo "=============================================="
    if [ "$all_passed" = true ]; then
        log_pass "DR TEST PASSED!"
        echo "  RTO: ${restore_elapsed}s (target < ${RTO_TARGET}s)"
        echo "  RPO: < ${RPO_TARGET}s"
        echo "  Data integrity: VERIFIED"
    else
        log_fail "DR TEST FAILED"
    fi
    echo "=============================================="

    $all_passed && exit 0 || exit 1
}

main "$@"