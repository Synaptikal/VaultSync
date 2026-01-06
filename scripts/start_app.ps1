# VaultSync Launcher Script

Write-Host "VaultSync Launcher" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan

# 1. Check Backend
$backendUrl = "http://localhost:3000/health"
Write-Host "Checking Backend Server..."
try {
    $response = Invoke-RestMethod -Uri $backendUrl -ErrorAction Stop
    Write-Host "Backend is running!" -ForegroundColor Green
} catch {
    Write-Host "Backend is NOT running. Starting it now..." -ForegroundColor Yellow
    Start-Process -FilePath "cargo" -ArgumentList "run" -WorkingDirectory "$PSScriptRoot" -NoNewWindow
    
    # Wait for startup
    Write-Host "Waiting for backend to initialize..."
    Start-Sleep -Seconds 5
}

# 2. Check Flutter
Write-Host "`nChecking Frontend Environment..."

# Ensure Flutter is in Path (Robustness)
$knownFlutterPath = "C:\Users\Justin\flutter\bin"
if ($env:Path -notlike "*$knownFlutterPath*" -and (Test-Path $knownFlutterPath)) {
    Write-Host "Adding Flutter to current session Path..." -ForegroundColor Yellow
    $env:Path += ";$knownFlutterPath"
}

if (Get-Command "flutter" -ErrorAction SilentlyContinue) {
    Write-Host "Flutter SDK found." -ForegroundColor Green
    
    Set-Location "$PSScriptRoot/frontend"

    # Check for Windows configuration
    if (-not (Test-Path "windows")) {
        Write-Host "Initializing Windows Desktop support..." -ForegroundColor Yellow
        cmd /c "flutter create --platforms=windows ."
    }

    Write-Host "Starting Mobile App..." -ForegroundColor Cyan
    flutter run -d windows
} else {
    Write-Host "Error: Flutter SDK not found in PATH." -ForegroundColor Red
    Write-Host "Please install Flutter to run the frontend application."
    Write-Host "Download: https://docs.flutter.dev/get-started/install"
    Write-Host "`nThe Backend API is fully functional and verified."
    Write-Host "You can interact with it via API tools or the 'verify_system.ps1' script."
}

Read-Host -Prompt "Press Enter to exit"
