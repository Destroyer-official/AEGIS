#!/bin/bash
# Deploy AEGIS Kyverno Policies
# Run: ./deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=============================================="
echo "  Deploying AEGIS Kyverno Policies"
echo "=============================================="
echo ""

echo "--- Installing Kyverno ---"
kubectl create namespace kyverno --dry-run=client -o yaml | kubectl apply -f -

kubectl apply -f https://raw.githubusercontent.com/kyverno/kyverno/main/config/install-latest-testing.yaml 2>/dev/null || {
    echo "Using local Kyverno install..."
    kubectl apply -f "$SCRIPT_DIR/kyverno-install.yaml"
}

echo "Waiting for Kyverno..."
kubectl wait --for=condition=Ready pods -l app=kyverno -n kyverno --timeout=300s 2>/dev/null || true

echo ""
echo "--- Applying AEGIS Policies ---"
kubectl apply -f "$SCRIPT_DIR/require-resource-limits.yaml"
kubectl apply -f "$SCRIPT_DIR/disallow-privileged.yaml"
kubectl apply -f "$SCRIPT_DIR/require-labels-and-network.yaml"

echo ""
echo "=============================================="
echo "  Kyverno Policies Deployed!"
echo "=============================================="
echo ""
echo "Policies:"
echo "  - require-resource-limits (Enforce): All pods must have CPU/mem requests+limits"
echo "  - disallow-privileged-containers (Enforce): No privileged, hostNetwork, hostPID, hostIPC"
echo "  - require-network-policy (Audit): Default deny-all NetworkPolicy per namespace"
echo "  - require-labels (Audit): Pods must have 'app' label"
echo "  - restrict-image-registries (Audit): Only approved registries"
echo ""
echo "Verify:"
echo "  kubectl get clusterpolicies"
echo "  kubectl get events -n kyverno"