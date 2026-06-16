$ErrorActionPreference = "Continue"

Write-Host "Waiting for API to start..."
$apiUp = $false
while (-not $apiUp) {
    try {
        $response = Invoke-WebRequest -Uri "http://127.0.0.1:8081/api/v1/health" -Method Get -ErrorAction Stop
        if ($response.StatusCode -eq 200) {      Write-Host "API is up!"
            $apiUp = $true
        }
    } catch {
        Start-Sleep -Seconds 5
        Write-Host "Still waiting..."
    }
}

& .\test_flow.ps1

Start-Sleep -Seconds 10 # Give workers time to process jobs

Write-Host "`n=== message_queue_jobs ==="
docker exec -i whatsup_postgres psql -U postgres -d whatsup -c "SELECT id, campaign_id, wa_account_id, contact_id, status FROM message_queue_jobs;"

Write-Host "`n=== messages ==="
docker exec -i whatsup_postgres psql -U postgres -d whatsup -c "SELECT id, body, status, direction FROM messages;"

Write-Host "`n=== campaign_activity_logs ==="
docker exec -i whatsup_postgres psql -U postgres -d whatsup -c "SELECT id, action, details FROM campaign_activity_logs;"
