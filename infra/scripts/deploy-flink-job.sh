#!/bin/bash
# AEGIS Flink Job Deployment Script
# Run this AFTER building the fat JAR with: gradle shadowJar

set -euo pipefail

JAR_PATH="build/libs/aegis-flink-job-0.1.0.jar"
JOBMANAGER_POD=$(kubectl get pods -l component=jobmanager -n default -o jsonpath='{.items[0].metadata.name}')

echo "=== AEGIS Flink Job Deployment ==="
echo "JobManager pod: $JOBMANAGER_POD"
echo "JAR path: $JAR_PATH"

# Check if JAR exists
if [ ! -f "$JAR_PATH" ]; then
    echo "ERROR: JAR not found at $JAR_PATH"
    echo "Build it first with: gradle shadowJar"
    exit 1
fi

# Copy JAR to Flink JobManager
echo "Copying JAR to JobManager..."
kubectl cp "$JAR_PATH" "$JOBMANAGER_POD:/tmp/aegis-flink-job.jar" -n default

# Submit job to Flink
echo "Submitting Flink job..."
kubectl exec "$JOBMANAGER_POD" -n default -- \
    flink run -d /tmp/aegis-flink-job.jar

echo "Job submitted successfully!"
echo ""
echo "Check job status:"
echo "  kubectl exec $JOBMANAGER_POD -n default -- flink list"
echo ""
echo "Check Flink UI: kubectl port-forward svc/aegis-flink-jobmanager 8081:8081"