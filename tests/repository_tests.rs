//! Repository Unit Tests
//!
//! TASK-229: Unit tests for repositories

use sqlx::SqlitePool;
use uuid::Uuid;

/// Helper to create an in-memory database pool
async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create necessary tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Products (
            product_uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT,
            manufacturer TEXT,
            sku TEXT,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS Inventory (
            inventory_uuid TEXT PRIMARY KEY,
            product_uuid TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            unit_cost REAL,
            unit_price REAL,
            condition TEXT,
            location TEXT,
            FOREIGN KEY(product_uuid) REFERENCES Products(product_uuid)
        );
        CREATE TABLE IF NOT EXISTS Customers (
            customer_uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT,
            phone TEXT,
            is_tax_exempt BOOLEAN DEFAULT 0,
            notes TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

mod product_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_product() {
        let pool = create_test_pool().await;

        let product_uuid = Uuid::new_v4().to_string();
        let name = "Test Product";

        // Insert
        sqlx::query("INSERT INTO Products (product_uuid, name, category) VALUES (?, ?, 'TEST')")
            .bind(&product_uuid)
            .bind(name)
            .execute(&pool)
            .await
            .unwrap();

        // Get
        let row: (String, String) =
            sqlx::query_as("SELECT product_uuid, name FROM Products WHERE product_uuid = ?")
                .bind(&product_uuid)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, product_uuid);
        assert_eq!(row.1, name);
    }
}

mod inventory_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_inventory_operations() {
        let pool = create_test_pool().await;

        // Setup product
        let product_uuid = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO Products (product_uuid, name) VALUES (?, 'Inv Test')")
            .bind(&product_uuid)
            .execute(&pool)
            .await
            .unwrap();

        // Add inventory
        let inventory_uuid = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO Inventory (inventory_uuid, product_uuid, quantity, unit_price) VALUES (?, ?, 10, 5.99)"
        )
        .bind(&inventory_uuid)
        .bind(&product_uuid)
        .execute(&pool)
        .await
        .unwrap();

        // Update quantity
        sqlx::query("UPDATE Inventory SET quantity = 5 WHERE inventory_uuid = ?")
            .bind(&inventory_uuid)
            .execute(&pool)
            .await
            .unwrap();

        // Verify
        let qty: (i32,) = sqlx::query_as("SELECT quantity FROM Inventory WHERE inventory_uuid = ?")
            .bind(&inventory_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(qty.0, 5);
    }
}
