# Database Rollback Script
# Usage: .\rollback_db.ps1 [-BackupFile "path\to\backup.db"]

param (
    [string]$BackupFile,
    [string]$DataDir = "$env:APPDATA\VaultSync\data",
    [string]$ContainerName = "vaultsync_backend"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $DataDir)) {
    Write-Error "Data directory $DataDir not found."
    exit 1
}

# 1. Select Backup
if (-not $BackupFile) {
    Write-Host "Available Backups:" -ForegroundColor Cyan
    $backups = Get-ChildItem -Path $DataDir -Filter "vaultsync_backup_*.db" | Sort-Object LastWriteTime -Descending
    
    if ($backups.Count -eq 0) {
        Write-Error "No backups found in $DataDir"
        exit 1
    }

    for ($i = 0; $i -lt $backups.Count; $i++) {
        Write-Host "[$i] $($backups[$i].Name) ($($backups[$i].LastWriteTime))"
    }

    $selection = Read-Host "Select backup to restore [0-$($backups.Count - 1)]"
    if ($selection -match "^\d+$" -and $selection -lt $backups.Count) {
        $BackupFile = $backups[$selection].FullName
    }
    else {
        Write-Error "Invalid selection."
        exit 1
    }
}

if (-not (Test-Path $BackupFile)) {
    Write-Error "Backup file $BackupFile not found."
    exit 1
}

Write-Host "Restoring from: $BackupFile" -ForegroundColor Yellow

# 2. Stop Container (if running)
$containerRunning = $false
if (Get-Command docker -ErrorAction SilentlyContinue) {
    if (docker ps -q -f name=$ContainerName) {
        Write-Host "Stopping container $ContainerName..."
        docker stop $ContainerName
        $containerRunning = $true
    }
}

# 3. Restore Database
$DbPath = Join-Path $DataDir "vaultsync.db"

# Backup current state just in case (Safety Net)
if (Test-Path $DbPath) {
    $SafetyBackup = Join-Path $DataDir "vaultsync_before_rollback_$(Get-Date -Format 'yyyyMMddHHmmss').db"
    Write-Host "Creating safety backup of current state to $SafetyBackup..."
    Copy-Item $DbPath $SafetyBackup
}

Write-Host "Overwriting vaultsync.db..."
Copy-Item $BackupFile $DbPath -Force

# 4. Restart Container
if ($containerRunning) {
    Write-Host "Restarting container..."
    docker start $ContainerName
}

Write-Host "Rollback Complete!" -ForegroundColor Green
