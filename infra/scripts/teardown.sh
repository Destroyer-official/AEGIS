#!/usr/bin/env bash
set -euo pipefail

echo "Tearing down AEGIS infrastructure..."

helm uninstall aegis-trino --ignore-not-found 2>/dev/null || true
helm uninstall aegis-flink --ignore-not-found 2>/dev/null || true
helm uninstall aegis-iceberg --ignore-not-found 2>/dev/null || true
helm uninstall aegis-kafka --ignore-not-found 2>/dev/null || true
helm uninstall aegis-polaris --ignore-not-found 2>/dev/null || true
helm uninstall aegis-minio --ignore-not-found 2>/dev/null || true

echo "Deleting kind cluster..."
kind delete cluster --name aegis 2>/dev/null || true

echo "Teardown complete"
