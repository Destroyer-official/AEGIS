#!/bin/bash
# Deploy ArgoCD + AEGIS GitOps Applications
# Run: ./deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=============================================="
echo "  Deploying ArgoCD + AEGIS GitOps"
echo "=============================================="
echo ""

# Install ArgoCD
echo "--- Installing ArgoCD ---"
kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -

# Install ArgoCD via official manifest
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml 2>/dev/null || {
    echo "Using local ArgoCD install..."
    kubectl apply -f "$SCRIPT_DIR/argocd-install.yaml"
}

# Wait for ArgoCD to be ready
echo "Waiting for ArgoCD server..."
kubectl wait --for=condition=Ready pods -l app.kubernetes.io/name=argocd-server -n argocd --timeout=300s

# Configure ArgoCD for insecure HTTP access
kubectl patch configmap argocd-cmd-params-cm -n argocd --type merge -p '{"data":{"server.insecure":"true"}}' 2>/dev/null || true

# Restart ArgoCD server to pick up config change
kubectl rollout restart deployment argocd-server -n argocd 2>/dev/null || true
sleep 10

# Apply AEGIS Applications
echo ""
echo "--- Applying AEGIS Application manifests ---"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-kafka.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-minio.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-polaris.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-trino.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-flink.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-iceberg.yaml"
kubectl apply -f "$SCRIPT_DIR/applications/aegis-monitoring.yaml"

echo ""
echo "=============================================="
echo "  ArgoCD + AEGIS GitOps Deployed!"
echo "=============================================="
echo ""
echo "Access ArgoCD:"
echo "  kubectl port-forward -n argocd svc/argocd-server 8080:443"
echo ""
echo "ArgoCD Login:"
echo "  User: admin"
echo "  Pass: \$(kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d)"
echo ""
echo "Applications deployed:"
echo "  - aegis-kafka       (Kafka streaming)"
echo "  - aegis-minio       (MinIO storage)"
echo "  - aegis-polaris     (Nessie catalog)"
echo "  - aegis-trino       (Trino query engine)"
echo "  - aegis-flink       (Flink processing)"
echo "  - aegis-iceberg     (Iceberg init)"
echo "  - aegis-monitoring  (Prometheus + Grafana)"
echo ""
echo "Verify:"
echo "  argocd app list"
echo "  argocd app get aegis-kafka"