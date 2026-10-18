#!/bin/bash
# Deploy AEGIS Multi-Cluster Federation
# Run: ./deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=============================================="
echo "  Deploying AEGIS Multi-Cluster Federation"
echo "=============================================="
echo ""

echo "--- Creating federation namespace ---"
kubectl create namespace federation-system --dry-run=client -o yaml | kubectl apply -f -

echo "--- Applying ClusterRegistry CRD ---"
kubectl apply -f "$SCRIPT_DIR/cluster-registry-crd.yaml"

echo "--- Deploying federation controllers ---"
kubectl apply -f "$SCRIPT_DIR/deploy.yaml"

echo "Waiting for controllers..."
kubectl rollout status deployment/cluster-registry-controller -n federation-system --timeout=120s
kubectl rollout status deployment/scan-scheduler -n federation-system --timeout=120s
kubectl rollout status deployment/trino-federation-sync -n federation-system --timeout=120s

echo ""
echo "--- Creating sample ClusterRegistry ---"
cat <<EOF | kubectl apply -f -
apiVersion: aegis.io/v1alpha1
kind: ClusterRegistry
metadata:
  name: aegis-clusters
spec:
  clusters:
    - name: aegis-primary
      endpoint: https://kubernetes.default.svc
      region: us-east-1
      isPrimary: true
      kubeconfigSecret: aegis-primary-kubeconfig
      labels:
        environment: production
        tier: primary
      capabilities: [high-memory]
    - name: aegis-secondary
      endpoint: https://kubernetes.default.svc
      region: us-east-2
      isPrimary: false
      kubeconfigSecret: aegis-secondary-kubeconfig
      labels:
        environment: production
        tier: secondary
      capabilities: [spot]
  scanScheduling:
    strategy: closest
    targetProximity:
      enabled: true
      maxLatencyMs: 100
  trinoFederation:
    enabled: true
    catalogName: aegis-federated
    syncInterval: "300"
EOF

echo ""
echo "=============================================="
echo "  Multi-Cluster Federation Deployed!"
echo "=============================================="
echo ""
echo "Components:"
echo "  - ClusterRegistry CRD (aegis.io/v1alpha1)"
echo "  - Cluster Registry Controller (health checks, status updates)"
echo "  - Federated Scan Scheduler (closest/round-robin/least-loaded/cost-optimized)"
echo "  - Trino Federation Sync (cross-cluster catalog)"
echo ""
echo "Verify:"
echo "  kubectl get clusterregistries"
echo "  kubectl describe cr aegis-clusters"
echo "  kubectl logs -n federation-system -l app=cluster-registry-controller"
echo ""
echo "Sample ClusterRegistry created: aegis-clusters"
echo "  - aegis-primary (us-east-1, primary, high-memory)"
echo "  - aegis-secondary (us-east-2, secondary, spot)"