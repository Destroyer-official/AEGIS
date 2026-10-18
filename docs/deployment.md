# AEGIS Deployment Guide

**Version:** 1.0  
**Target:** Production / Staging / Local Development

---

## 1. Prerequisites

### 1.1 Infrastructure Requirements

| Component | Minimum | Recommended (Production) |
|-----------|---------|--------------------------|
| Kubernetes | 1.28+ | 1.29+ (EKS/GKE/AKS) |
| Nodes | 3 × 8 vCPU / 32 GB | 6+ × 16 vCPU / 64 GB |
| Storage | 500 GB SSD | 2+ TB NVMe (MinIO) |
| Network | 10 Gbps | 25-100 Gbps |
| Load Balancer | Layer 4/7 | NLB + ALB |

### 1.2 Software Dependencies

| Tool | Version |
|------|---------|
| kubectl | ≥ 1.28 |
| helm | ≥ 3.13 |
| kind | ≥ 0.23 (local only) |
| docker | ≥ 24.0 |
| protoc | 36+ |
| Rust | 1.98+ (stable) |
| Node.js | 26+ |
| Python | 3.14+ |

---

## 2. Local Development (Kind)

### 2.1 Bootstrap Cluster
```bash
cd infra
./scripts/bootstrap.sh
```

This creates:
- Kind cluster (`aegis-dev`) with port mappings
- Kafka (KRaft), Flink, MinIO, Trino, Nessie
- All Helm releases in `aegis` namespace

### 2.2 Verify Deployment
```bash
./scripts/verify.sh
```

Expected output:
```
✓ Kafka broker ready at localhost:9092
✓ Flink UI at http://localhost:8081
✓ Trino at http://localhost:8080
✓ MinIO API at http://localhost:9000
✓ Nessie catalog at http://localhost:19120
✓ Iceberg tables: hosts, services, certificates
```

### 2.3 Generate Test Data
```bash
cd infra/synthetic-generator
cargo run --release -- \
  --kafka-broker localhost:9092 \
  --topics raw-l4-scans,tls-fingerprints,l7-banners \
  --rate 1000 \
  --duration 60
```

### 2.4 Access Services
| Service | URL | Credentials |
|---------|-----|-------------|
| Trino | http://localhost:8080 | none |
| Flink UI | http://localhost:8081 | none |
| MinIO Console | http://localhost:9001 | minioadmin / minioadmin |
| Grafana | http://localhost:3000 | admin / admin |
| Gateway API | http://localhost:3000 | JWT |
| Dashboard | http://localhost:5173 | none |

### 2.5 Teardown
```bash
./scripts/teardown.sh
```

---

## 3. Production Deployment (Kubernetes)

### 3.1 Cluster Preparation

```bash
# Create namespace
kubectl create namespace aegis

# Install cert-manager (for TLS)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml

# Install ingress-nginx
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.10.0/deploy/static/provider/cloud/deploy.yaml
```

### 3.2 Secrets Management

```bash
# Create secrets from Vault/SealedSecrets
kubectl create secret generic aegis-secrets \
  --from-literal=jwt-secret="$(openssl rand -base64 64)" \
  --from-literal=trino-password="$(openssl rand -base64 32)" \
  --from-literal=minio-access-key="$(openssl rand -base64 16)" \
  --from-literal=minio-secret-key="$(openssl rand -base64 32)" \
  --from-literal=audit-signing-key="$(openssl rand -base64 32)" \
  -n aegis

# TLS certificates (cert-manager)
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: aegis-tls
  namespace: aegis
spec:
  secretName: aegis-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
    - api.aegis.example.com
    - dashboard.aegis.example.com
EOF
```

### 3.3 Helm Deployment

```bash
cd infra/charts

# Deploy dependencies first
helm dependency update aegis-kafka
helm dependency update aegis-flink
helm dependency update aegis-minio
helm dependency update aegis-polaris
helm dependency update aegis-trino

# Deploy in order
helm install aegis-kafka ./aegis-kafka -n aegis --wait
helm install aegis-minio ./aegis-minio -n aegis --wait
helm install aegis-polaris ./aegis-polaris -n aegis --wait
helm install aegis-flink ./aegis-flink -n aegis --wait
helm install aegis-trino ./aegis-trino -n aegis --wait
helm install aegis-iceberg ./aegis-iceberg -n aegis --wait
```

### 3.4 Application Deployments

```bash
# Build and push images
docker build -t ghcr.io/your-org/aegis-gateway:1.0.0 gateway/
docker build -t ghcr.io/your-org/aegis-dashboard:1.0.0 dashboard/
docker build -t ghcr.io/your-org/aegis-synthetic-generator:1.0.0 infra/synthetic-generator/
docker push ghcr.io/your-org/aegis-gateway:1.0.0
docker push ghcr.io/your-org/aegis-dashboard:1.0.0
docker push ghcr.io/your-org/aegis-synthetic-generator:1.0.0

# Deploy applications
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aegis-gateway
  namespace: aegis
spec:
  replicas: 3
  selector:
    matchLabels:
      app: aegis-gateway
  template:
    metadata:
      labels:
        app: aegis-gateway
    spec:
      containers:
      - name: gateway
        image: ghcr.io/your-org/aegis-gateway:1.0.0
        ports:
        - containerPort: 3000
        - containerPort: 50051
        env:
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: aegis-secrets
              key: jwt-secret
        - name: TRINO_COORDINATOR_URL
          value: "http://trino.aegis:8080"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: aegis-gateway
  namespace: aegis
spec:
  selector:
    app: aegis-gateway
  ports:
  - name: http
    port: 80
    targetPort: 3000
  - name: grpc
    port: 50051
    targetPort: 50051
  type: ClusterIP
EOF

# Dashboard
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aegis-dashboard
  namespace: aegis
spec:
  replicas: 2
  selector:
    matchLabels:
      app: aegis-dashboard
  template:
    metadata:
      labels:
        app: aegis-dashboard
    spec:
      containers:
      - name: dashboard
        image: ghcr.io/your-org/aegis-dashboard:1.0.0
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: aegis-dashboard
  namespace: aegis
spec:
  selector:
    app: aegis-dashboard
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
EOF
```

### 3.5 Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: aegis-ingress
  namespace: aegis
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.aegis.example.com
    - dashboard.aegis.example.com
    secretName: aegis-tls
  rules:
  - host: api.aegis.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: aegis-gateway
            port:
              number: 80
  - host: dashboard.aegis.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: aegis-dashboard
            port:
              number: 80
```

---

## 4. Synthetic Generator (Continuous)

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: aegis-synthetic-generator
  namespace: aegis
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: generator
            image: ghcr.io/your-org/aegis-synthetic-generator:1.0.0
            args:
            - --kafka-broker
            - kafka.aegis:9092
            - --topics
            - raw-l4-scans,tls-fingerprints,l7-banners
            - --rate
            - "5000"
            - --duration
            - "300"
            env:
            - name: RUST_LOG
              value: info
          restartPolicy: OnFailure
  schedule: "*/5 * * * *"
```

---

## 5. Monitoring & Alerting

### 5.1 Prometheus Rules
```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: aegis-alerts
  namespace: aegis
spec:
  groups:
  - name: aegis
    rules:
    - alert: AEGISGatewayDown
      expr: up{job="aegis-gateway"} == 0
      for: 2m
      labels:
        severity: critical
      annotations:
        summary: "AEGIS Gateway down"
    - alert: AEGISFlinkLagHigh
      expr: kafka_consumer_lag_seconds{job="aegis-flink"} > 300
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "Flink consumer lag > 5min"
    - alert: AEGISIcebergCompactionLag
      expr: iceberg_pending_compaction_files > 1000
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: "Iceberg compaction backlog"
    - alert: AEGISOptOutViolation
      expr: increase(aegis_opt_out_violations_total[5m]) > 0
      labels:
        severity: critical
      annotations:
        summary: "Opt-out violation detected"
```

### 5.2 Grafana Dashboards
- Import `infra/monitoring/grafana-dashboards/*.json`
- Key dashboards: Gateway, Flink, Iceberg, Targeting ML, Synthetic Generator

---

## 6. Backup & Disaster Recovery

### 6.1 Iceberg Backup
```bash
# Snapshot-based backup (MinIO replication)
mc mirror --overwrite minio/aegis/iceberg s3://backup-bucket/aegis/iceberg/

# Catalog backup (Nessie)
curl -X POST http://nessie.aegis:19120/api/v1/branches/main/tags/backup-$(date +%Y%m%d)
```

### 6.2 Recovery Procedure
```bash
# 1. Restore MinIO from backup
mc mirror --overwrite s3://backup-bucket/aegis/iceberg minio/aegis/iceberg/

# 2. Reset Nessie to backup tag
curl -X POST http://nessie:19120/api/v1/branches/main/assign \
  -H "Content-Type: application/json" \
  -d '{"hash": "<backup-tag-hash>"}'

# 3. Restart Flink to reprocess from last checkpoint
kubectl rollout restart deployment/aegis-flink-taskmanager -n aegis
```

---

## 7. Security Hardening

### 7.1 Network Policies
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: aegis-gateway-ingress
  namespace: aegis
spec:
  podSelector:
    matchLabels:
      app: aegis-gateway
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
    - protocol: TCP
      port: 50051
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: aegis-deny-all-egress
  namespace: aegis
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: aegis
    - to:
    - namespaceSelector:
        matchLabels:
          name: monitoring
  ports:
  - protocol: TCP
    port: 9090  # Prometheus
```

### 7.2 Pod Security Standards
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: aegis
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
```

---

## 8. Scaling Guidelines

| Component | Metric | Scale Action |
|-----------|--------|--------------|
| Gateway | CPU > 70% | HPA: replicas 3→10 |
| Gateway | P99 latency > 500ms | HPA: replicas +2 |
| Flink | Consumer lag > 60s | TaskManager +1 |
| Kafka | Disk > 70% | Add brokers, increase partitions |
| MinIO | Disk > 70% | Add nodes, rebalance |
| Trino | Query queue > 50 | Add workers |
| ML Serving | P99 > 200ms | HPA +2, Triton replica |

---

## 9. Rollback Procedure

```bash
# Gateway rollback
kubectl rollout undo deployment/aegis-gateway -n aegis

# Flink job rollback
kubectl rollout undo deployment/aegis-flink-taskmanager -n aegis

# Iceberg table rollback (time-travel)
# In Trino:
# ALTER TABLE aegis.hosts SET TBLPROPERTIES ('snapshot-id' = '12345');
```

---

## 10. Cost Optimization

| Component | Monthly Estimate (US-East) | Optimization |
|-----------|---------------------------|--------------|
| EKS (3× m6i.2xlarge) | ~$1,500 | Spot instances for batch |
| Kafka (3× r6g.xlarge) | ~$600 | Tiered storage to S3 |
| MinIO (3× r6g.2xlarge) | ~$1,200 | Erasure coding, lifecycle to Glacier |
| Trino (3× r6g.xlarge) | ~$600 | Auto-scaling, query caching |
| Flink (3× r6g.xlarge) | ~$600 | Reactive scaling |
| **Total** | **~$5,100/mo** | **~$3,500 with spot + tiering** |

---

## 11. Troubleshooting

| Symptom | Check | Resolution |
|---------|-------|------------|
| Gateway 503 | `kubectl logs -l app=aegis-gateway` | Check Trino connectivity, JWT secret |
| Flink lag | Flink UI → Backpressure tab | Increase parallelism, check Kafka |
| Iceberg write failures | Flink task logs | Check MinIO connectivity, Nessie |
| ML serving 500 | `kubectl logs -l app=aegis-ml-serving` | Check model path, GPU memory |
| Synthetic generator 0 events | Generator logs | Verify Kafka connectivity, topic names |