#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
CHARTS_DIR="$INFRA_DIR/charts"
KIND_CONFIG="$INFRA_DIR/kind/kind-config.yaml"

echo "============================================="
echo "  AEGIS Infrastructure Bootstrap"
echo "============================================="

# 1. Create kind cluster
echo ""
echo "[1/7] Creating kind cluster..."
if kind get clusters 2>/dev/null | grep -q "^aegis$"; then
    echo "  Cluster 'aegis' already exists, skipping creation"
else
    kind create cluster --config "$KIND_CONFIG" --name aegis
fi

echo "  Waiting for cluster to be ready..."
kubectl cluster-info --context kind-aegis
kubectl wait --for=condition=Ready nodes --all --timeout=120s

# 2. Install Helm charts in dependency order
echo ""
echo "[2/7] Deploying MinIO..."
helm upgrade --install aegis-minio "$CHARTS_DIR/aegis-minio" \
    --namespace default --wait --timeout 120s

echo ""
echo "[3/7] Deploying Polaris..."
helm upgrade --install aegis-polaris "$CHARTS_DIR/aegis-polaris" \
    --namespace default --wait --timeout 120s

echo ""
echo "[4/7] Deploying Kafka (KRaft mode)..."
helm upgrade --install aegis-kafka "$CHARTS_DIR/aegis-kafka" \
    --namespace default --wait --timeout 300s

echo ""
echo "[5/7] Initializing Iceberg tables..."
helm upgrade --install aegis-iceberg "$CHARTS_DIR/aegis-iceberg" \
    --namespace default --wait --timeout 120s

echo ""
echo "[6/7] Deploying Flink cluster..."
helm upgrade --install aegis-flink "$CHARTS_DIR/aegis-flink" \
    --namespace default --wait --timeout 180s

echo ""
echo "[7/7] Deploying Trino..."
helm upgrade --install aegis-trino "$CHARTS_DIR/aegis-trino" \
    --namespace default --wait --timeout 180s

echo ""
echo "============================================="
echo "  All components deployed!"
echo "============================================="
echo ""
echo "  Kafka:      localhost:9092"
echo "  Flink UI:   http://localhost:8081"
echo "  Trino:      http://localhost:8080"
echo "  MinIO API:  http://localhost:9000"
echo "  MinIO UI:   http://localhost:9001"
echo "  Polaris:    http://localhost:8181"
echo ""
echo "Run ./scripts/verify.sh to validate all services"
