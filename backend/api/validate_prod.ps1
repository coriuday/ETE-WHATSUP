$ErrorActionPreference = "Stop"

# Clear Database
Write-Host "Clearing database..."
docker exec -i whatsup_postgres psql -U postgres -d whatsup -c "TRUNCATE organizations CASCADE;"

$API_URL = "http://127.0.0.1:8081/api/v1"

Write-Host "Registering user..."
try {
    $reg = Invoke-RestMethod -Method Post -Uri "$API_URL/auth/register" -ContentType "application/json" -Body '{"email": "admin@whatsup.local", "password": "Password123!", "first_name": "Admin", "last_name": "User"}'
} catch {
    Write-Host "User might already exist, continuing..."
}

Write-Host "Logging in..."
$login = Invoke-RestMethod -Method Post -Uri "$API_URL/auth/login" -ContentType "application/json" -Body '{"email": "admin@whatsup.local", "password": "Password123!"}'
$token = $login.data.tokens.access_token
$headers = @{ "Authorization" = "Bearer $token"; "Content-Type" = "application/json" }

try {
    Write-Host "Creating Organization..."
    $org = Invoke-RestMethod -Method Post -Uri "$API_URL/organizations" -Headers $headers -Body '{"name": "Test Org", "website": "https://example.com"}'
    $orgId = $org.data.id
    Write-Host "Org ID: $orgId"

    Write-Host "Re-logging in to get updated token with org_id..."
    $login2 = Invoke-RestMethod -Method Post -Uri "$API_URL/auth/login" -ContentType "application/json" -Body '{"email": "admin@whatsup.local", "password": "Password123!"}'
    $token2 = $login2.data.tokens.access_token
    $headers = @{ "Authorization" = "Bearer $token2"; "Content-Type" = "application/json" }

    Write-Host "Creating WA Account..."
    $wa = Invoke-RestMethod -Method Post -Uri "$API_URL/whatsapp/accounts" -Headers $headers -Body '{"display_name": "Main WA", "phone_number": "1234567890", "phone_number_id": "12345", "waba_id": "67890", "access_token": "test_token"}'
    $waId = $wa.data.id
    Write-Host "WA Account ID: $waId"

    Write-Host "Creating Template..."
    $tplBody = @"
    {
        "wa_account_id": "$waId",
        "name": "hello_world",
        "display_name": "Hello World",
        "category": "marketing",
        "language": "en_US",
        "body_text": "Hello {{1}}! Welcome."
    }
"@
    $tpl = Invoke-RestMethod -Method Post -Uri "$API_URL/templates" -Headers $headers -Body $tplBody
    $tplId = $tpl.data.id
    Write-Host "Template ID: $tplId"

    Write-Host "Importing Contacts CSV..."
    $curlOut = curl.exe -s -X POST -F "file=@contacts_sample.csv" -H "Authorization: Bearer $token2" "$API_URL/contacts/import"
    Write-Host "Import response: $curlOut"
    $importObj = $curlOut | ConvertFrom-Json
    $jobId = $importObj.data.id
    Write-Host "Import Job ID: $jobId"

    Write-Host "Waiting for import to finish..."
    Start-Sleep -Seconds 3

    Write-Host "Creating Campaign..."
    $campBody = @"
    {
        "wa_account_id": "$waId",
        "name": "Prod Validation Campaign",
        "template_id": "$tplId",
        "type": "bulk_message",
        "target_type": "all_contacts"
    }
"@
    $camp = Invoke-RestMethod -Method Post -Uri "$API_URL/campaigns" -Headers $headers -Body $campBody
    $campId = $camp.data.id
    Write-Host "Campaign ID: $campId"

    Write-Host "Launching Campaign..."
    Invoke-RestMethod -Method Post -Uri "$API_URL/campaigns/$campId/launch" -Headers $headers -Body '{}'
} catch {
    Write-Host "Error: $($_.Exception.Message)"
    if ($_.ErrorDetails) {
        Write-Host "Details: $($_.ErrorDetails.Message)"
    } else {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $reader.BaseStream.Position = 0
        Write-Host "Response: $($reader.ReadToEnd())"
    }
    exit 1
}

Write-Host "Flow complete! Verification queries can now be run."
