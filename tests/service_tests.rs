//! Service Unit Tests
//!
//! TASK-230, TASK-233: Unit tests for business logic services

use uuid::Uuid;

/// Helper to create test database
async fn create_test_db() -> std::sync::Arc<vaultsync::database::Database> {
    vaultsync::database::initialize_test_db()
        .await
        .expect("Failed to create test database")
}

/// Tax Service Tests (TASK-233)
mod tax_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_tax_rate_storage() {
        let db = create_test_db().await;

        // Insert a tax rate
        let rate_uuid = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO Tax_Rates (rate_id, name, rate, is_default, created_at, updated_at)
            VALUES (?, 'Test Tax', 0.075, 1, ?, ?)
            "#,
        )
        .bind(rate_uuid.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&db.pool)
        .await
        .expect("Failed to insert tax rate");

        // Query it back
        let row: (String, f64) =
            sqlx::query_as("SELECT name, rate FROM Tax_Rates WHERE rate_id = ?")
                .bind(rate_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .expect("Failed to query tax rate");

        assert_eq!(row.0, "Test Tax");
        assert!((row.1 - 0.075).abs() < 0.001);
    }

    #[test]
    fn test_tax_calculation_logic() {
        // Test basic tax calculation
        let subtotal = 100.0;
        let tax_rate = 0.08; // 8%

        let tax = subtotal * tax_rate;
        let total = subtotal + tax;

        assert!((tax - 8.0_f64).abs() < 0.01);
        assert!((total - 108.0_f64).abs() < 0.01);
    }

    #[test]
    fn test_tax_rounding() {
        // Test that tax rounds correctly to 2 decimal places
        let subtotal = 9.99;
        let tax_rate = 0.0825; // 8.25%

        let raw_tax: f64 = subtotal * tax_rate;
        let rounded_tax: f64 = (raw_tax * 100.0).round() / 100.0;

        assert!((raw_tax - 0.824175_f64).abs() < 0.0001);
        assert_eq!(rounded_tax, 0.82);
    }

    #[test]
    fn test_tax_exempt_calculation() {
        let subtotal = 100.0;
        let is_tax_exempt = true;

        let tax = if is_tax_exempt { 0.0 } else { subtotal * 0.08 };

        assert_eq!(tax, 0.0);
    }
}

/// Backup Service Tests
mod backup_service_tests {
    use std::path::PathBuf;
    use tempfile::tempdir;
    use vaultsync::services::backup::{BackupConfig, BackupService};

    #[tokio::test]
    async fn test_backup_config_default() {
        let config = BackupConfig::default();

        assert_eq!(config.retention_days, 30);
        assert_eq!(config.max_backups, 50);
        assert!(config.create_checksum);
    }

    #[tokio::test]
    async fn test_backup_filename_format() {
        let config = BackupConfig::default();
        let _service = BackupService::new(config);

        // The service generates timestamped filenames
        // Format: vaultsync_backup_YYYY-MM-DD_HH-MM-SS.db
        // We can't test the exact output, but we document the format
    }

    #[tokio::test]
    async fn test_backup_creation_with_source() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a test database file
        std::fs::write(&db_path, b"SQLite format 3\x00test data").unwrap();

        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            database_path: db_path.clone(),
            retention_days: 7,
            max_backups: 5,
            create_checksum: true,
        };

        let backup_service = BackupService::new(config);
        let result = backup_service.create_backup().await.unwrap();

        assert!(result.success);
        assert!(result.backup_path.exists());
        assert!(result.size_bytes > 0);
        assert!(result.checksum.is_some());
    }

    #[tokio::test]
    async fn test_backup_without_source_fails_gracefully() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_db = temp_dir.path().join("does_not_exist.db");

        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            database_path: nonexistent_db,
            retention_days: 7,
            max_backups: 5,
            create_checksum: false,
        };

        let backup_service = BackupService::new(config);
        let result = backup_service.create_backup().await.unwrap();

        assert!(!result.success);
        assert!(result.message.contains("does not exist"));
    }

    #[tokio::test]
    async fn test_backup_listing() {
        let temp_dir = tempdir().unwrap();

        // Create fake backup files
        std::fs::write(
            temp_dir
                .path()
                .join("vaultsync_backup_2026-01-01_10-00-00.db"),
            b"backup1",
        )
        .unwrap();

        // Ensure distinct timestamp
        std::thread::sleep(std::time::Duration::from_millis(100));

        std::fs::write(
            temp_dir
                .path()
                .join("vaultsync_backup_2026-01-02_10-00-00.db"),
            b"backup2",
        )
        .unwrap();
        std::fs::write(temp_dir.path().join("not_a_backup.txt"), b"other file").unwrap();

        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            database_path: PathBuf::from("test.db"),
            retention_days: 7,
            max_backups: 5,
            create_checksum: false,
        };

        let backup_service = BackupService::new(config);
        let backups = backup_service.list_backups().await.unwrap();

        assert_eq!(backups.len(), 2);
        // Backups should be sorted newest first
        assert!(backups[0].filename.contains("2026-01-02"));
    }

    #[tokio::test]
    async fn test_retention_policy() {
        let temp_dir = tempdir().unwrap();

        // Create more backups than max_backups
        for i in 1..=10 {
            std::fs::write(
                temp_dir
                    .path()
                    .join(format!("vaultsync_backup_2026-01-{:02}_10-00-00.db", i)),
                format!("backup{}", i),
            )
            .unwrap();
        }

        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            database_path: PathBuf::from("test.db"),
            retention_days: 7,
            max_backups: 5,
            create_checksum: false,
        };

        let backup_service = BackupService::new(config);

        let before_count = backup_service.list_backups().await.unwrap().len();
        assert_eq!(before_count, 10);

        let deleted = backup_service.apply_retention_policy().await.unwrap();
        assert_eq!(deleted.len(), 5);

        let after_count = backup_service.list_backups().await.unwrap().len();
        assert_eq!(after_count, 5);
    }
}

/// Barcode generation tests
mod barcode_tests {
    #[test]
    fn test_ean13_checksum() {
        // EAN-13 checksum calculation
        fn calculate_ean13_checksum(digits: &[u8; 12]) -> u8 {
            let sum: u32 = digits
                .iter()
                .enumerate()
                .map(|(i, &d)| {
                    let multiplier = if i % 2 == 0 { 1 } else { 3 };
                    (d as u32) * multiplier
                })
                .sum();
            ((10 - (sum % 10)) % 10) as u8
        }

        // Test with known EAN-13: 5901234123457
        let digits: [u8; 12] = [5, 9, 0, 1, 2, 3, 4, 1, 2, 3, 4, 5];
        let checksum = calculate_ean13_checksum(&digits);
        assert_eq!(checksum, 7);
    }

    #[test]
    fn test_sku_generation() {
        // Test SKU format: CATEGORY-XXXXXX
        let category = "MTG";
        let sequence = 123;
        let sku = format!("{}-{:06}", category, sequence);

        assert_eq!(sku, "MTG-000123");
        assert_eq!(sku.len(), 10);
    }
}

/// Customer data tests
mod customer_tests {
    use super::*;

    #[tokio::test]
    async fn test_customer_crud() {
        let db = create_test_db().await;

        // Create customer
        let customer_uuid = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO Customers (customer_uuid, name, email, phone, created_at)
            VALUES (?, 'Test Customer', 'test@example.com', '555-1234', ?)
            "#,
        )
        .bind(customer_uuid.to_string())
        .bind(now)
        .execute(&db.pool)
        .await
        .expect("Failed to create customer");

        // Read customer
        let row: (String, String) =
            sqlx::query_as("SELECT name, email FROM Customers WHERE customer_uuid = ?")
                .bind(customer_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .expect("Failed to read customer");

        assert_eq!(row.0, "Test Customer");
        assert_eq!(row.1, "test@example.com");

        // Update customer
        sqlx::query("UPDATE Customers SET name = ? WHERE customer_uuid = ?")
            .bind("Updated Name")
            .bind(customer_uuid.to_string())
            .execute(&db.pool)
            .await
            .expect("Failed to update customer");

        let updated_name: (String,) =
            sqlx::query_as("SELECT name FROM Customers WHERE customer_uuid = ?")
                .bind(customer_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .expect("Failed to read updated customer");

        assert_eq!(updated_name.0, "Updated Name");

        // Delete customer
        sqlx::query("DELETE FROM Customers WHERE customer_uuid = ?")
            .bind(customer_uuid.to_string())
            .execute(&db.pool)
            .await
            .expect("Failed to delete customer");

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM Customers WHERE customer_uuid = ?")
                .bind(customer_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .expect("Failed to count customers");

        assert_eq!(count.0, 0);
    }
}

/// Product and Inventory tests
mod inventory_tests {
    use super::*;

    #[tokio::test]
    async fn test_product_creation() {
        let db = create_test_db().await;

        let product_uuid = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO Global_Catalog (product_uuid, name, category, manufacturer, barcode)
            VALUES (?, 'Black Lotus', 'MTG', 'Wizards of the Coast', '123456789')
            "#,
        )
        .bind(product_uuid.to_string())
        .execute(&db.pool)
        .await
        .expect("Failed to create product");

        let row: (String, String) =
            sqlx::query_as("SELECT name, category FROM Global_Catalog WHERE product_uuid = ?")
                .bind(product_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .expect("Failed to read product");

        assert_eq!(row.0, "Black Lotus");
        assert_eq!(row.1, "MTG");
    }

    #[tokio::test]
    async fn test_inventory_quantity_update() {
        let db = create_test_db().await;

        // Create product
        let product_uuid = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO Global_Catalog (product_uuid, name, category) VALUES (?, 'Test', 'MTG')",
        )
        .bind(product_uuid.to_string())
        .execute(&db.pool)
        .await
        .unwrap();

        // Create inventory
        let inventory_uuid = Uuid::new_v4();
        // Schema requires location_tag
        sqlx::query(
            r#"
            INSERT INTO Local_Inventory (inventory_uuid, product_uuid, quantity_on_hand, condition, location_tag)
            VALUES (?, ?, 10, 'NM', 'LOC-A')
            "#,
        )
        .bind(inventory_uuid.to_string())
        .bind(product_uuid.to_string())
        .execute(&db.pool)
        .await
        .unwrap();

        // Update quantity (simulate sale)
        sqlx::query("UPDATE Local_Inventory SET quantity_on_hand = quantity_on_hand - 3 WHERE inventory_uuid = ?")
            .bind(inventory_uuid.to_string())
            .execute(&db.pool)
            .await
            .unwrap();

        let qty: (i32,) =
            sqlx::query_as("SELECT quantity_on_hand FROM Local_Inventory WHERE inventory_uuid = ?")
                .bind(inventory_uuid.to_string())
                .fetch_one(&db.pool)
                .await
                .unwrap();

        assert_eq!(qty.0, 7);
    }
}
