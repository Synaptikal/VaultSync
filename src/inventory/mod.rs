use crate::core::{Condition, InventoryItem, InventoryItemWithProduct};
use crate::database::repositories::inventory::InventoryRepository;
use crate::errors::Result;
use std::sync::Arc;
use tracing;

#[derive(Clone)]
pub struct InventoryService {
    repository: Arc<InventoryRepository>,
}

impl InventoryService {
    pub fn new(repository: InventoryRepository) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn add_item(&self, mut item: InventoryItem) -> Result<()> {
        tracing::debug!(
            "Adding item to inventory: product={}, condition={:?}, quantity={}",
            item.product_uuid,
            item.condition,
            item.quantity_on_hand
        );

        // If item has serialized details or specific price, it is unique and should not be aggregated
        let is_unique = item.serialized_details.is_some() || item.specific_price.is_some();

        if !is_unique {
            // Check if item already exists with same product_uuid and condition
            // If so, update quantity instead of creating new entry
            let existing_items = self
                .get_items_by_product_and_condition(item.product_uuid, item.condition.clone())
                .await?;

            // Find a bulk pile (no serialized details, no specific price)
            let bulk_pile = existing_items
                .into_iter()
                .find(|i| i.serialized_details.is_none() && i.specific_price.is_none());

            if let Some(mut existing_item) = bulk_pile {
                // Update existing item quantity
                let new_qty = existing_item.quantity_on_hand + item.quantity_on_hand;
                existing_item.quantity_on_hand = new_qty;

                self.repository
                    .update_quantity(existing_item.inventory_uuid, item.quantity_on_hand)
                    .await?;

                tracing::debug!(
                    "Updated existing inventory item: {} -> {}",
                    existing_item.quantity_on_hand - item.quantity_on_hand,
                    new_qty
                );
                return Ok(());
            }
        }

        // Insert new item (Unique or new Bulk pile)
        if item.inventory_uuid.is_nil() {
            item.inventory_uuid = uuid::Uuid::new_v4();
        }
        self.repository.insert(&item).await?;
        tracing::debug!("Created new inventory item: {}", item.inventory_uuid);

        Ok(())
    }

    pub async fn remove_item(&self, inventory_uuid: uuid::Uuid, quantity: i32) -> Result<()> {
        let item = self
            .repository
            .get_by_id(inventory_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found"))?;

        if item.quantity_on_hand < quantity {
            return Err(anyhow::anyhow!("Not enough quantity in stock"));
        }

        if item.quantity_on_hand == quantity {
            self.repository.delete(inventory_uuid).await?;
        } else {
            self.repository
                .update_quantity(inventory_uuid, -quantity)
                .await?;
        }

        Ok(())
    }

    pub async fn get_item(&self, inventory_uuid: uuid::Uuid) -> Option<InventoryItem> {
        self.repository
            .get_by_id(inventory_uuid)
            .await
            .ok()
            .flatten()
    }

    pub async fn get_items_by_product(
        &self,
        product_uuid: uuid::Uuid,
    ) -> Result<Vec<InventoryItem>> {
        self.repository.get_by_product(product_uuid).await
    }

    pub async fn get_items_by_product_and_condition(
        &self,
        product_uuid: uuid::Uuid,
        condition: Condition,
    ) -> Result<Vec<InventoryItem>> {
        // ARCH-02: Moved filtering logic from Database proxy to Service
        let all = self.repository.get_by_product(product_uuid).await?;
        Ok(all
            .into_iter()
            .filter(|i| i.condition == condition)
            .collect())
    }

    pub async fn get_all_items(&self) -> Result<Vec<InventoryItem>> {
        self.repository.get_all().await
    }

    pub async fn get_items_paginated(&self, limit: i32, offset: i32) -> Result<Vec<InventoryItem>> {
        self.repository.get_paginated(limit, offset).await
    }

    pub async fn get_items_with_products_paginated(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<InventoryItemWithProduct>> {
        self.repository
            .get_paginated_with_products(limit, offset)
            .await
    }

    pub async fn search_items(&self, query: &str) -> Result<Vec<InventoryItem>> {
        self.repository.search_by_name(query, 100).await
    }

    pub async fn update_condition(
        &self,
        inventory_uuid: uuid::Uuid,
        new_condition: Condition,
    ) -> Result<()> {
        let mut item = self
            .repository
            .get_by_id(inventory_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found"))?;

        item.condition = new_condition;
        self.repository.insert(&item).await?; // Repository use INSERT OR REPLACE
        Ok(())
    }

    pub async fn update_quantity(
        &self,
        inventory_uuid: uuid::Uuid,
        new_quantity: i32,
    ) -> Result<()> {
        let item = self
            .repository
            .get_by_id(inventory_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found"))?;

        let diff = new_quantity - item.quantity_on_hand;
        if diff != 0 {
            self.repository
                .update_quantity(inventory_uuid, diff)
                .await?;
        }
        Ok(())
    }

    pub async fn get_low_stock_items(&self, threshold: i32) -> Result<Vec<InventoryItem>> {
        self.repository.get_low_stock(threshold).await
    }

    pub async fn bulk_update(&self, items: Vec<InventoryItem>) -> Result<()> {
        for item in items {
            self.add_item(item).await?;
        }
        Ok(())
    }

    pub async fn get_inventory_matrix(
        &self,
        product_uuid: uuid::Uuid,
    ) -> Result<std::collections::HashMap<Condition, i32>> {
        let items = self.get_items_by_product(product_uuid).await?;
        let mut matrix = std::collections::HashMap::new();

        for item in items {
            let count = matrix.entry(item.condition).or_insert(0);
            *count += item.quantity_on_hand;
        }

        Ok(matrix)
    }
    pub async fn update_item(&self, item: InventoryItem) -> Result<()> {
        self.repository.insert(&item).await
    }

    /// Submits a blind count for audit purposes and checks for discrepancies.
    ///
    /// # Arguments
    /// * `scanned_items` - A list of (product_uuid, condition, quantity) tuples representing physical counts.
    ///
    /// # Returns
    /// * A list of discrepancies found (product_uuid, condition, expected, actual).
    pub async fn submit_blind_count(
        &self,
        scanned_items: Vec<(uuid::Uuid, Condition, i32)>,
    ) -> Result<Vec<crate::core::AuditDiscrepancy>> {
        let mut discrepancies = Vec::new();

        // 1. Group scanned items by Product+Condition
        let mut scanned_map: std::collections::HashMap<(uuid::Uuid, Condition), i32> =
            std::collections::HashMap::new();
        for (pid, cond, qty) in scanned_items {
            *scanned_map.entry((pid, cond)).or_insert(0) += qty;
        }

        // 2. Fetch all system inventory (Optimized: fetch by product or batch? For now, iterative check)

        for ((pid, cond), actual_qty) in scanned_map.iter() {
            let db_items = self
                .get_items_by_product_and_condition(*pid, cond.clone())
                .await?;
            let expected_qty: i32 = db_items.iter().map(|i| i.quantity_on_hand).sum();

            if expected_qty != *actual_qty {
                discrepancies.push(crate::core::AuditDiscrepancy {
                    product_uuid: *pid,
                    condition: cond.clone(),
                    expected_quantity: expected_qty,
                    actual_quantity: *actual_qty,
                    variance: actual_qty - expected_qty,
                });
            }
        }

        Ok(discrepancies)
    }
}
