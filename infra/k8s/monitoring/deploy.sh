#!/bin/bash
# Deploy AEGIS Monitoring Stack (Prometheus + Grafana)
# Run: ./deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=============================================="
echo "  Deploying AEGIS Monitoring Stack"
echo "=============================================="
echo ""

# Create namespace
echo "--- Creating monitoring namespace ---"
kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -

# Apply ServiceMonitors
echo "--- Applying ServiceMonitors ---"
kubectl apply -f "$SCRIPT_DIR/servicemonitors/servicemonitors.yaml"

# Apply Prometheus
echo "--- Deploying Prometheus ---"
kubectl apply -f "$SCRIPT_DIR/prometheus.yaml"

# Wait for Prometheus to be ready
echo "Waiting for Prometheus..."
kubectl rollout status deployment/prometheus -n monitoring --timeout=120s

# Create Grafana dashboards ConfigMap from files
echo ""
echo "--- Creating Grafana dashboards ConfigMap ---"
kubectl create configmap grafana-dashboards \
    --namespace=monitoring \
    --from-file=kafka-dashboard.json="$SCRIPT_DIR/grafana/dashboards/kafka-dashboard.json" \
    --from-file=flink-dashboard.json="$SCRIPT_DIR/grafana/dashboards/flink-dashboard.json" \
    --from-file=trino-dashboard.json="$SCRIPT_DIR/grafana/dashboards/trino-dashboard.json" \
    --from-file=minio-dashboard.json="$SCRIPT_DIR/grafana/dashboards/minio-dashboard.json" \
    --from-file=k8s-dashboard.json="$SCRIPT_DIR/grafana/dashboards/k8s-dashboard.json" \
    --from-file=nessie-dashboard.json="$SCRIPT_DIR/grafana/dashboards/nessie-dashboard.json" \
    --dry-run=client -o yaml | kubectl apply -f -

# Apply Grafana
echo ""
echo "--- Deploying Grafana ---"
kubectl apply -f "$SCRIPT_DIR/grafana.yaml"

# Wait for Grafana to be ready
echo "Waiting for Grafana..."
kubectl rollout status deployment/grafana -n monitoring --timeout=120s

echo ""
echo "=============================================="
echo "  Monitoring Stack Deployed!"
echo "=============================================="
echo ""
echo "Access URLs (via port-forward):"
echo "  Prometheus: kubectl port-forward -n monitoring svc/prometheus 9090:9090"
echo "  Grafana:    kubectl port-forward -n monitoring svc/grafana 3000:3000"
echo ""
echo "Grafana Login:"
echo "  User: admin"
echo "  Pass: aegis123"
echo ""
echo "Pre-configured Dashboards (in AEGIS folder):"
echo "  - AEGIS Kafka Monitoring"
echo "  - AEGIS Flink Monitoring"
echo "  - AEGIS Trino Monitoring"
echo "  - AEGIS MinIO Monitoring"
echo "  - AEGIS Kubernetes Cluster"
echo "  - AEGIS Nessie Catalog"
echo ""
echo "Alert Rules (in Prometheus):"
echo "  - Kafka: Broker down, under-replicated, consumer lag, disk usage"
echo "  - Flink: JobManager/TaskManager down, job failed, checkpoint issues, backpressure"
echo "  - Trino: Coordinator down, query queue, memory usage, failed queries"
echo "  - MinIO: Down, disk usage, bucket missing, offline disks"
echo "  - Nessie: Down, API errors"
echo "  - K8s: Node not ready, pod not running, OOM killed, high CPU/memory"
echo ""
echo "ServiceMonitors applied for automatic metric discovery."