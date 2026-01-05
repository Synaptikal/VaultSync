# VaultSync Database Migration Guide

## Overview

VaultSync uses an embedded migration system. Database schema updates are applied automatically when the application starts up. This ensures that the application code and database schema are always in sync.

## Migration Process

1.  **Preparation**:
    *   Ensure a backup of the current database is taken (handled automatically by `deploy_update.ps1`).
    *   Check `AUDIT_REPORT.md` for any breaking changes warnings.

2.  **Deployment**:
    *   Deploy the new Container or Binary.
    *   Start the service.
    *   The application will inspect the `_migrations` table.
    *   Any pending migrations (defined in `src/database/mod.rs`) will be applied sequentially.

3.  **Verification**:
    *   Check the application logs (`docker logs vaultsync_backend`).
    *   Look for "Applying migration X: Description".
    *   Verify application health.

## Rollback Procedure

If a migration fails or the application is unstable after update:

1.  **Do NOT** attempt to manually revert schema changes unless you are an expert.
2.  **Use the Rollback Script**:
    ```powershell
    .\scripts\rollback_db.ps1
    ```
    *   This script lists available backups.
    *   Select the backup taken immediately before the deployment.
    *   The script will stop the container, restore the database file, and restart the container.

3.  **Revert Application Version**:
    *   You must also roll back the application binary/image to the previous version that matches the restored database schema.
    ```powershell
    # Example to revert to specific tag
    .\scripts\deploy_update.ps1 -ImageName "vaultsync:v1.2.3"
    ```

## Development

*   **Adding Migrations**: Add a new tuple to `get_schema_migrations()` in `src/database/mod.rs`. Increment the version number.
*   **Testing**: Run `cargo test` to verify migrations apply cleanly in a test environment.
*   **Preview**: Use the `preview_migrations()` method (or test) to see what will be applied.

## Troubleshooting

*   **Migration Lock**: If the app crashes *during* migration, the DB might be in an inconsistent state. Restore from backup immediately.
*   **Checksum Mismatch**: Currently not enforced, but ensure migration history is immutable.
