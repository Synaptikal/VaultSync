use crate::core::{Condition, Customer, WantsItem};
use crate::database::Database;
use crate::errors::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct WantsMatchingService {
    db: Arc<Database>,
}

impl WantsMatchingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// HIGH-007 FIX: Now uses indexed query instead of O(N²) iteration
    /// Checks if a newly acquired item matches any active wants list.
    /// Returns a list of matches (Customer, WantsItem).
    pub async fn find_matches(
        &self,
        product_uuid: Uuid,
        condition: &Condition,
        price: f64,
    ) -> Result<Vec<(Customer, WantsItem)>> {
        // Use the new indexed query to get all wants for this specific product
        let potential_matches = self
            .db
            .customers
            .get_wants_items_by_product(product_uuid)
            .await?;

        let mut matches = Vec::new();

        for (customer, item) in potential_matches {
            // Check condition (is acquired condition >= wanted min_condition?)
            if !self.is_condition_acceptable(condition, &item.min_condition) {
                continue;
            }

            // Check price constraint
            if let Some(max_price) = item.max_price {
                if price > max_price {
                    continue;
                }
            }

            matches.push((customer, item));
        }

        Ok(matches)
    }

    fn is_condition_acceptable(&self, actual: &Condition, min_required: &Condition) -> bool {
        // Simple hierarchy check.
        // A better way is to move this into the Condition enum itself (impl PartialOrd).
        // For now, we'll do a quick map to integer.
        let actual_score = self.condition_score(actual);
        let min_score = self.condition_score(min_required);

        actual_score >= min_score
    }

    fn condition_score(&self, c: &Condition) -> i32 {
        match c {
            Condition::GemMint => 100,
            Condition::Mint => 90,
            Condition::NearMintMint => 85,
            Condition::NM => 80,
            Condition::New => 80, // Eq to NM
            Condition::LP => 70,
            Condition::VeryFine => 70,
            Condition::OpenBox => 70,
            Condition::MP => 60,
            Condition::Fine => 60,
            Condition::Used => 60,
            Condition::HP => 40,
            Condition::Good => 40,
            Condition::DMG => 20,
            Condition::Poor => 20,
        }
    }
}
