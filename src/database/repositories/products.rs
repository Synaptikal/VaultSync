use crate::core::{Category, Product};
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::sync::SyncRepository;

#[derive(Clone)]
pub struct ProductRepository {
    pool: SqlitePool,
    sync: SyncRepository,
}

impl ProductRepository {
    pub fn new(pool: SqlitePool, sync: SyncRepository) -> Self {
        Self { pool, sync }
    }

    pub async fn insert(&self, product: &Product) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO Global_Catalog 
            (product_uuid, name, category, set_code, collector_number, barcode, release_year, metadata,
             weight_oz, length_in, width_in, height_in, upc, isbn, manufacturer, msrp, deleted_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(product.product_uuid.to_string())
        .bind(&product.name)
        .bind(format!("{:?}", product.category))
        .bind(&product.set_code)
        .bind(&product.collector_number)
        .bind(&product.barcode)
        .bind(product.release_year.map(|y| y as i64))
        .bind(serde_json::to_string(&product.metadata)?)
        .bind(product.weight_oz)
        .bind(product.length_in)
        .bind(product.width_in)
        .bind(product.height_in)
        .bind(&product.upc)
        .bind(&product.isbn)
        .bind(&product.manufacturer)
        .bind(product.msrp)
        .bind(product.deleted_at.map(|d| d.to_rfc3339()))
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &product.product_uuid.to_string(),
                "Product",
                "Update",
                &serde_json::to_value(product).unwrap_or_default(),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        product: &Product,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO Global_Catalog 
            (product_uuid, name, category, set_code, collector_number, barcode, release_year, metadata,
             weight_oz, length_in, width_in, height_in, upc, isbn, manufacturer, msrp, deleted_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(product.product_uuid.to_string())
        .bind(&product.name)
        .bind(format!("{:?}", product.category))
        .bind(&product.set_code)
        .bind(&product.collector_number)
        .bind(&product.barcode)
        .bind(product.release_year.map(|y| y as i64))
        .bind(serde_json::to_string(&product.metadata)?)
        .bind(product.weight_oz)
        .bind(product.length_in)
        .bind(product.width_in)
        .bind(product.height_in)
        .bind(&product.upc)
        .bind(&product.isbn)
        .bind(&product.manufacturer)
        .bind(product.msrp)
        .bind(product.deleted_at.map(|d| d.to_rfc3339()))
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                tx,
                &product.product_uuid.to_string(),
                "Product",
                "Update",
                &serde_json::to_value(product).unwrap_or_default(),
            )
            .await?;

        Ok(())
    }

    pub async fn get_by_id(&self, product_uuid: Uuid) -> Result<Option<Product>> {
        let row = sqlx::query("SELECT * FROM Global_Catalog WHERE product_uuid = ?")
            .bind(product_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
                crate::errors::VaultSyncError::DatabaseError(format!("Missing product_uuid: {}", e))
            })?;
            let product_uuid = Uuid::parse_str(&product_uuid_str).map_err(|e| {
                crate::errors::VaultSyncError::ValidationError(format!(
                    "Invalid product_uuid: {}",
                    e
                ))
            })?;
            let name: String = row.try_get("name").unwrap_or_default();
            let category_str: String = row.try_get("category").unwrap_or_default();
            let category = match category_str.as_str() {
                "TCG" => Category::TCG,
                "SportsCard" => Category::SportsCard,
                "Comic" => Category::Comic,
                "Bobblehead" => Category::Bobblehead,
                "Apparel" => Category::Apparel,
                "Figure" => Category::Figure,
                "Accessory" => Category::Accessory,
                _ => Category::Other,
            };
            let set_code: Option<String> = row.try_get("set_code").ok();
            let collector_number: Option<String> = row.try_get("collector_number").ok();
            let barcode: Option<String> = row.try_get("barcode").ok();
            let release_year: Option<i64> = row.try_get("release_year").ok();
            let metadata_str: String = row.try_get("metadata").unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

            let weight_oz: Option<f64> = row.try_get("weight_oz").ok();
            let length_in: Option<f64> = row.try_get("length_in").ok();
            let width_in: Option<f64> = row.try_get("width_in").ok();
            let height_in: Option<f64> = row.try_get("height_in").ok();
            let upc: Option<String> = row.try_get("upc").ok();
            let isbn: Option<String> = row.try_get("isbn").ok();
            let manufacturer: Option<String> = row.try_get("manufacturer").ok();
            let msrp: Option<f64> = row.try_get("msrp").ok();
            let deleted_at_str: Option<String> = row.try_get("deleted_at").ok();
            let deleted_at = deleted_at_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            });

            Ok(Some(Product {
                product_uuid,
                name,
                category,
                set_code,
                collector_number,
                barcode,
                release_year: release_year.map(|y| y as i32),
                metadata,
                weight_oz,
                length_in,
                width_in,
                height_in,
                upc,
                isbn,
                manufacturer,
                msrp,
                deleted_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all(&self) -> Result<Vec<Product>> {
        self.search("", 100, 0).await
    }

    pub async fn get_by_category(&self, category: Category) -> Result<Vec<Product>> {
        let category_str = format!("{:?}", category);
        let rows = sqlx::query("SELECT * FROM Global_Catalog WHERE category = ?")
            .bind(category_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut products = Vec::new();
        for row in rows {
            let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
                crate::errors::VaultSyncError::DatabaseError(format!("Missing product_uuid: {}", e))
            })?;
            let product_uuid = Uuid::parse_str(&product_uuid_str).map_err(|e| {
                crate::errors::VaultSyncError::ValidationError(format!(
                    "Invalid product_uuid: {}",
                    e
                ))
            })?;
            let name: String = row.try_get("name").unwrap_or_default();
            let _category_str: String = row.try_get("category").unwrap_or_default();

            let set_code: Option<String> = row.try_get("set_code").ok();
            let collector_number: Option<String> = row.try_get("collector_number").ok();
            let barcode: Option<String> = row.try_get("barcode").ok();
            let release_year: Option<i64> = row.try_get("release_year").ok();
            let metadata_str: String = row.try_get("metadata").unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

            let weight_oz: Option<f64> = row.try_get("weight_oz").ok();
            let length_in: Option<f64> = row.try_get("length_in").ok();
            let width_in: Option<f64> = row.try_get("width_in").ok();
            let height_in: Option<f64> = row.try_get("height_in").ok();
            let upc: Option<String> = row.try_get("upc").ok();
            let isbn: Option<String> = row.try_get("isbn").ok();
            let manufacturer: Option<String> = row.try_get("manufacturer").ok();
            let msrp: Option<f64> = row.try_get("msrp").ok();
            let deleted_at_str: Option<String> = row.try_get("deleted_at").ok();
            let deleted_at = deleted_at_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            });

            products.push(Product {
                product_uuid,
                name,
                category: category.clone(),
                set_code,
                collector_number,
                barcode,
                release_year: release_year.map(|y| y as i32),
                metadata,
                weight_oz,
                length_in,
                width_in,
                height_in,
                upc,
                isbn,
                manufacturer,
                msrp,
                deleted_at,
            });
        }
        Ok(products)
    }

    pub async fn search(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Product>> {
        let sql = if query.is_empty() {
            "SELECT * FROM Global_Catalog LIMIT ? OFFSET ?".to_string()
        } else {
            "SELECT * FROM Global_Catalog WHERE name LIKE ? OR set_code LIKE ? OR barcode LIKE ? LIMIT ? OFFSET ?".to_string()
        };

        let mut q = sqlx::query(&sql);
        let pattern = format!("%{}%", query);

        if query.is_empty() {
            q = q.bind(limit as i64).bind(offset as i64);
        } else {
            q = q
                .bind(&pattern)
                .bind(&pattern)
                .bind(&pattern) // Bind for barcode
                .bind(limit as i64)
                .bind(offset as i64);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut products = Vec::new();
        for row in rows {
            // Mapping logic duplicated for now - normally I'd use a generic mapper or specific struct FromRow
            let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
                crate::errors::VaultSyncError::DatabaseError(format!("Missing product_uuid: {}", e))
            })?;
            let product_uuid = Uuid::parse_str(&product_uuid_str).map_err(|e| {
                crate::errors::VaultSyncError::ValidationError(format!(
                    "Invalid product_uuid: {}",
                    e
                ))
            })?;
            let name: String = row.try_get("name").unwrap_or_default();
            let category_str: String = row.try_get("category").unwrap_or_default();
            let category = match category_str.as_str() {
                "TCG" => Category::TCG,
                "SportsCard" => Category::SportsCard,
                "Comic" => Category::Comic,
                "Bobblehead" => Category::Bobblehead,
                "Apparel" => Category::Apparel,
                "Figure" => Category::Figure,
                "Accessory" => Category::Accessory,
                _ => Category::Other,
            };
            let set_code: Option<String> = row.try_get("set_code").ok();
            let collector_number: Option<String> = row.try_get("collector_number").ok();
            let barcode: Option<String> = row.try_get("barcode").ok();
            let release_year: Option<i64> = row.try_get("release_year").ok();
            let metadata_str: String = row.try_get("metadata").unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

            let weight_oz: Option<f64> = row.try_get("weight_oz").ok();
            let length_in: Option<f64> = row.try_get("length_in").ok();
            let width_in: Option<f64> = row.try_get("width_in").ok();
            let height_in: Option<f64> = row.try_get("height_in").ok();
            let upc: Option<String> = row.try_get("upc").ok();
            let isbn: Option<String> = row.try_get("isbn").ok();
            let manufacturer: Option<String> = row.try_get("manufacturer").ok();
            let msrp: Option<f64> = row.try_get("msrp").ok();
            let deleted_at_str: Option<String> = row.try_get("deleted_at").ok();
            let deleted_at = deleted_at_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            });

            products.push(Product {
                product_uuid,
                name,
                category,
                set_code,
                collector_number,
                barcode,
                release_year: release_year.map(|y| y as i32),
                metadata,
                weight_oz,
                length_in,
                width_in,
                height_in,
                upc,
                isbn,
                manufacturer,
                msrp,
                deleted_at,
            });
        }

        Ok(products)
    }
}
