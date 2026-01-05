use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// P0 Fix: Proper decimal handling for monetary values
pub mod money;
pub use money::Money;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum Category {
    TCG,
    SportsCard,
    Comic,
    Bobblehead,
    Apparel,
    Figure,
    Accessory,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub product_uuid: Uuid,
    pub name: String,
    pub category: Category,
    // Optional fields for specific categories
    pub set_code: Option<String>,
    pub collector_number: Option<String>,
    pub barcode: Option<String>, // UPC/EAN
    pub release_year: Option<i32>,
    #[schema(value_type = Object)]
    pub metadata: serde_json::Value, // Flexible storage for category-specific fields (e.g., "team", "publisher")

    // Phase 14 / Schema Updates
    pub weight_oz: Option<f64>,
    pub length_in: Option<f64>,
    pub width_in: Option<f64>,
    pub height_in: Option<f64>,
    pub upc: Option<String>,
    pub isbn: Option<String>,
    pub manufacturer: Option<String>,
    pub msrp: Option<f64>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryItem {
    pub inventory_uuid: Uuid,
    pub product_uuid: Uuid,
    pub variant_type: Option<VariantType>, // Made optional for non-variant items
    pub condition: Condition,
    pub quantity_on_hand: i32,
    pub location_tag: String,
    pub specific_price: Option<f64>, // Override market price for this specific item
    #[schema(value_type = Object)]
    pub serialized_details: Option<serde_json::Value>, // Cert #, Grader, Images, etc.

    // Phase 14 / Schema Updates
    pub cost_basis: Option<f64>,
    pub supplier_uuid: Option<Uuid>,
    pub received_date: Option<DateTime<Utc>>,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub bin_location: Option<String>,
    pub last_sold_date: Option<DateTime<Utc>>,
    pub last_counted_date: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VariantType {
    Normal,
    Foil,
    ReverseHolo,
    FirstEdition,
    Stamped,
    // Add variants for other categories if needed, or rely on metadata
    Signed,
    Graded,
    Refractor,
    Patch,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum Condition {
    // TCG
    NM,  // Near Mint
    LP,  // Lightly Played
    MP,  // Moderately Played
    HP,  // Heavily Played
    DMG, // Damaged

    // General / Merch
    New,
    OpenBox,
    Used,

    // Comics / Graded
    GemMint,      // 10
    Mint,         // 9.9 - 9.0
    NearMintMint, // 9.8
    VeryFine,
    Fine,
    Good,
    Poor,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryItemWithProduct {
    #[serde(flatten)]
    pub item: InventoryItem,

    // Product details flattened or nested?
    // Frontend expects flattened in some ways but let's nest "product" for cleaner API or flat fields?
    // Report said "Update to return joined Product data".
    // Let's provide an optional "product" field that contains the full Product spec,
    // OR just the essential fields.
    // Given the N+1 issue was fetching the *entire* product, returning the entire product nested is safest for backward compat if we change the outer type.
    // However, existing `InventoryItem` doesn't have `product`.
    // Let's define it as a hybrid to serve the UI needs efficiently.
    pub product_name: String,
    pub category: Category,
    pub product_metadata: serde_json::Value,
    // We can add more fields if needed, like image URL if it was in metadata
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriceInfo {
    pub price_uuid: Uuid,
    pub product_uuid: Uuid,
    pub market_mid: f64,
    pub market_low: f64,
    pub last_sync_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Transaction {
    pub transaction_uuid: Uuid,
    pub items: Vec<TransactionItem>,
    pub customer_uuid: Option<Uuid>,
    pub user_uuid: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionItem {
    pub item_uuid: Uuid,
    pub product_uuid: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
    pub condition: Condition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum TransactionType {
    Sale,
    Buy,
    Trade,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Customer {
    pub customer_uuid: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub store_credit: f64,
    pub tier: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WantsList {
    pub wants_list_uuid: Uuid,
    pub customer_uuid: Uuid,
    pub items: Vec<WantsItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WantsItem {
    pub item_uuid: Uuid,
    pub product_uuid: Uuid,
    pub min_condition: Condition,
    pub max_price: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Event {
    pub event_uuid: Uuid,
    pub name: String,
    pub event_type: String, // e.g., "Tournament", "Casual", "Release"
    pub date: DateTime<Utc>,
    pub entry_fee: f64,
    pub max_participants: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventParticipant {
    pub participant_uuid: Uuid,
    pub event_uuid: Uuid,
    pub customer_uuid: Option<Uuid>, // Optional for walk-ins
    pub name: String,
    pub paid: bool,
    pub placement: Option<i32>,
    pub created_at: DateTime<Utc>,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VectorTimestamp {
    pub entries: HashMap<String, u64>,
}

impl VectorTimestamp {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn from_entries(entries: HashMap<String, u64>) -> Self {
        Self { entries }
    }

    pub fn get_clock(&self, node_id: &str) -> u64 {
        *self.entries.get(node_id).unwrap_or(&0)
    }

    pub fn increment(&mut self, node_id: String) {
        let counter = self.entries.entry(node_id).or_insert(0);
        *counter += 1;
    }

    pub fn merge(&mut self, other: &VectorTimestamp) {
        for (node, &clock) in &other.entries {
            let local_clock = self.entries.entry(node.clone()).or_insert(0);
            *local_clock = std::cmp::max(*local_clock, clock);
        }
    }

    // Returns true if self dominates (is strictly greater or equal to) other
    pub fn dominates(&self, other: &VectorTimestamp) -> bool {
        for (node, &other_clock) in &other.entries {
            if self.get_clock(node) < other_clock {
                return false;
            }
        }
        true
    }

    pub fn compare(&self, other: &VectorTimestamp) -> Ordering {
        let self_dominates = self.dominates(other);
        let other_dominates = other.dominates(self);

        if self_dominates && other_dominates {
            Ordering::Equal
        } else if self_dominates {
            Ordering::Greater
        } else if other_dominates {
            Ordering::Less
        } else {
            Ordering::Concurrent
        }
    }
}

#[derive(Debug, PartialEq, Eq, ToSchema)]
pub enum Ordering {
    Less,
    Greater,
    Equal,
    Concurrent,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum RecordType {
    Product,
    InventoryItem,
    PriceInfo,
    Transaction,
    Customer,
    WantsList,
    Event,
    EventParticipant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum SyncOperation {
    Insert,
    Update,
    Delete,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum PriceStatus {
    Safe,
    Flagged,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriceHistoryEntry {
    pub history_uuid: Uuid,
    pub product_uuid: Uuid,
    pub market_mid: f64,
    pub market_low: f64,
    pub source: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditDiscrepancy {
    pub product_uuid: Uuid,
    pub condition: Condition,
    pub expected_quantity: i32,
    pub actual_quantity: i32,
    pub variance: i32,
}
