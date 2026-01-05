//! Backup Service
//!
//! Phase 11: Automated SQLite backup with verification and rotation
//!
//! Features:
//! - Safe SQLite database backup
//! - Backup verification
//! - Retention policy management
//! - Manual and scheduled backups

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Directory to store backups
    pub backup_dir: PathBuf,
    
    /// Path to the database file
    pub database_path: PathBuf,
    
    /// Number of days to retain backups
    pub retention_days: u32,
    
    /// Maximum number of backups to keep (regardless of age)
    pub max_backups: u32,
    
    /// Whether to create checksums for backups
    pub create_checksum: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("./backups"),
            database_path: PathBuf::from("./vaultsync.db"),
            retention_days: 30,
            max_backups: 50,
            create_checksum: true,
        }
    }
}

impl BackupConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            backup_dir: std::env::var("BACKUP_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./backups")),
            database_path: std::env::var("DATABASE_URL")
                .map(|s| PathBuf::from(s.replace("sqlite:", "")))
                .unwrap_or_else(|_| PathBuf::from("./vaultsync.db")),
            retention_days: std::env::var("BACKUP_RETENTION_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_backups: std::env::var("BACKUP_MAX_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            create_checksum: std::env::var("BACKUP_CHECKSUM")
                .map(|s| s.to_lowercase() != "false")
                .unwrap_or(true),
        }
    }
}

/// Information about a backup file
#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub checksum: Option<String>,
    pub verified: bool,
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize)]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: Option<String>,
    pub duration_ms: u64,
    pub message: String,
}

/// Backup service for managing database backups
pub struct BackupService {
    config: BackupConfig,
}

impl BackupService {
    pub fn new(config: BackupConfig) -> Self {
        Self { config }
    }

    /// Create a new backup service with default config from environment
    pub fn from_env() -> Self {
        Self::new(BackupConfig::from_env())
    }

    /// Ensure the backup directory exists
    fn ensure_backup_dir(&self) -> Result<()> {
        if !self.config.backup_dir.exists() {
            fs::create_dir_all(&self.config.backup_dir)
                .context("Failed to create backup directory")?;
        }
        Ok(())
    }

    /// Generate a backup filename with timestamp
    fn generate_backup_filename(&self) -> String {
        let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        format!("vaultsync_backup_{}.db", timestamp)
    }

    /// Calculate SHA256 checksum of a file
    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        let mut file = fs::File::open(path).context("Failed to open file for checksum")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// TASK-219: Create a backup of the database
    pub async fn create_backup(&self) -> Result<BackupResult> {
        let start = std::time::Instant::now();
        self.ensure_backup_dir()?;

        let backup_filename = self.generate_backup_filename();
        let backup_path = self.config.backup_dir.join(&backup_filename);

        // Check if source database exists
        if !self.config.database_path.exists() {
            return Ok(BackupResult {
                success: false,
                backup_path,
                size_bytes: 0,
                checksum: None,
                duration_ms: start.elapsed().as_millis() as u64,
                message: "Source database does not exist".to_string(),
            });
        }

        // Copy the database file
        // Note: For a more robust solution, use SQLite's backup API
        // This simple copy works when database is not being heavily written to
        fs::copy(&self.config.database_path, &backup_path)
            .context("Failed to copy database file")?;

        // Get backup size
        let size_bytes = fs::metadata(&backup_path)?.len();

        // Calculate checksum if enabled
        let checksum = if self.config.create_checksum {
            let checksum = self.calculate_checksum(&backup_path)?;
            
            // Write checksum to file
            let checksum_path = backup_path.with_extension("db.sha256");
            let mut checksum_file = fs::File::create(&checksum_path)?;
            writeln!(checksum_file, "{}  {}", checksum, backup_filename)?;
            
            Some(checksum)
        } else {
            None
        };

        tracing::info!(
            "Backup created: {} ({} bytes)",
            backup_filename,
            size_bytes
        );

        Ok(BackupResult {
            success: true,
            backup_path,
            size_bytes,
            checksum,
            duration_ms: start.elapsed().as_millis() as u64,
            message: format!("Backup created successfully: {}", backup_filename),
        })
    }

    /// TASK-222: Verify a backup file is valid
    pub async fn verify_backup(&self, backup_path: &Path) -> Result<bool> {
        // Check file exists
        if !backup_path.exists() {
            return Ok(false);
        }

        // Check checksum if exists
        let checksum_path = backup_path.with_extension("db.sha256");
        if checksum_path.exists() {
            let checksum_content = fs::read_to_string(&checksum_path)?;
            let expected_checksum: String = checksum_content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            let actual_checksum = self.calculate_checksum(backup_path)?;

            if expected_checksum != actual_checksum {
                tracing::warn!(
                    "Checksum mismatch for {:?}: expected {}, got {}",
                    backup_path,
                    expected_checksum,
                    actual_checksum
                );
                return Ok(false);
            }
        }

        // Try to open as SQLite database and run a simple query
        let db_url = format!("sqlite:{}?mode=ro", backup_path.display());
        match sqlx::SqlitePool::connect(&db_url).await {
            Ok(pool) => {
                // Try a simple query to verify database integrity
                let result = sqlx::query("SELECT COUNT(*) FROM sqlite_master")
                    .fetch_one(&pool)
                    .await;
                
                pool.close().await;
                
                match result {
                    Ok(_) => {
                        tracing::info!("Backup verified: {:?}", backup_path);
                        Ok(true)
                    }
                    Err(e) => {
                        tracing::warn!("Backup verification failed: {:?} - {}", backup_path, e);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to open backup database: {:?} - {}", backup_path, e);
                Ok(false)
            }
        }
    }

    /// TASK-224: Apply retention policy and delete old backups
    pub async fn apply_retention_policy(&self) -> Result<Vec<PathBuf>> {
        let mut deleted = Vec::new();
        
        // Get all backup files
        let mut backups = self.list_backups().await?;
        
        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let cutoff_date = Utc::now() - Duration::days(self.config.retention_days as i64);
        
        for (index, backup) in backups.iter().enumerate() {
            let should_delete = 
                // Delete if over max count
                index >= self.config.max_backups as usize ||
                // Delete if older than retention period (but keep minimum of 1)
                (backup.created_at < cutoff_date && index > 0);
            
            if should_delete {
                if let Err(e) = fs::remove_file(&backup.path) {
                    tracing::warn!("Failed to delete old backup {:?}: {}", backup.path, e);
                } else {
                    tracing::info!("Deleted old backup: {:?}", backup.path);
                    deleted.push(backup.path.clone());
                    
                    // Also delete checksum file if exists
                    let checksum_path = backup.path.with_extension("db.sha256");
                    let _ = fs::remove_file(checksum_path);
                }
            }
        }
        
        Ok(deleted)
    }

    /// List all available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        self.ensure_backup_dir()?;
        
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&self.config.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only include .db files that match our naming pattern
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                let filename = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if !filename.starts_with("vaultsync_backup_") {
                    continue;
                }
                
                let metadata = fs::metadata(&path)?;
                let created_at = metadata.created()
                    .or_else(|_| metadata.modified())
                    .map(|t| DateTime::<Utc>::from(t))
                    .unwrap_or_else(|_| Utc::now());
                
                // Check for checksum file
                let checksum_path = path.with_extension("db.sha256");
                let checksum = if checksum_path.exists() {
                    fs::read_to_string(&checksum_path)
                        .ok()
                        .and_then(|s| s.split_whitespace().next().map(String::from))
                } else {
                    None
                };
                
                backups.push(BackupInfo {
                    filename,
                    path,
                    size_bytes: metadata.len(),
                    created_at,
                    checksum,
                    verified: false, // Not verified until explicitly checked
                });
            }
        }
        
        // Sort by date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }

    /// TASK-226: Restore from a backup file
    pub async fn restore_backup(&self, backup_path: &Path) -> Result<()> {
        // Verify backup first
        if !self.verify_backup(backup_path).await? {
            anyhow::bail!("Backup verification failed - refusing to restore");
        }

        // Create a backup of current database before restore
        let pre_restore_backup = self.config.database_path.with_extension("db.pre-restore");
        if self.config.database_path.exists() {
            fs::copy(&self.config.database_path, &pre_restore_backup)
                .context("Failed to create pre-restore backup")?;
        }

        // Copy backup to database location
        fs::copy(backup_path, &self.config.database_path)
            .context("Failed to restore backup")?;

        tracing::info!(
            "Database restored from {:?}. Pre-restore backup saved to {:?}",
            backup_path,
            pre_restore_backup
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_filename_format() {
        let service = BackupService::new(BackupConfig::default());
        let filename = service.generate_backup_filename();
        
        assert!(filename.starts_with("vaultsync_backup_"));
        assert!(filename.ends_with(".db"));
    }

    #[test]
    fn test_default_config() {
        let config = BackupConfig::default();
        
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.max_backups, 50);
        assert!(config.create_checksum);
    }
}
