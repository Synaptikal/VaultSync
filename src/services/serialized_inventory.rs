use crate::database::Database;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Serialized inventory service for unique/graded items
pub struct SerializedInventoryService {
    db: Arc<Database>,
}

/// Detailed serialized item information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedItem {
    pub inventory_uuid: Uuid,
    pub product_uuid: Uuid,
    pub serial_number: Option<String>,
    pub grading: Option<GradingInfo>,
    pub certificate: Option<CertificateInfo>,
    pub provenance: Vec<ProvenanceEntry>,
    pub images: Vec<String>,
    pub custom_price: Option<f64>,
    pub acquisition_cost: Option<f64>,
    pub acquisition_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingInfo {
    pub grader: String, // PSA, BGS, CGC, etc.
    pub grade: String,  // 10, 9.5, NM, etc.
    pub sub_grades: Option<SubGrades>,
    pub cert_number: Option<String>,
    pub graded_date: Option<String>,
    pub population: Option<PopulationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGrades {
    pub centering: Option<String>,
    pub corners: Option<String>,
    pub edges: Option<String>,
    pub surface: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationData {
    pub same_grade: i32,
    pub higher_grade: i32,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub cert_type: String, // COA, Authentication, etc.
    pub issuer: String,
    pub cert_number: Option<String>,
    pub issue_date: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub entry_uuid: Uuid,
    pub event_type: ProvenanceEventType,
    pub description: String,
    pub date: DateTime<Utc>,
    pub source: Option<String>,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceEventType {
    Acquisition,
    Authentication,
    Grading,
    Exhibition,
    Sale,
    Consignment,
    PriceAdjustment,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSearchResult {
    pub inventory_uuid: Uuid,
    pub product_name: String,
    pub serial_number: Option<String>,
    pub grade: Option<String>,
    pub grader: Option<String>,
    pub price: f64,
    pub location: String,
}

impl SerializedInventoryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// TASK-156: Get serialized details for an inventory item
    pub async fn get_serialized_item(
        &self,
        inventory_uuid: Uuid,
    ) -> Result<Option<SerializedItem>> {
        let row = sqlx::query(
            "SELECT i.inventory_uuid, i.product_uuid, i.specific_price, i.serialized_details, i.location_tag
             FROM Inventory i WHERE i.inventory_uuid = ?"
        )
        .bind(inventory_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        if let Some(row) = row {
            use sqlx::Row;
            let details_json: Option<String> = row.try_get("serialized_details").ok();
            let product_uuid_str: String = row
                .try_get("product_uuid")
                .map_err(|e| anyhow::anyhow!("Missing product_uuid: {}", e))?;
            let specific_price: Option<f64> = row.try_get("specific_price").ok();

            // Parse serialized_details JSON
            let details: serde_json::Value = details_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::json!({}));

            let grading = details
                .get("grading")
                .and_then(|g| serde_json::from_value(g.clone()).ok());
            let certificate = details
                .get("certificate")
                .and_then(|c| serde_json::from_value(c.clone()).ok());
            let images: Vec<String> = details
                .get("images")
                .and_then(|i| serde_json::from_value(i.clone()).ok())
                .unwrap_or_default();

            Ok(Some(SerializedItem {
                inventory_uuid,

                product_uuid: Uuid::parse_str(&product_uuid_str).map_err(|e| {
                    anyhow::anyhow!("Invalid product_uuid '{}': {}", product_uuid_str, e)
                })?,
                serial_number: details
                    .get("serial_number")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
                grading,
                certificate,
                provenance: self.get_provenance(inventory_uuid).await?,
                images,
                custom_price: specific_price,
                acquisition_cost: details.get("acquisition_cost").and_then(|c| c.as_f64()),
                acquisition_date: details
                    .get("acquisition_date")
                    .and_then(|d| d.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                notes: details
                    .get("notes")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    /// TASK-156/157/158: Update serialized details
    pub async fn update_serialized_item(
        &self,
        inventory_uuid: Uuid,
        item: &SerializedItem,
    ) -> Result<()> {
        let details = serde_json::json!({
            "serial_number": item.serial_number,
            "grading": item.grading,
            "certificate": item.certificate,
            "images": item.images,
            "acquisition_cost": item.acquisition_cost,
            "acquisition_date": item.acquisition_date.map(|d| d.to_rfc3339()),
            "notes": item.notes,
        });

        sqlx::query(
            "UPDATE Inventory SET serialized_details = ?, specific_price = ? WHERE inventory_uuid = ?"
        )
        .bind(serde_json::to_string(&details).unwrap_or_default())
        .bind(item.custom_price)
        .bind(inventory_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::info!(
            "Updated serialized details for inventory {}",
            inventory_uuid
        );
        Ok(())
    }

    /// TASK-157: Add grading information
    pub async fn add_grading(&self, inventory_uuid: Uuid, grading: GradingInfo) -> Result<()> {
        let mut item = self
            .get_serialized_item(inventory_uuid)
            .await?
            .context("Item not found")?;

        item.grading = Some(grading.clone());
        self.update_serialized_item(inventory_uuid, &item).await?;

        // Log provenance
        self.add_provenance_entry(
            inventory_uuid,
            ProvenanceEventType::Grading,
            format!("Graded {} by {}", grading.grade, grading.grader),
            None,
        )
        .await?;

        Ok(())
    }

    /// TASK-158: Add certificate/COA
    pub async fn add_certificate(
        &self,
        inventory_uuid: Uuid,
        certificate: CertificateInfo,
    ) -> Result<()> {
        let mut item = self
            .get_serialized_item(inventory_uuid)
            .await?
            .context("Item not found")?;

        item.certificate = Some(certificate.clone());
        self.update_serialized_item(inventory_uuid, &item).await?;

        // Log provenance
        self.add_provenance_entry(
            inventory_uuid,
            ProvenanceEventType::Authentication,
            format!("{} by {}", certificate.cert_type, certificate.issuer),
            None,
        )
        .await?;

        Ok(())
    }

    /// TASK-159: Set individual item price
    pub async fn set_custom_price(&self, inventory_uuid: Uuid, price: f64) -> Result<()> {
        let old_price = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT specific_price FROM Inventory WHERE inventory_uuid = ?",
        )
        .bind(inventory_uuid.to_string())
        .fetch_one(&self.db.pool)
        .await
        .context("Database error")?;

        sqlx::query("UPDATE Inventory SET specific_price = ? WHERE inventory_uuid = ?")
            .bind(price)
            .bind(inventory_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .context("Database error")?;

        // Log provenance
        self.add_provenance_entry(
            inventory_uuid,
            ProvenanceEventType::PriceAdjustment,
            format!(
                "Price changed from ${:.2} to ${:.2}",
                old_price.unwrap_or(0.0),
                price
            ),
            Some(price),
        )
        .await?;

        Ok(())
    }

    /// TASK-160: Search by serial number
    pub async fn search_by_serial(&self, query: &str) -> Result<Vec<SerializedSearchResult>> {
        let rows = sqlx::query(
            "SELECT i.inventory_uuid, i.location_tag, i.specific_price, i.serialized_details,
                    p.name as product_name
             FROM Inventory i
             JOIN Products p ON i.product_uuid = p.product_uuid
             WHERE i.serialized_details LIKE ?",
        )
        .bind(format!("%{}%", query))
        .fetch_all(&self.db.pool)
        .await
        .context("Database error")?;

        let mut results = Vec::new();
        for row in rows {
            use sqlx::Row;
            let details_json: String = row
                .try_get("serialized_details")
                .unwrap_or_else(|_| "{}".to_string());
            let details: serde_json::Value =
                serde_json::from_str(&details_json).unwrap_or(serde_json::json!({}));

            let grading = details.get("grading");

            results.push(SerializedSearchResult {
                inventory_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("inventory_uuid")
                        .unwrap_or_else(|_| Uuid::nil().to_string()),
                )
                .unwrap_or_default(),
                product_name: row
                    .try_get("product_name")
                    .unwrap_or_else(|_| "Unknown Product".to_string()),
                serial_number: details
                    .get("serial_number")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
                grade: grading
                    .and_then(|g| g.get("grade"))
                    .and_then(|g| g.as_str())
                    .map(|s| s.to_string()),
                grader: grading
                    .and_then(|g| g.get("grader"))
                    .and_then(|g| g.as_str())
                    .map(|s| s.to_string()),
                price: row.try_get("specific_price").unwrap_or(0.0),
                location: row.try_get("location_tag").unwrap_or_default(),
            });
        }

        Ok(results)
    }

    /// Get provenance history for an item
    async fn get_provenance(&self, inventory_uuid: Uuid) -> Result<Vec<ProvenanceEntry>> {
        let rows = sqlx::query(
            "SELECT entry_uuid, event_type, description, event_date, source, price
             FROM Item_Provenance WHERE inventory_uuid = ? ORDER BY event_date DESC",
        )
        .bind(inventory_uuid.to_string())
        .fetch_all(&self.db.pool)
        .await
        .context("Database error")?;

        let mut entries = Vec::new();
        for row in rows {
            use sqlx::Row;
            let event_type_str: String = row.try_get("event_type").unwrap_or_default();

            entries.push(ProvenanceEntry {
                entry_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("entry_uuid")
                        .unwrap_or_else(|_| Uuid::nil().to_string()),
                )
                .unwrap_or_default(),
                event_type: match event_type_str.as_str() {
                    "acquisition" => ProvenanceEventType::Acquisition,
                    "authentication" => ProvenanceEventType::Authentication,
                    "grading" => ProvenanceEventType::Grading,
                    "exhibition" => ProvenanceEventType::Exhibition,
                    "sale" => ProvenanceEventType::Sale,
                    "consignment" => ProvenanceEventType::Consignment,
                    "price_adjustment" => ProvenanceEventType::PriceAdjustment,
                    _ => ProvenanceEventType::Note,
                },
                description: row.try_get("description").unwrap_or_default(),
                date: DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("event_date").unwrap_or_default(),
                )
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
                source: row.try_get("source").ok(),
                price: row.try_get("price").ok(),
            });
        }

        Ok(entries)
    }

    /// Add provenance entry
    async fn add_provenance_entry(
        &self,
        inventory_uuid: Uuid,
        event_type: ProvenanceEventType,
        description: String,
        price: Option<f64>,
    ) -> Result<()> {
        let event_type_str = match event_type {
            ProvenanceEventType::Acquisition => "acquisition",
            ProvenanceEventType::Authentication => "authentication",
            ProvenanceEventType::Grading => "grading",
            ProvenanceEventType::Exhibition => "exhibition",
            ProvenanceEventType::Sale => "sale",
            ProvenanceEventType::Consignment => "consignment",
            ProvenanceEventType::PriceAdjustment => "price_adjustment",
            ProvenanceEventType::Note => "note",
        };

        sqlx::query(
            "INSERT INTO Item_Provenance (entry_uuid, inventory_uuid, event_type, description, event_date, price)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(inventory_uuid.to_string())
        .bind(event_type_str)
        .bind(&description)
        .bind(Utc::now().to_rfc3339())
        .bind(price)
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        Ok(())
    }
}
