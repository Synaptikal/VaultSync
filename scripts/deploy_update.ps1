# Deploy/Update Script for VaultSync Backend
# Usage: .\deploy_update.ps1 -ImageName "ghcr.io/youruser/vaultsync:latest"

param (
    [string]$ImageName = "vaultsync:latest",
    [string]$ContainerName = "vaultsync_backend",
    [string]$DataDir = "$env:APPDATA\VaultSync\data"
)

$ErrorActionPreference = "Stop"

Write-Host ">>> VaultSync Deployment Helper" -ForegroundColor Cyan

# 1. Check Docker
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "Docker is not installed or not in PATH."
    exit 1
}

# 2. Pull Image
Write-Host "Pulling latest image: $ImageName..."
docker pull $ImageName

# 3. Stop existing container
if (docker ps -q -f name=$ContainerName) {
    Write-Host "Stopping existing container..."
    docker stop $ContainerName
    docker rm $ContainerName
} elseif (docker ps -a -q -f name=$ContainerName) {
    Write-Host "Removing stopped container..."
    docker rm $ContainerName
}

# 4. Backup Database
if (-not (Test-Path $DataDir)) {
    New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
}

$DbPath = Join-Path $DataDir "vaultsync.db"
if (Test-Path $DbPath) {
    $BackupPath = Join-Path $DataDir "vaultsync_backup_$(Get-Date -Format 'yyyyMMddHHmmss').db"
    Write-Host "Backing up database to $BackupPath..."
    Copy-Item $DbPath $BackupPath
}

# 5. Start New Container
Write-Host "Starting new container..."
docker run -d `
    --name $ContainerName `
    --restart unless-stopped `
    -p 3000:3000 `
    -v "${DataDir}:/app/data" `
    -e RUST_LOG=info `
    $ImageName

Write-Host "Deployment Complete!" -ForegroundColor Green
docker ps -f name=$ContainerName
