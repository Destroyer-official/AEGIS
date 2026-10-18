#!/bin/bash
# Deploy AEGIS Disaster Recovery Components

set -euo pipefail

echo "=============================================="
echo "  Deploying AEGIS DR Components"
echo "=============================================="
echo ""

# Create backup storage PVC
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: dr-backup-pvc
  namespace: default
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
  storageClassName: standard
EOF

echo "--- DR Backup PVC created ---"

# Make scripts executable
chmod +x iceberg-backup.sh minio-backup.sh kafka-backup.sh dr_test.sh

# Create CronJob for scheduled backups (every hour)
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: CronJob
metadata:
  name: aegis-dr-backup
  namespace: default
spec:
  schedule: "0 * * * *"  # Every hour
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 1
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: default
          containers:
            - name: backup
              image: bitnami/kubectl:latest
              command:
                - /bin/bash
                - -c
                - |
                  cd /dr-scripts
                  ./iceberg-backup.sh backup
                  ./minio-backup.sh backup
                  ./kafka-backup.sh backup
              volumeMounts:
                - name: dr-scripts
                  mountPath: /dr-scripts
                - name: backup-storage
                  mountPath: /backup
          restartPolicy: OnFailure
          volumes:
            - name: dr-scripts
              configMap:
                name: dr-backup-scripts
            - name: backup-storage
              persistentVolumeClaim:
                claimName: dr-backup-pvc
EOF

echo "--- Backup CronJob created (hourly) ---"

# Create ConfigMap with DR scripts
kubectl create configmap dr-backup-scripts \
    --from-file=iceberg-backup.sh \
    --from-file=minio-backup.sh \
    --from-file=kafka-backup.sh \
    -n default --dry-run=client -o yaml | kubectl apply -f -

echo "--- DR scripts ConfigMap created ---"

echo ""
echo "=============================================="
echo "  DR Components Deployed!"
echo "=============================================="
echo ""
echo "Scheduled backups: Every hour (CronJob)"
echo "Backup storage: 50Gi PVC"
echo ""
echo "Manual commands:"
echo "  Backup:    ./iceberg-backup.sh backup && ./minio-backup.sh backup && ./kafka-backup.sh backup"
echo "  Restore:   ./iceberg-backup.sh restore <timestamp> && ./minio-backup.sh restore <timestamp> && ./kafka-backup.sh restore <timestamp>"
echo "  DR Test:   ./dr_test.sh"
echo ""
echo "RTO Target: < 15 minutes"
echo "RPO Target: < 1 minute"