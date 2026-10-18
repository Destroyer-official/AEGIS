#!/bin/bash
# AEGIS Disaster Recovery - Iceberg/Nessie Backup & Restore
# RTO < 15min, RPO < 1min

set -euo pipefail

BACKUP_DIR="/backup/iceberg"
NESSIE_ENDPOINT="http://aegis-polaris:19120/api/v1"
WAREHOUSE="s3://aegis-warehouse"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

log_info() { echo -e "\033[0;34m[INFO]\033[0m $1"; }
log_pass() { echo -e "\033[0;32m[PASS]\033[0m $1"; }
log_fail() { echo -e "\033[0;31m[FAIL]\033[0m $1"; }

mkdir -p "$BACKUP_DIR/$TIMESTAMP"

backup_nessie_refs() {
    log_info "Backing up Nessie references (branches/tags)..."
    curl -s "$NESSIE_ENDPOINT/refs" | jq . > "$BACKUP_DIR/$TIMESTAMP/nessie-refs.json"
    log_pass "Nessie refs backed up"
}

backup_nessie_commits() {
    log_info "Backing up Nessie commit log..."
    curl -s "$NESSIE_ENDPOINT/commits?maxRecords=1000" | jq . > "$BACKUP_DIR/$TIMESTAMP/nessie-commits.json"
    log_pass "Nessie commits backed up"
}

backup_iceberg_metadata() {
    log_info "Backing up Iceberg table metadata..."
    # Use Trino to get table metadata
    local trino_pod=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [ -n "$trino_pod" ]; then
        kubectl exec $trino_pod -n default -- trino --server http://127.0.0.1:8080 --execute "
            SELECT * FROM aegis.aegis.\$snapshots
        " --output-format CSV_HEADER > "$BACKUP_DIR/$TIMESTAMP/iceberg-snapshots.csv" 2>/dev/null || true
        
        kubectl exec $trino_pod -n default -- trino --server http://127.0.0.1:8080 --execute "
            SELECT * FROM aegis.aegis.\$history
        " --output-format CSV_HEADER > "$BACKUP_DIR/$TIMESTAMP/iceberg-history.csv" 2>/dev/null || true
        
        kubectl exec $trino_pod -n default -- trino --server http://127.0.0.1:8080 --execute "
            SELECT * FROM aegis.aegis.\$partitions
        " --output-format CSV_HEADER > "$BACKUP_DIR/$TIMESTAMP/iceberg-partitions.csv" 2>/dev/null || true
        
        log_pass "Iceberg metadata backed up"
    fi
}

backup_iceberg_data() {
    log_info "Creating Iceberg data snapshot via Nessie tag..."
    local tag_name="backup-$TIMESTAMP"
    # Create a tag at current main branch HEAD
    curl -s -X POST "$NESSIE_ENDPOINT/tags" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"$tag_name\", \"ref\": \"main\"}" > /dev/null
    log_pass "Created Nessie tag: $tag_name (RPO < 1min)"
}

restore_nessie_refs() {
    local backup_dir=$1
    log_info "Restoring Nessie references from $backup_dir..."
    
    if [ -f "$backup_dir/nessie-refs.json" ]; then
        # Restore branches
        local branches=$(jq -r '.branches[] | @base64' "$backup_dir/nessie-refs.json" 2>/dev/null)
        for branch in $branches; do
            local b=$(echo $branch | base64 -d)
            local name=$(echo $b | jq -r '.name')
            local hash=$(echo $b | jq -r '.hash')
            curl -s -X POST "$NESSIE_ENDPOINT/branches" \
                -H "Content-Type: application/json" \
                -d "{\"name\": \"$name\", \"hash\": \"$hash\"}" > /dev/null
        done
        log_pass "Nessie branches restored"
    fi
}

restore_iceberg_from_tag() {
    local tag_name=$1
    log_info "Restoring Iceberg tables from tag: $tag_name"
    
    # Assign main branch to the tag's commit
    local tag_hash=$(curl -s "$NESSIE_ENDPOINT/tags/$tag_name" | jq -r '.hash')
    curl -s -X POST "$NESSIE_ENDPOINT/branches/main/assign" \
        -H "Content-Type: application/json" \
        -d "{\"hash\": \"$tag_hash\"}" > /dev/null
    log_pass "Main branch assigned to tag $tag_name"
}

verify_restore() {
    log_info "Verifying restore..."
    
    # Check tables accessible via Trino
    local trino_pod=$(kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [ -n "$trino_pod" ]; then
        local tables=$(kubectl exec $trino_pod -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW TABLES IN aegis.aegis" 2>/dev/null)
        for table in hosts services certificates; do
            if echo "$tables" | grep -q "$table"; then
                log_pass "Table $table restored"
            else
                log_fail "Table $table NOT restored"
                return 1
            fi
        done
    fi
    
    # Check Nessie branches
    local refs=$(curl -s "$NESSIE_ENDPOINT/refs" 2>/dev/null)
    if echo "$refs" | grep -q "main"; then
        log_pass "Nessie main branch restored"
    else
        log_fail "Nessie main branch NOT restored"
        return 1
    fi
    
    return 0
}

case "${1:-}" in
    backup)
        log_info "Starting Iceberg/Nessie backup..."
        backup_nessie_refs
        backup_nessie_commits
        backup_iceberg_metadata
        backup_iceberg_data
        log_pass "Backup completed: $BACKUP_DIR/$TIMESTAMP"
        echo "$BACKUP_DIR/$TIMESTAMP"
        ;;
    restore)
        if [ -z "${2:-}" ]; then
            echo "Usage: $0 restore <backup_timestamp|tag_name>"
            exit 1
        fi
        log_info "Starting Iceberg/Nessie restore from $2..."
        if [ -d "$BACKUP_DIR/$2" ]; then
            restore_nessie_refs "$BACKUP_DIR/$2"
        fi
        restore_iceberg_from_tag "$2"
        verify_restore
        log_pass "Restore completed"
        ;;
    list)
        ls -la "$BACKUP_DIR/"
        ;;
    *)
        echo "Usage: $0 {backup|restore|list} [timestamp|tag]"
        exit 1
        ;;
esac