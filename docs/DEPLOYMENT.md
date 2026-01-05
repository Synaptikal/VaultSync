# VaultSync Deployment Guide

This guide details the process for deploying the VaultSync backend and frontend for production use.

## Prerequisites

- **Host Machine:** Windows 10/11 Professional (Preferred for POS drivers) or Linux (Debian 12+).
- **Runtime:** Docker Engine & Docker Compose OR Native Rust/Flutter toolchains.
- **Hardware:** Min 8GB RAM, SSD Storage.

---

## Option 1: Docker Deployment (Recommended for Server/Master Node)

This method isolates the backend services and ensures environment consistency.

### 1. Build the Image
```bash
docker build -t vaultsync-backend:latest .
```

### 2. Configure Environment
Create a `.env` file in the deployment directory (use `.env.example` as a template):
```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite://vaultsync.db
JWT_SECRET=<generate_secure_random_string>
ENVIRONMENT=production
NODE_ID=node_master_01
BACKUP_ENABLED=true
BACKUP_INTERVAL_HOURS=6
```

### 3. Run with Docker Compose
Create a `docker-compose.yml`:
```yaml
version: '3.8'
services:
  vaultsync:
    image: vaultsync-backend:latest
    restart: unless-stopped
    ports:
      - "3000:3000"     # API
      - "5353:5353/udp" # mDNS Discovery
    volumes:
      - ./data:/app/data         # Persist DB
      - ./backups:/app/backups   # Persist Backups
      - ./logs:/app/logs
    env_file:
      - .env
```

Start the service:
```bash
docker-compose up -d
```

---

## Option 2: Native Windows Deployment (Recommended for POS Terminals)

This is required if you need direct access to USB printers/scanners that don't pass through Docker easily.

### 1. Build Release Binary
On a Windows machine with Rust installed:
```powershell
cargo build --release
```
The binary will be at `target/release/vaultsync.exe`.

### 2. Prepare Directory Structure
Create a folder `C:\VaultSync\` and populate it:
```text
C:\VaultSync\
  ├── vaultsync.exe
  ├── .env
  ├── logs\
  └── backups\
```

### 3. Install as a Windows Service (Optional)
Use a tool like `NSSM` (Non-Sucking Service Manager) to run `vaultsync.exe` as a background service.
```powershell
nssm install VaultSync "C:\VaultSync\vaultsync.exe"
nssm set VaultSync AppDirectory "C:\VaultSync"
nssm start VaultSync
```

---

## Frontend Deployment (Flutter)

### 1. Build for Windows
```powershell
cd frontend
flutter build windows --release
```
The executable is located at `build/windows/runner/Release/vaultsync.exe`.

### 2. Build for Android/iOS
Connect device and run:
```bash
flutter build apk --release
# or
flutter build ios --release
```

### 3. Distribution
- **Windows:** Zip the `Release` folder and distribute to terminals.
- **Mobile:** Use TestFlight (iOS) or internal APK distribution (Android).

---

## Post-Deployment Verification

1. **Check Health:** Visit `http://localhost:3000/health`. Should return `{"status":"ok"}`.
2. **Verify Logs:** check `logs/` to ensure no startup errors.
3. **Test Sync:** Start a second node and verify they discover each other via mDNS logs.