# VaultSync Backup & Recovery Guide

This guide covers backup procedures, restore operations, and disaster recovery for VaultSync.

---

## Quick Reference

| Action | API Endpoint | CLI (if available) |
|--------|--------------|---------------------|
| Create Backup | `POST /api/admin/backup` | - |
| List Backups | `GET /api/admin/backups` | - |
| Verify Backup | `POST /api/admin/backup/verify` | - |
| Apply Retention | `POST /api/admin/backup/retention` | - |

---

## 1. Backup Configuration

### Environment Variables

```env
# Backup Directory (where backups are stored)
BACKUP_DIR=./backups

# Database Path (source database)
DATABASE_URL=sqlite:./vaultsync.db

# Retention Settings
BACKUP_RETENTION_DAYS=30    # Keep backups for 30 days
BACKUP_MAX_COUNT=50         # Maximum number of backups to keep

# Checksum Verification
BACKUP_CHECKSUM=true        # Create SHA256 checksums
```

### Default Locations

| Item | Default Path |
|------|--------------|
| Backups | `./backups/` |
| Database | `./vaultsync.db` |
| Checksum Files | `./backups/*.sha256` |

---

## 2. Creating Backups

### Manual Backup (via API)

```bash
# Create a new backup (requires manager authentication)
curl -X POST http://localhost:3000/api/admin/backup \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Response:**
```json
{
  "success": true,
  "backup_path": "./backups/vaultsync_backup_2026-01-03_14-30-00.db",
  "size_bytes": 1048576,
  "checksum": "abc123...",
  "duration_ms": 150,
  "message": "Backup created successfully"
}
```

### Automated Backups

For production, set up a scheduled task (cron job or Windows Task Scheduler):

**Linux (crontab):**
```bash
# Daily backup at 2 AM
0 2 * * * curl -X POST http://localhost:3000/api/admin/backup -H "Authorization: Bearer $BACKUP_TOKEN"
```

**Windows (Task Scheduler):**
1. Create a new basic task
2. Set trigger to "Daily" at 2:00 AM
3. Action: Start a program → PowerShell
4. Arguments: `-Command "Invoke-WebRequest -Method POST -Uri 'http://localhost:3000/api/admin/backup' -Headers @{Authorization='Bearer TOKEN'}"`

---

## 3. Listing Backups

```bash
curl http://localhost:3000/api/admin/backups \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Response:**
```json
{
  "count": 3,
  "backups": [
    {
      "filename": "vaultsync_backup_2026-01-03_14-30-00.db",
      "path": "./backups/vaultsync_backup_2026-01-03_14-30-00.db",
      "size_bytes": 1048576,
      "created_at": "2026-01-03T14:30:00Z",
      "checksum": "abc123...",
      "verified": false
    }
  ]
}
```

---

## 4. Verifying Backups

Always verify a backup before attempting to restore from it:

```bash
curl -X POST http://localhost:3000/api/admin/backup/verify \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"backup_path": "./backups/vaultsync_backup_2026-01-03_14-30-00.db"}'
```

**Response:**
```json
{
  "valid": true,
  "path": "./backups/vaultsync_backup_2026-01-03_14-30-00.db"
}
```

---

## 5. Restore Procedures

### Step 1: Stop the VaultSync Server
```bash
# Stop the running server
# (method depends on how you're running VaultSync)
```

### Step 2: Verify the Backup
Ensure the backup file is valid before restoring.

### Step 3: Backup Current Database
```bash
# Create a safety backup of current (possibly corrupted) database
cp ./vaultsync.db ./vaultsync.db.pre-restore-$(date +%Y%m%d_%H%M%S)
```

### Step 4: Restore the Backup
```bash
# Copy the backup file to the database location
cp ./backups/vaultsync_backup_2026-01-03_14-30-00.db ./vaultsync.db
```

### Step 5: Restart VaultSync
```bash
# Start the server (method depends on deployment)
./vaultsync  # or your startup command
```

### Step 6: Verify Restoration
- Check that the application starts without errors
- Verify recent transactions are present
- Check inventory counts

---

## 6. Retention Policy

Apply the retention policy to clean up old backups:

```bash
curl -X POST http://localhost:3000/api/admin/backup/retention \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Response:**
```json
{
  "deleted_count": 5,
  "deleted_files": ["./backups/old_backup_1.db", "..."]
}
```

---

## 7. Disaster Recovery Runbook

### Scenario 1: Database Corruption

**Symptoms:**
- Application crashes on startup
- SQLite errors in logs
- Data inconsistencies

**Recovery Steps:**
1. Stop VaultSync
2. Identify the most recent valid backup using the list/verify APIs
3. Restore from the backup (see Section 5)
4. Restart VaultSync
5. Report any data loss to users

### Scenario 2: Hardware Failure

**Symptoms:**
- Server/disk completely inaccessible

**Recovery Steps (on new hardware):**
1. Install VaultSync on new system
2. Copy backup files from offsite storage (if available)
3. Verify the most recent backup
4. Restore the database
5. Update any necessary configuration (paths, hostnames)
6. Start VaultSync
7. Re-sync with other terminals if applicable

### Scenario 3: Ransomware/Security Incident

**Symptoms:**
- Files encrypted or inaccessible
- Suspicious activity in logs

**Recovery Steps:**
1. Disconnect affected system from network IMMEDIATELY
2. Do NOT pay ransom
3. Report to appropriate authorities
4. Obtain clean backups from offsite/offline storage
5. Wipe and reinstall the operating system
6. Reinstall VaultSync
7. Restore from verified backup
8. Change ALL credentials (database, admin users, API keys)

### Scenario 4: Accidental Data Deletion

**Symptoms:**
- Missing transactions, inventory, or customers

**Recovery Steps:**
1. Stop any further operations immediately
2. Determine when the deletion occurred
3. Find a backup from before the deletion
4. You have two options:
   - Full restore (will lose changes after backup)
   - Partial recovery (export specific tables from backup, merge manually)
5. For partial recovery, contact technical support

---

## 8. Best Practices

### Backup Schedule
- **Minimum:** Daily backups
- **Recommended:** Every 4 hours during business hours
- **Critical:** Before any major updates or changes

### Offsite Backups
- Copy backups to an external drive daily
- Consider cloud backup (AWS S3, Google Drive, Dropbox)
- Store at least one offsite copy in case of physical disaster

### Backup Testing
- **Weekly:** Verify at least one backup using the verify endpoint
- **Monthly:** Perform a test restore to a separate location
- **Quarterly:** Full disaster recovery drill

### Security
- Encrypt backup files if they contain sensitive data
- Restrict access to backup files (file permissions)
- Protect API tokens used for automated backups
- Keep offsite backups in a secure location

---

## 9. Troubleshooting

### Backup Fails with "Source database does not exist"
- Check the `DATABASE_URL` environment variable
- Verify the database file path is correct

### Backup Verification Fails
- The backup file may be corrupted
- Check disk space on the backup drive
- Try creating a new backup

### Large Backup Files
- VaultSync databases can grow over time
- Consider archiving old transactions
- Run SQLite VACUUM periodically: `sqlite3 vaultsync.db "VACUUM;"`

### Backup Takes Too Long
- Large databases may take time to copy
- Schedule backups during low-activity periods
- Consider incremental backup solutions for very large databases

---

## Support

For additional assistance:
1. Check the VaultSync logs for detailed error messages
2. Review the API documentation at `/swagger-ui`
3. Contact technical support with backup/restore logs
