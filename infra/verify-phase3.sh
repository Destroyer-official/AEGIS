#!/bin/bash
# AEGIS Phase 3 Verification Script
# Verifies: ArgoCD, Kyverno, Cost Controller, Federation

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

check() {
    local desc=$1
    local cmd=$2
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}[PASS]${NC} $desc"
        ((PASS++))
    else
        echo -e "${RED}[FAIL]${NC} $desc"
        ((FAIL++))
    fi
}

warn() {
    local desc=$1
    echo -e "${YELLOW}[WARN]${NC} $desc"
    ((WARN++))
}

echo "=============================================="
echo "  AEGIS Phase 3 Verification"
echo "=============================================="
echo ""

echo "--- Day 1: ArgoCD GitOps ---"
check "Argocd namespace exists" "kubectl get namespace argocd"
check "ArgoCD server deployed" "kubectl get deployment argocd-server -n argocd"
check "ArgoCD repo-server deployed" "kubectl get deployment argocd-repo-server -n argocd"
check "ArgoCD appset controller deployed" "kubectl get deployment argocd-applicationset-controller -n argocd"
check "ArgoCD notifications deployed" "kubectl get deployment argocd-notifications-controller -n argocd"
echo ""

echo "--- ArgoCD Applications ---"
check "aegis-kafka app" "kubectl get application aegis-kafka -n argocd"
check "aegis-minio app" "kubectl get application aegis-minio -n argocd"
check "aegis-polaris app" "kubectl get application aegis-polaris -n argocd"
check "aegis-trino app" "kubectl get application aegis-trino -n argocd"
check "aegis-flink app" "kubectl get application aegis-flink -n argocd"
check "aegis-iceberg app" "kubectl get application aegis-iceberg -n argocd"
check "aegis-monitoring app" "kubectl get application aegis-monitoring -n argocd"
echo ""

echo "--- Day 1: Kyverno Policies ---"
check "Kyverno namespace exists" "kubectl get namespace kyverno"
check "Kyverno deployed" "kubectl get deployment kyverno -n kyverno"
check "Policy: require-resource-limits" "kubectl get clusterpolicy require-resource-limits"
check "Policy: disallow-privileged-containers" "kubectl get clusterpolicy disallow-privileged-containers"
check "Policy: require-network-policy" "kubectl get clusterpolicy require-network-policy"
check "Policy: require-labels" "kubectl get clusterpolicy require-labels"
check "Policy: restrict-image-registries" "kubectl get clusterpolicy restrict-image-registries"
echo ""

echo "--- Day 2: Cost Optimization Controller ---"
check "Cost namespace exists" "kubectl get namespace cost-system"
check "Cost controller deployed" "kubectl get deployment cost-controller -n cost-system"
check "Cost controller SA" "kubectl get serviceaccount cost-controller -n cost-system"
check "Cost controller ClusterRole" "kubectl get clusterrole cost-controller"
check "Cost controller ClusterRoleBinding" "kubectl get clusterrolebinding cost-controller"
echo ""

echo "--- Day 3: Multi-Cluster Federation ---"
check "Federation namespace exists" "kubectl get namespace federation-system"
check "ClusterRegistry CRD" "kubectl get crd clusterregistries.aegis.io"
check "Cluster Registry Controller" "kubectl get deployment cluster-registry-controller -n federation-system"
check "Scan Scheduler" "kubectl get deployment scan-scheduler -n federation-system"
check "Trino Federation Sync" "kubectl get deployment trino-federation-sync -n federation-system"
check "Sample ClusterRegistry" "kubectl get clusterregistry aegis-clusters"
echo ""

echo "--- Kyverno Policy Test ---"
cat <<EOF | kubectl apply -f - 2>/dev/null && echo -e "${YELLOW}[WARN]${NC} Kyverno did NOT block pod without resources (policy may not be active)" && ((WARN++)) || echo -e "${GREEN}[PASS]${NC} Kyverno correctly blocked pod without resources" && ((PASS++))
apiVersion: v1
kind: Pod
metadata:
  name: test-bad-pod
  namespace: default
spec:
  containers:
    - name: test
      image: busybox
      command: ["sleep", "1"]
EOF

# Cleanup test pod if it was created
kubectl delete pod test-bad-pod -n default --ignore-not-found=true 2>/dev/null || true

echo ""
echo "=============================================="
echo "  Results: $PASS passed, $FAIL failed, $WARN warnings"
echo "=============================================="

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}ALL CHECKS PASSED!${NC}"
    exit 0
else
    echo -e "${RED}SOME CHECKS FAILED${NC}"
    exit 1
fi