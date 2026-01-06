# VaultSync System Verification Script

$baseUrl = "http://localhost:3000"

function Test-Endpoint {
    param($Method, $Url, $Body, $Token)
    
    $headers = @{}
    if ($Token) {
        $headers["Authorization"] = "Bearer $Token"
    }
    
    try {
        if ($Body) {
            $jsonBody = $Body | ConvertTo-Json -Depth 5
            $response = Invoke-RestMethod -Method $Method -Uri "$baseUrl$Url" -Body $jsonBody -ContentType "application/json" -Headers $headers -ErrorAction Stop
        } else {
            $response = Invoke-RestMethod -Method $Method -Uri "$baseUrl$Url" -Headers $headers -ErrorAction Stop
        }
        return $response
    } catch {
        Write-Host "Error calling $Url : $($_.Exception.Message)" -ForegroundColor Red
        if ($_.Exception.Response) {
            $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
            Write-Host "Response: $($reader.ReadToEnd())" -ForegroundColor Red
        }
        return $null
    }
}

Write-Host "1. Checking Health..." -ForegroundColor Cyan
$health = Test-Endpoint -Method GET -Url "/health"
Write-Host "Status: $($health.status)" -ForegroundColor Green

Write-Host "`n2. Registering Admin User..." -ForegroundColor Cyan
$userUuid = [guid]::NewGuid().ToString()
$registerBody = @{
    username = "test_admin_$((Get-Date).Ticks)"
    password = "password123"
    role = "admin"
}
$register = Test-Endpoint -Method POST -Url "/auth/register" -Body $registerBody
Write-Host "User Registered" -ForegroundColor Green

Write-Host "`n3. Logging In..." -ForegroundColor Cyan
$loginBody = @{
    username = $registerBody.username
    password = $registerBody.password
}
$login = Test-Endpoint -Method POST -Url "/auth/login" -Body $loginBody
$token = $login.token
Write-Host "Token Received: $token" -ForegroundColor Green

Write-Host "`n4. Creating Product..." -ForegroundColor Cyan
$productBody = @{
    product_uuid = [guid]::NewGuid().ToString()
    name = "Black Lotus"
    category = "TCG"
    set_code = "Alpha"
    metadata = @{ rarity = "Rare" }
}
$product = Test-Endpoint -Method POST -Url "/api/products" -Body $productBody -Token $token
Write-Host "Product Created: $($product.name)" -ForegroundColor Green

Write-Host "`n5. Adding Inventory..." -ForegroundColor Cyan
$inventoryBody = @{
    inventory_uuid = [guid]::NewGuid().ToString()
    product_uuid = $productBody.product_uuid
    condition = "NM"
    quantity_on_hand = 5
    location_tag = "Safe"
}
$inventory = Test-Endpoint -Method POST -Url "/api/inventory" -Body $inventoryBody -Token $token
Write-Host "Inventory Added" -ForegroundColor Green

Write-Host "`n6. Creating Customer..." -ForegroundColor Cyan
$customerBody = @{
    customer_uuid = [guid]::NewGuid().ToString()
    name = "John Doe"
    email = "john@example.com"
    store_credit = 0.0
    created_at = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ssZ")
}
$customer = Test-Endpoint -Method POST -Url "/api/customers" -Body $customerBody -Token $token
Write-Host "Customer Created: $($customer.name)" -ForegroundColor Green

Write-Host "`n7. Processing Sale Transaction..." -ForegroundColor Cyan
$transactionBody = @{
    customer_uuid = $customerBody.customer_uuid
    items = @(
        @{
            item_uuid = [guid]::NewGuid().ToString()
            product_uuid = $productBody.product_uuid
            quantity = 1
            unit_price = 10000.0
            condition = "NM"
        }
    )
}
$transaction = Test-Endpoint -Method POST -Url "/api/transactions" -Body $transactionBody -Token $token
Write-Host "Transaction Processed: Type=$($transaction.transaction_type)" -ForegroundColor Green

Write-Host "`nVerification Complete!" -ForegroundColor Green
