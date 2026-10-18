#!/usr/bin/env bash
set -euo pipefail

echo "Creating Kafka topics for AEGIS..."

TOPICS=(
    "raw-l4-scans:3:1:86400000"
    "tls-fingerprints:3:1:86400000"
    "l7-banners:3:1:86400000"
    "scan-commands:3:1:86400000"
    "enrichment-events:3:1:86400000"
)

KAFKA_BROKER="${KAFKA_BROKER:-aegis-kafka:9092}"

for topic_spec in "${TOPICS[@]}"; do
    IFS=':' read -r name partitions replication retention <<< "$topic_spec"
    echo "Creating topic: $name (partitions=$partitions, replication=$replication)"
    
    kubectl exec -it kafka-0 -- /opt/bitnami/kafka/bin/kafka-topics.sh \
        --bootstrap-server "$KAFKA_BROKER" \
        --create \
        --if-not-exists \
        --topic "$name" \
        --partitions "$partitions" \
        --replication-factor "$replication" \
        --config retention.ms="$retention" \
        --config cleanup.policy="delete" \
        2>/dev/null || echo "  Topic $name may already exist"
done

echo ""
echo "Listing all topics:"
kubectl exec -it kafka-0 -- /opt/bitnami/kafka/bin/kafka-topics.sh \
    --bootstrap-server "$KAFKA_BROKER" \
    --list

echo ""
echo "Topic creation complete"
