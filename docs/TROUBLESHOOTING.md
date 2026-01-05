# VaultSync Troubleshooting Guide

## Common Issues & Solutions

### 1. Sync Failures
**Symptom:** Terminal is not receiving updates from other nodes.
**Diagnosis:**
1. Check network connectivity.
2. Verify all nodes are on the same subnet.
3. Check `/health/detailed` endpoint for sync status.
**Resolution:**
- Restart the application service.
- If persistent, verify firewall settings allow TCP port 3000-3005 and UDP 5353 (mDNS).
- Check logs for "Partition detected" messages.

### 2. Database Locked
**Symptom:** "Database is locked" error in logs or UI.
**Resolution:**
- Ideally, the application handles retries.
- If stuck, ensure no other process (like a SQLite browser) has the file open.
- Restart the backend service to release file handles.

### 3. "Double Sale" Conflicts
**Symptom:** Two customers bought the same item on different offline terminals.
**Resolution:**
- The system defaults to Last-Write-Wins (LWW).
- Check the Audit Log (`/api/admin/audit-log`) to identify the conflicting transaction.
- Manager must explicitly refund one customer and explain the error.

### 4. Barcode Scanner Not Working
**Diagnosis:**
- Ensure scanner is in "HID Keyboard Mode".
- Focus the search input field before scanning.
**Resolution:**
- Scan the manufacturer's "Factory Reset" barcode.
- Re-configure for "Enter Key Suffix" (Carriage Return) after scan.

## Incident Response

### Data Corruption
1. **Stop** all terminals immediately.
2. Locate the latest valid backup in `backups/`.
3. Use the restore command: `./vaultsync restore <backup_filename>`.
4. Verify data integrity before allowing terminals to reconnect.

### Security Breach
1. Rotate all JWT secrets in `.env`.
2. Force logout all users (restart service/invalidate tokens).
3. Review Audit Logs for suspicious activity.
