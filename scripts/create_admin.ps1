$baseUrl = "http://localhost:3000"

Write-Host "Attempting to create 'admin' user..." -ForegroundColor Cyan

try {
    $registerBody = @{
        username = "admin"
        password = "password123"
        role = "admin"
    }
    $jsonBody = $registerBody | ConvertTo-Json
    $response = Invoke-RestMethod -Method POST -Uri "$baseUrl/auth/register" -Body $jsonBody -ContentType "application/json" -ErrorAction Stop
    Write-Host "Admin user created successfully!" -ForegroundColor Green
    Write-Host "Username: admin" -ForegroundColor Green
    Write-Host "Password: password123" -ForegroundColor Green
} catch {
    if ($_.Exception.Response.StatusCode -eq "Conflict") {
        Write-Host "User 'admin' already exists." -ForegroundColor Yellow
        
        # If it exists, maybe the password is different? We can't reset it easily via API without a reset endpoint.
        # But for dev, we can try to login to verify if 'password123' works.
        
        Write-Host "Verifying login with 'password123'..."
        try {
            $loginBody = @{
                username = "admin"
                password = "password123"
            }
            $jsonLogin = $loginBody | ConvertTo-Json
            $loginResponse = Invoke-RestMethod -Method POST -Uri "$baseUrl/auth/login" -Body $jsonLogin -ContentType "application/json" -ErrorAction Stop
            Write-Host "Login successful! Credentials are correct." -ForegroundColor Green
        } catch {
            Write-Host "Login failed. The user 'admin' exists but the password is not 'password123'." -ForegroundColor Red
            Write-Host "You may need to restart the backend with a fresh database or use a different username." -ForegroundColor Red
        }
    } else {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        if ($_.Exception.Response) {
             $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
             Write-Host "Response: $($reader.ReadToEnd())" -ForegroundColor Red
        }
    }
}
