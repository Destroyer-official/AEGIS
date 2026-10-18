# AEGIS Monitoring Alert Rules Summary

## Kafka Alerts
| Alert | Expression | Severity | For |
|-------|------------|----------|-----|
| KafkaBrokerDown | `up{job="kafka"} == 0` | critical | 1m |
| KafkaUnderReplicatedPartitions | `kafka_server_replicamanager_underreplicatedpartitions > 0` | warning | 5m |
| KafkaOfflinePartitions | `kafka_controller_kafkacontroller_offlinepartitionscount > 0` | critical | 1m |
| KafkaConsumerLagHigh | `kafka_consumergroup_lag > 10000` | warning | 5m |

## Flink Alerts
| Alert | Expression | Severity | For |
|-------|------------|----------|-----|
| FlinkJobManagerDown | `up{job="flink-jobmanager"} == 0` | critical | 1m |
| FlinkTaskManagerDown | `up{job="flink-taskmanager"} == 0` | critical | 1m |
| FlinkJobFailed | `flink_jobmanager_job_status{status="FAILED"} == 1` | critical | 1m |
| FlinkCheckpointSlow | `flink_jobmanager_job_last_checkpoint_duration > 300000` | warning | 5m |
| FlinkCheckpointFailed | `flink_jobmanager_job_last_checkpoint_size < 0` | critical | 1m |

## Trino Alerts
| Alert | Expression | Severity | For |
|-------|------------|----------|-----|
| TrinoCoordinatorDown | `up{job="trino"} == 0` | critical | 1m |
| TrinoQueryQueueHigh | `trino_query_manager_queued_queries > 50` | warning | 5m |
| TrinoMemoryUsageHigh | `(trino_memory_pool_usage / trino_memory_pool_max) > 0.9` | warning | 5m |

## MinIO Alerts
| Alert | Expression | Severity | For |
|-------|------------|----------|-----|
| MinIODown | `up{job="minio"} == 0` | critical | 1m |
| MinIODiskUsageHigh | `(minio_disk_usage_bytes / minio_disk_capacity_bytes) > 0.85` | warning | 5m |
| MinIOBucketMissing | `minio_bucket_exists{bucket="aegis-warehouse"} == 0` | critical | 1m |

## Nessie Alerts
| Alert | Expression | Severity | For |
|-------|------------|----------|-----|
| NessieDown | `up{job="nessie"} == 0` | critical | 1m |

## Testing Alerts

### Verify Prometheus targets
```bash
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Open http://localhost:9090/targets
```

### Test specific alert rules
```bash
# Test Kafka broker down
kubectl exec -n monitoring deploy/prometheus -- promtool query instant 'up{job="kafka"}'

# Test Flink job status
kubectl exec -n monitoring deploy/prometheus -- promtool query instant 'flink_jobmanager_job_status{status="FAILED"}'

# Test Trino memory
kubectl exec -n monitoring deploy/prometheus -- promtool query instant '(trino_memory_pool_usage / trino_memory_pool_max) > 0.9'
```

### Simulate failures
```bash
# Simulate Kafka broker down
kubectl scale sts aegis-kafka --replicas=0 -n default

# Simulate Flink job failure (check Flink UI for failed job)
# Simulate Trino coordinator down
kubectl scale deploy aegis-trino-coordinator --replicas=0 -n default

# Simulate MinIO disk pressure
kubectl exec -it aegis-minio-0 -n default -- dd if=/dev/zero of=/data/test.img bs=1M count=4000
```

## Grafana Dashboard URLs (after port-forward)
```
kubectl port-forward -n monitoring svc/grafana 3000:3000
# Open http://localhost:3000
# Login: admin / aegis123

Dashboards available:
- AEGIS Kafka Monitoring
- AEGIS Flink Monitoring
- AEGIS Trino Monitoring
- AEGIS MinIO Monitoring
- AEGIS Nessie Monitoring
```

## Required Exporters (for production)

The current dashboards expect these metrics exporters. For kind/dev:

1. **Kafka**: JMX Exporter on port 7071 (add to Kafka pods)
2. **Flink**: Built-in metrics on port 9251 (enable in flink-conf.yaml)
3. **Trino**: Built-in `/metrics` endpoint on port 8080
4. **MinIO**: Built-in `/minio/v2/metrics/prometheus` on port 9000
5. **Nessie**: Micrometer/Prometheus endpoint on port 19120

For production, deploy:
- `strimzi/kafka-operator` for Kafka monitoring
- `flink-operator` with metrics enabled
- `trino-prometheus-connector` for Trino
- `minio-operator` with metrics