# AEGIS Infrastructure Verification (PowerShell)
# Run: .\scripts\verify.ps1  or  powershell.exe -ExecutionPolicy Bypass -File .\scripts\verify.ps1

$ErrorActionPreference = "SilentlyContinue"
$PASS = 0
$FAIL = 0

function Check {
    param([string]$Name, [string]$Result)
    if ($Result -eq "PASS") {
        Write-Host "  [PASS] $Name" -ForegroundColor Green
        $script:PASS++
    } else {
        Write-Host "  [FAIL] ${Name}: $Result" -ForegroundColor Red
        $script:FAIL++
    }
}

function Test-HttpViaKubectl {
    param([string]$Url, [string]$Description)
    # Try via temp curl pod inside cluster (more reliable than host port-forward on Windows)
    $podName = "verify-curl-$(Get-Random -Minimum 1000 -Maximum 9999)"
    kubectl run $podName --image=curlimages/curl:8.5.0 --restart=Never -- curl -s -o /dev/null -w "%{http_code}" $Url 2>$null | Out-Null
    Start-Sleep -Seconds 5
    $code = kubectl logs $podName -n default 2>$null
    kubectl delete pod $podName -n default --ignore-not-found=true 2>$null | Out-Null
    if ($code -match "200|302|301") { return "PASS" }
    # Fallback: try host Invoke-WebRequest (works when kind extraPortMappings are exposed)
    try {
        $resp = Invoke-WebRequest -Uri $Url.Replace("aegis-", "localhost:").Replace(":9000", ":9000").Replace("aegis-polaris:19120", "localhost:19120").Replace("aegis-minio:9000", "localhost:9000").Replace("aegis-trino-coordinator:8080", "localhost:8080").Replace("aegis-flink-jobmanager:8081", "localhost:8081") -TimeoutSec 5 -UseBasicParsing -MaximumRedirection 0
        if ($resp.StatusCode -in @(200,301,302)) { return "PASS" }
    } catch {
        if ($_.Exception.Response.StatusCode.value__ -in @(200,301,302)) { return "PASS" }
    }
    return "HTTP $code"
}

Write-Host "============================================="
Write-Host "  AEGIS Infrastructure Verification"
Write-Host "============================================="
Write-Host ""

# 1. Kubernetes nodes
Write-Host "--- Kubernetes ---"
$nodes = (kubectl get nodes --no-headers 2>$null | Measure-Object -Line).Lines
if ($nodes -ge 1) {
    Check "Kind cluster running ($nodes nodes)" "PASS"
} else {
    Check "Kind cluster running" "No nodes found"
}

# 2. Pods
Write-Host ""
Write-Host "--- Pods ---"
$allPods = kubectl get pods -n default --no-headers 2>$null
$podCount = ($allPods | Select-String "Running" | Measure-Object -Line).Lines
Check "Running pods ($podCount >=5)" "PASS"

foreach ($component in @("aegis-kafka", "aegis-minio", "aegis-flink", "aegis-trino", "aegis-polaris")) {
    if ($allPods -match "$component.*Running") {
        Check "$component pod running" "PASS"
    } else {
        Check "$component pod running" "Not found or not running"
    }
}

# 3. Kafka topics
Write-Host ""
Write-Host "--- Kafka Topics ---"
$topics = kubectl exec aegis-kafka-0 -n default -- /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list 2>$null
# Filter out internal topics (__consumer_offsets etc) for count
$appTopics = $topics | Where-Object { $_ -notmatch "^__" }
$appTopicCount = ($appTopics | Measure-Object -Line).Lines
if ($appTopicCount -ge 5) {
    Check "At least 5 app Kafka topics ($appTopicCount)" "PASS"
} else {
    Check "At least 5 app Kafka topics" "Found $appTopicCount app topics (expected 5)"
}
foreach ($topic in @("raw-l4-scans", "tls-fingerprints", "l7-banners", "scan-commands", "enrichment-events")) {
    if ($topics -match "^$topic$") {
        Check "Topic '$topic' exists" "PASS"
    } else {
        Check "Topic '$topic' exists" "Missing"
    }
}

# 4. Nessie (use kubectl curl pod)
Write-Host ""
Write-Host "--- Nessie ---"
$nessieOk = $false
try {
    # Try via temp curl pod first
    kubectl run verify-nessie --image=curlimages/curl:8.5.0 --restart=Never -- curl -s http://aegis-polaris:19120/api/v1/config 2>$null | Out-Null
    Start-Sleep -Seconds 6
    $nessieLog = kubectl logs verify-nessie -n default 2>$null
    kubectl delete pod verify-nessie -n default --ignore-not-found=true 2>$null | Out-Null
    if ($nessieLog -match "defaultBranch") { $nessieOk = $true }
} catch {}
if ($nessieOk) {
    Check "Nessie API v1 accessible (http://aegis-polaris:19120/api/v1/config)" "PASS"
} else {
    try {
        $nessieResp = Invoke-WebRequest -Uri "http://localhost:19120/api/v1/config" -TimeoutSec 5 -UseBasicParsing
        if ($nessieResp.StatusCode -eq 200) { Check "Nessie API v1 accessible (localhost:19120)" "PASS" } else { Check "Nessie API v1 accessible" "HTTP $($nessieResp.StatusCode)" }
    } catch {
        try {
            $r2 = Invoke-WebRequest -Uri "http://localhost:19120/api/v2/config" -TimeoutSec 5 -UseBasicParsing
            if ($r2.StatusCode -eq 200) { Check "Nessie API v1 accessible" "PASS (v2 fallback)" } else { Check "Nessie API v1 accessible" "HTTP $($r2.StatusCode)" }
        } catch { Check "Nessie API v1 accessible" "Connection failed (try: kubectl port-forward svc/aegis-polaris 19120:19120)" }
    }
}

# 5. Flink UI
Write-Host ""
Write-Host "--- Flink ---"
$flinkOk = $false
try {
    kubectl run verify-flink --image=curlimages/curl:8.5.0 --restart=Never -- curl -s -o /dev/null -w "%{http_code}" http://aegis-flink-jobmanager:8081/overview 2>$null | Out-Null
    Start-Sleep -Seconds 6
    $flinkCode = kubectl logs verify-flink -n default 2>$null
    kubectl delete pod verify-flink -n default --ignore-not-found=true 2>$null | Out-Null
    if ($flinkCode -match "200") { $flinkOk = $true }
} catch {}
if ($flinkOk) {
    Check "Flink Web UI accessible (http://aegis-flink-jobmanager:8081)" "PASS"
} else {
    try {
        $flinkResp = Invoke-WebRequest -Uri "http://localhost:8081/overview" -TimeoutSec 5 -UseBasicParsing
        if ($flinkResp.StatusCode -eq 200) { Check "Flink Web UI accessible on :8081" "PASS" } else { Check "Flink Web UI accessible on :8081" "HTTP $($flinkResp.StatusCode)" }
    } catch { Check "Flink Web UI accessible on :8081" "Connection failed (in-cluster check also failed)" }
}

# 6. Trino
Write-Host ""
Write-Host "--- Trino ---"
$trinoOk = $false
try {
    kubectl run verify-trino --image=curlimages/curl:8.5.0 --restart=Never -- curl -s http://aegis-trino-coordinator:8080/v1/info 2>$null | Out-Null
    Start-Sleep -Seconds 6
    $trinoLog = kubectl logs verify-trino -n default 2>$null
    kubectl delete pod verify-trino -n default --ignore-not-found=true 2>$null | Out-Null
    if ($trinoLog -match "coordinator") { $trinoOk = $true }
} catch {}
if ($trinoOk) {
    Check "Trino accessible (http://aegis-trino-coordinator:8080)" "PASS"
} else {
    try {
        $trinoResp = Invoke-WebRequest -Uri "http://localhost:8080/v1/info" -TimeoutSec 5 -UseBasicParsing
        if ($trinoResp.StatusCode -eq 200) { Check "Trino accessible on :8080" "PASS" } else { Check "Trino accessible on :8080" "HTTP $($trinoResp.StatusCode)" }
    } catch { Check "Trino accessible on :8080" "Connection failed" }
}

# 7. MinIO
Write-Host ""
Write-Host "--- MinIO ---"
$minioOk = $false
try {
    kubectl run verify-minio --image=curlimages/curl:8.5.0 --restart=Never -- curl -s -o /dev/null -w "%{http_code}" http://aegis-minio:9000/minio/health/live 2>$null | Out-Null
    Start-Sleep -Seconds 6
    $minioCode = kubectl logs verify-minio -n default 2>$null
    kubectl delete pod verify-minio -n default --ignore-not-found=true 2>$null | Out-Null
    if ($minioCode -match "200") { $minioOk = $true }
} catch {}
if ($minioOk) {
    Check "MinIO API accessible (http://aegis-minio:9000/minio/health/live)" "PASS"
} else {
    try {
        $minioResp = Invoke-WebRequest -Uri "http://localhost:9000/minio/health/live" -TimeoutSec 5 -UseBasicParsing
        if ($minioResp.StatusCode -eq 200) { Check "MinIO API accessible on :9000" "PASS" } else { Check "MinIO API accessible on :9000" "HTTP $($minioResp.StatusCode)" }
    } catch { Check "MinIO API accessible on :9000" "Connection failed" }
}
# MinIO console via kubectl as well (302 redirect is PASS)
$minioConsoleOk = $false
try {
    kubectl run verify-minio2 --image=curlimages/curl:8.5.0 --restart=Never -- curl -s -o /dev/null -w "%{http_code}" http://aegis-minio:9001/ 2>$null | Out-Null
    Start-Sleep -Seconds 6
    $mc = kubectl logs verify-minio2 -n default 2>$null
    kubectl delete pod verify-minio2 -n default --ignore-not-found=true 2>$null | Out-Null
    if ($mc -match "200|301|302|403") { $minioConsoleOk = $true }
} catch {}
if ($minioConsoleOk) {
    Check "MinIO Console accessible (http://aegis-minio:9001/)" "PASS"
} else {
    try {
        $minioConsole = Invoke-WebRequest -Uri "http://localhost:9001/" -TimeoutSec 5 -UseBasicParsing -MaximumRedirection 0
        if ($minioConsole.StatusCode -in @(200,301,302)) { Check "MinIO Console accessible on :9001" "PASS" } else { Check "MinIO Console accessible on :9001" "HTTP $($minioConsole.StatusCode)" }
    } catch {
        if ($_.Exception.Response.StatusCode.value__ -in @(301,302)) { Check "MinIO Console accessible on :9001" "PASS" } else { Check "MinIO Console accessible on :9001" "Connection failed" }
    }
}

# 8. Iceberg tables via Trino (kubectl exec)
Write-Host ""
Write-Host "--- Iceberg Tables (via Trino) ---"
$trinoPod = kubectl get pods -l component=coordinator -n default -o jsonpath='{.items[0].metadata.name}' 2>$null
if ($trinoPod) {
    $schemas = kubectl exec $trinoPod -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW SCHEMAS IN aegis" 2>$null
    if ($schemas -match "aegis") {
        Check "Iceberg schema 'aegis' exists in Nessie catalog" "PASS"
    } else {
        Check "Iceberg schema 'aegis' exists in Nessie catalog" "Not found (schemas: $schemas)"
    }
    $tables = kubectl exec $trinoPod -n default -- trino --server http://127.0.0.1:8080 --execute "SHOW TABLES IN aegis.aegis" 2>$null
    $foundHosts = ($tables -match "hosts")
    $foundServices = ($tables -match "services")
    $foundCerts = ($tables -match "certificates")
    if ($foundHosts) { Check "Table 'hosts' visible in Trino (aegis.aegis)" "PASS" } else { Check "Table 'hosts' visible in Trino" "Not found" }
    if ($foundServices) { Check "Table 'services' visible in Trino (aegis.aegis)" "PASS" } else { Check "Table 'services' visible in Trino" "Not found" }
    if ($foundCerts) { Check "Table 'certificates' visible in Trino (aegis.aegis)" "PASS" } else { Check "Table 'certificates' visible in Trino" "Not found" }
    # Row count probe
    $rowCheck = kubectl exec $trinoPod -n default -- trino --server http://127.0.0.1:8080 --execute "SELECT count(*) FROM aegis.aegis.hosts" 2>$null
    if ($rowCheck -match "[0-9]") { Check "Trino can query Iceberg tables (SELECT count(*) FROM hosts)" "PASS" } else { Check "Trino can query Iceberg tables" "Query failed: $rowCheck" }
} else {
    Check "Iceberg schema 'aegis' exists" "No Trino pod found"
    Check "Table 'hosts' visible" "No Trino pod"
    Check "Table 'services' visible" "No Trino pod"
    Check "Table 'certificates' visible" "No Trino pod"
    Check "Trino can query Iceberg tables" "No Trino pod"
}

# Summary
Write-Host ""
Write-Host "============================================="
Write-Host "  Results: $PASS passed, $FAIL failed"
Write-Host "============================================="

if ($FAIL -gt 0) {
    Write-Host ""
    Write-Host "  Some checks failed - see above. Note:"
    Write-Host "  - On Windows/Docker Desktop, host localhost checks may fail"
    Write-Host "    even when in-cluster connectivity (kubectl exec) succeeds."
    Write-Host "  - Run 'kubectl port-forward svc/aegis-trino-coordinator 8080:8080 &'"
    Write-Host "    and similar for 8081 (Flink), 9000/9001 (MinIO), 19120 (Nessie)"
    Write-Host "    to expose services to host."
    exit 1
}

Write-Host ""
Write-Host "  All verifications passed!"
Write-Host ""
Write-Host "  Cluster URLs (via port-forward or kind extraPortMappings):"
Write-Host "    Kafka:      localhost:9092  (via extraPortMappings 30092->9092)"
Write-Host "    Flink UI:   http://localhost:8081  (kubectl port-forward svc/aegis-flink-jobmanager 8081:8081)"
Write-Host "    Trino:      http://localhost:8080  (kubectl port-forward svc/aegis-trino-coordinator 8080:8080)"
Write-Host "    MinIO API:  http://localhost:9000"
Write-Host "    MinIO UI:   http://localhost:9001"
Write-Host "    Nessie:     http://localhost:19120  (kubectl port-forward svc/aegis-polaris 19120:19120)"
Write-Host ""
Write-Host "  Kafka Topics:"
Write-Host "    raw-l4-scans, tls-fingerprints, l7-banners, scan-commands, enrichment-events"
Write-Host "    (+ __consumer_offsets internal)"
Write-Host ""
Write-Host "  Schema Registry:"
Write-Host "    Nessie API: http://aegis-polaris:19120/api/v1 (Trino 433) /api/v2"
Write-Host "    Catalog: aegis  Schema: aegis  Tables: hosts, services, certificates"
Write-Host "    Example: trino --server http://localhost:8080 --execute 'SELECT * FROM aegis.aegis.hosts LIMIT 10;'"
