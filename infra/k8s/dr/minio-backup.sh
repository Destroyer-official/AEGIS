#!/bin/bash
# AEGIS Disaster Recovery - MinIO Backup & Restore
# RTO < 15min, RPO < 1min

set -euo pipefail

BACKUP_DIR="/backup/minio"
MINIO_ALIAS="minio"
BUCKET="aegis-warehouse"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

log_info() { echo -e "\033[0;34m[INFO]\033[0m $1"; }
log_pass() { echo -e "\033[0;32m[PASS]\033[0m $1"; }
log_fail() { echo -e "\033[0;31m[FAIL]\033[0m $1"; }
log_warn() { echo -e "\033[1;33m[WARN]\033[0m $1"; }

mkdir -p "$BACKUP_DIR/$TIMESTAMP"

configure_mc() {
    mc alias set $MINIO_ALIAS http://aegis-minio:9000 minioadmin minioadmin > /dev/null 2>&1
}

backup_minio() {
    log_info "Backing up MinIO bucket: $BUCKET"
    configure_mc

    mc mirror --overwrite $MINIO_ALIAS/$BUCKET $BACKUP_DIR/$TIMESTAMP/$BUCKET/ > /dev/null 2>&1
    log_pass "MinIO bucket mirrored to $BACKUP_DIR/$TIMESTAMP/$BUCKET"

    mc version info $MINIO_ALIAS/$BUCKET > "$BACKUP_DIR/$TIMESTAMP/version-info.txt" 2>/dev/null || true
    log_pass "Version info saved"
}

restore_minio() {
    local backup_path=$1
    log_info "Restoring MinIO bucket from $backup_path"
    configure_mc

    log_warn "Removing existing bucket content..."
    mc rm --recursive --force $MINIO_ALIAS/$BUCKET 2>/dev/null || true

    mc mirror --overwrite $backup_path/$BUCKET $MINIO_ALIAS/$BUCKET/ > /dev/null 2>&1
    log_pass "MinIO bucket restored from $backup_path"
}

verify_restore() {
    log_info "Verifying MinIO restore..."
    configure_mc

    local objects=$(mc ls $MINIO_ALIAS/$BUCKET --recursive 2>/dev/null | wc -l)
    if [ "$objects" -gt 0 ]; then
        log_pass "MinIO bucket has $objects objects restored"
    else
        log_fail "MinIO bucket appears empty"
        return 1
    fi

    for table in hosts services certificates; do
        if mc ls $MINIO_ALIAS/$BUCKET/$table 2>/dev/null | grep -q "metadata"; then
            log_pass "Iceberg table $table metadata present"
        else
            log_fail "Iceberg table $table metadata MISSING"
            return 1
        fi
    done

    return 0
}

case "${1:-}" in
    backup)
        log_info "Starting MinIO backup..."
        backup_minio
        log_pass "MinIO backup completed: $BACKUP_DIR/$TIMESTAMP"
        ;;
    restore)
        if [ -z "${2:-}" ]; then
            echo "Usage: $0 restore <backup_timestamp>"
            exit 1
        fi
        log_info "Starting MinIO restore from $2..."
        if [ -d "$BACKUP_DIR/$2" ]; then
            restore_minio "$BACKUP_DIR/$2"
            verify_restore
            log_pass "MinIO restore completed"
        else
            log_fail "Backup directory not found: $BACKUP_DIR/$2"
            exit 1
        fi
        ;;
    list)
        ls -la "$BACKUP_DIR/"
        ;;
    *)
        echo "Usage: $0 {backup|restore|list} [timestamp]"
        exit 1
        ;;
esac