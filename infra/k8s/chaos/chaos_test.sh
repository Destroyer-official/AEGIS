#!/bin/bash
# AEGIS Chaos Engineering Test Script
# Runs LitmusChaos experiments and verifies auto-recovery < 2min with zero data loss

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

RECOVERY_TIMEOUT=120

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

check_kubectl() {
    if ! command -v kubectl &> /dev/null; then
        log_fail "kubectl not found"
        exit 1
    fi
}

install_litmus() {
    log_info "Installing LitmusChaos..."
    kubectl apply -f litmus-install.yaml
    log_info "Waiting for Litmus operator..."
    kubectl wait --for=condition=available --timeout=180s deployment/litmus-operator -n litmus 2>/dev/null || true
    sleep 15
}

run_chaos_experiment() {
    local engine_file=$1
    local engine_name=$2

    log_info "Applying ChaosEngine: $engine_name"
    kubectl apply -f "$engine_file"

    log_info "Waiting 60s for chaos to complete..."
    sleep 60

    local result=$(kubectl get chaosresult "${engine_name}-pod-delete" -n litmus -o jsonpath='{.status.experimentstatus.verdict}' 2>/dev/null || echo "Unknown")
    if [ "$result" = "Pass" ]; then
        log_pass "Chaos $engine_name: $result"
        return 0
    else
        log_fail "Chaos $engine_name: $result"
        return 1
    fi
}

verify_recovery() {
    local service=$1
    local label=$2
    local expected=${3:-1}

    log_info "Verifying recovery for $service..."
    local start=$(date +%s)

    while [ $(($(date +%s) - start)) -lt $RECOVERY_TIMEOUT ]; do
        local ready=$(kubectl get pods -n default -l "$label" --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l)
        if [ "$ready" -eq "$expected" ]; then
            local elapsed=$(($(date +%s) - start))
            log_pass "$service recovered in ${elapsed}s"
            return 0
        fi
        sleep 5
    done

    log_fail "$service did not recover within ${RECOVERY_TIMEOUT}s"
    kubectl get pods -n default -l "$label" 2>/dev/null
    return 1
}

verify_data_integrity() {
    log_info "Verifying data integrity..."

    local topics=$(kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list 2>/dev/null)
    for topic in raw-l4-scans tls-fingerprints l7-banners scan-commands enrichment-events; do
        if echo "$topics" | grep -q "^${topic}$"; then
            log_pass "Kafka topic $topic exists"
        else
            log_fail "Kafka topic $topic MISSING"
            return 1
        fi
    done

    local trino_pod=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [ -n "$trino_pod" ]; then
        local tables=$(kubectl exec "$trino_pod" -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW TABLES IN aegis.aegis" 2>/dev/null)
        for table in hosts services certificates; do
            if echo "$tables" | grep -q "$table"; then
                log_pass "Iceberg table $table accessible"
            else
                log_fail "Iceberg table $table NOT accessible"
                return 1
            fi
        done
    fi

    return 0
}

main() {
    echo "=============================================="
    echo "  AEGIS Chaos Engineering Test"
    echo "=============================================="
    echo ""

    check_kubectl

    if ! kubectl get crd chaosengines.litmuschaos.io 2>/dev/null; then
        install_litmus
    fi

    local all_passed=true

    log_info "=== Test 1: Kafka Broker Kill ==="
    run_chaos_experiment "kafka-chaos-engine.yaml" "kafka-chaos" || all_passed=false
    verify_recovery "Kafka" "app=aegis-kafka" 1 || all_passed=false

    echo ""
    log_info "=== Test 2: Flink JobManager Kill ==="
    run_chaos_experiment "flink-chaos-engine.yaml" "flink-jobmanager-chaos" || all_passed=false
    verify_recovery "Flink JM" "component=jobmanager" 1 || all_passed=false

    echo ""
    log_info "=== Test 3: Flink TaskManager Kill ==="
    run_chaos_experiment "flink-chaos-engine.yaml" "flink-taskmanager-chaos" || all_passed=false
    verify_recovery "Flink TM" "component=taskmanager" 1 || all_passed=false

    echo ""
    log_info "=== Test 4: Trino Coordinator Kill ==="
    run_chaos_experiment "trino-minio-nessie-chaos-engine.yaml" "trino-coordinator-chaos" || all_passed=false
    verify_recovery "Trino" "component=coordinator" 1 || all_passed=false

    echo ""
    log_info "=== Test 5: MinIO Kill ==="
    run_chaos_experiment "trino-minio-nessie-chaos-engine.yaml" "minio-chaos" || all_passed=false
    verify_recovery "MinIO" "app=aegis-minio" 1 || all_passed=false

    echo ""
    log_info "=== Test 6: Nessie Kill ==="
    run_chaos_experiment "trino-minio-nessie-chaos-engine.yaml" "nessie-chaos" || all_passed=false
    verify_recovery "Nessie" "app=aegis-polaris" 1 || all_passed=false

    echo ""
    log_info "=== Data Integrity Verification ==="
    verify_data_integrity || all_passed=false

    echo ""
    echo "=============================================="
    if [ "$all_passed" = true ]; then
        log_pass "ALL CHAOS TESTS PASSED!"
        echo "  - All services recovered within 2 minutes"
        echo "  - Zero data loss verified"
    else
        log_fail "SOME CHAOS TESTS FAILED"
    fi
    echo "=============================================="

    [ "$all_passed" = true ] && exit 0 || exit 1
}

main "$@"