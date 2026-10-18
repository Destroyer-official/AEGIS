#!/bin/bash
# Deploy AEGIS Cost Optimization Controller
# Run: ./deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=============================================="
echo "  Deploying AEGIS Cost Optimization Controller"
echo "=============================================="
echo ""

echo "--- Building cost controller image ---"
if command -v docker &> /dev/null; then
    docker build -t aegis/cost-controller:latest "$SCRIPT_DIR"
    kind load docker-image aegis/cost-controller:latest --name aegis 2>/dev/null || echo "Kind load skipped (cluster may not be running)"
else
    echo "Docker not available, using deploy.yaml as-is"
fi

echo "--- Deploying cost controller ---"
kubectl apply -f "$SCRIPT_DIR/deploy.yaml"

echo "Waiting for cost controller..."
kubectl rollout status deployment/cost-controller -n cost-system --timeout=120s

echo ""
echo "=============================================="
echo "  Cost Optimization Controller Deployed!"
echo "=============================================="
echo ""
echo "Check logs:"
echo "  kubectl logs -n cost-system -l app=cost-controller -f"
echo ""
echo "View reports:"
echo "  kubectl exec -n cost-system deploy/cost-controller -- ls /var/lib/cost-controller/reports/"
echo ""
echo "Features:"
echo "  - CPU/Memory right-sizing recommendations"
echo "  - PVC downsizing for idle volumes"
echo "  - Spot instance recommendations"
echo "  - Idle resource detection"
echo "  - Report generation every 5 minutes"