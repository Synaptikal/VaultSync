# Phase 1: Database Foundation

**Priority:** P1 - HIGH  
**Duration:** Weeks 2-3  
**Developers:** 2  
**Status:** IN PROGRESS (Migrations 14-19 added)  
**Depends On:** Phase 0 Complete

---

## Overview
Fix the data layer before building features on top. This includes adding missing tables, columns, indexes, and creating proper repository patterns.

---

## 1.1 Schema Migrations - Critical Tables

### TASK-019: Create Payment_Methods Table
**Status:** [ ] Not Started  
**File:** `src/database/mod.rs` - Add migration 14  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Payment_Methods (
    payment_uuid TEXT PRIMARY KEY,
    transaction_uuid TEXT NOT NULL,
    method_type TEXT NOT NULL CHECK(method_type IN ('Cash', 'Card', 'StoreCredit', 'Check', 'Other')),
    amount REAL NOT NULL,
    reference TEXT,
    card_last_four TEXT,
    auth_code TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (transaction_uuid) REFERENCES Transactions(transaction_uuid)
);
CREATE INDEX IF NOT EXISTS idx_payment_methods_transaction ON Payment_Methods(transaction_uuid);
```

**Acceptance Criteria:**
- [ ] Migration runs successfully
- [ ] Table created with all columns
- [ ] Foreign key constraint works
- [ ] Index created

---

### TASK-020: Create Tax_Rates Table
**Status:** [ ] Not Started  
**File:** `src/database/mod.rs` - Add to migration 14  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Tax_Rates (
    rate_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    rate REAL NOT NULL CHECK(rate >= 0 AND rate <= 1),
    applies_to_category TEXT,
    is_default INTEGER DEFAULT 0,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

### TASK-021: Create Store_Locations Table
**Status:** [ ] Not Started  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Store_Locations (
    location_uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT,
    phone TEXT,
    is_primary INTEGER DEFAULT 0,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);
```

---

### TASK-022: Create Holds Table (Layaway)
**Status:** [ ] Not Started  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Holds (
    hold_uuid TEXT PRIMARY KEY,
    customer_uuid TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('Active', 'Completed', 'Cancelled', 'Expired')),
    total_amount REAL NOT NULL,
    deposit_amount REAL NOT NULL,
    balance_due REAL NOT NULL,
    expiration_date TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (customer_uuid) REFERENCES Customers(customer_uuid)
);

CREATE TABLE IF NOT EXISTS Hold_Items (
    item_uuid TEXT PRIMARY KEY,
    hold_uuid TEXT NOT NULL,
    inventory_uuid TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    FOREIGN KEY (hold_uuid) REFERENCES Holds(hold_uuid),
    FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
);

CREATE TABLE IF NOT EXISTS Hold_Payments (
    payment_uuid TEXT PRIMARY KEY,
    hold_uuid TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_method TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (hold_uuid) REFERENCES Holds(hold_uuid)
);

CREATE INDEX IF NOT EXISTS idx_holds_customer ON Holds(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_holds_status ON Holds(status);
CREATE INDEX IF NOT EXISTS idx_hold_items_hold ON Hold_Items(hold_uuid);
```

---

### TASK-023: Create Damaged_Items Table
**Status:** [ ] Not Started  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Damaged_Items (
    damage_uuid TEXT PRIMARY KEY,
    inventory_uuid TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    damage_type TEXT NOT NULL CHECK(damage_type IN ('Defective', 'Damaged', 'Missing', 'Expired', 'Other')),
    description TEXT,
    disposition TEXT CHECK(disposition IN ('Pending', 'WriteOff', 'ReturnToVendor', 'Discounted', 'Repaired')),
    original_value REAL,
    recovered_value REAL DEFAULT 0,
    reported_by TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
);
```

---

### TASK-024: Create Consignment Table
**Status:** [ ] Not Started  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Consignors (
    consignor_uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    commission_rate REAL NOT NULL DEFAULT 0.4,
    balance_owed REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Consignment_Items (
    consignment_uuid TEXT PRIMARY KEY,
    consignor_uuid TEXT NOT NULL,
    inventory_uuid TEXT NOT NULL,
    asking_price REAL NOT NULL,
    minimum_price REAL,
    commission_rate REAL NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('Active', 'Sold', 'Returned', 'Expired')),
    received_date TEXT NOT NULL,
    sold_date TEXT,
    FOREIGN KEY (consignor_uuid) REFERENCES Consignors(consignor_uuid),
    FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
);
```

---

### TASK-025: Create Suppliers Table
**Status:** [ ] Not Started  
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS Suppliers (
    supplier_uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    contact_name TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    payment_terms TEXT,
    notes TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);
```

---

## 1.2 Schema Migrations - Column Additions

### TASK-026: Add Columns to Transactions
**Status:** [ ] Not Started  
**Migration:**
```sql
ALTER TABLE Transactions ADD COLUMN subtotal REAL DEFAULT 0;
ALTER TABLE Transactions ADD COLUMN tax_amount REAL DEFAULT 0;
ALTER TABLE Transactions ADD COLUMN total REAL DEFAULT 0;
ALTER TABLE Transactions ADD COLUMN cash_tendered REAL;
ALTER TABLE Transactions ADD COLUMN change_given REAL;
ALTER TABLE Transactions ADD COLUMN notes TEXT;
ALTER TABLE Transactions ADD COLUMN void_reason TEXT;
ALTER TABLE Transactions ADD COLUMN voided_at TEXT;
ALTER TABLE Transactions ADD COLUMN location_uuid TEXT;
```

---

### TASK-027: Add Columns to Customers
**Status:** [ ] Not Started  
**Migration:**
```sql
ALTER TABLE Customers ADD COLUMN trade_in_limit REAL DEFAULT 500.0;
ALTER TABLE Customers ADD COLUMN is_banned INTEGER DEFAULT 0;
ALTER TABLE Customers ADD COLUMN ban_reason TEXT;
ALTER TABLE Customers ADD COLUMN notes TEXT;
ALTER TABLE Customers ADD COLUMN preferred_contact TEXT CHECK(preferred_contact IN ('Email', 'Phone', 'SMS', 'None'));
ALTER TABLE Customers ADD COLUMN tax_exempt INTEGER DEFAULT 0;
ALTER TABLE Customers ADD COLUMN tax_exempt_id TEXT;
ALTER TABLE Customers ADD COLUMN loyalty_points INTEGER DEFAULT 0;
ALTER TABLE Customers ADD COLUMN tier TEXT DEFAULT 'Standard';
```

---

### TASK-028: Add Columns to Local_Inventory
**Status:** [ ] Not Started  
**Migration:**
```sql
ALTER TABLE Local_Inventory ADD COLUMN cost_basis REAL;
ALTER TABLE Local_Inventory ADD COLUMN supplier_uuid TEXT;
ALTER TABLE Local_Inventory ADD COLUMN received_date TEXT;
ALTER TABLE Local_Inventory ADD COLUMN min_stock_level INTEGER DEFAULT 0;
ALTER TABLE Local_Inventory ADD COLUMN max_stock_level INTEGER;
ALTER TABLE Local_Inventory ADD COLUMN reorder_point INTEGER;
ALTER TABLE Local_Inventory ADD COLUMN location_uuid TEXT;
ALTER TABLE Local_Inventory ADD COLUMN bin_location TEXT;
ALTER TABLE Local_Inventory ADD COLUMN last_sold_date TEXT;
ALTER TABLE Local_Inventory ADD COLUMN last_counted_date TEXT;
```

---

### TASK-029: Add Columns to Global_Catalog
**Status:** [ ] Not Started  
**Migration:**
```sql
ALTER TABLE Global_Catalog ADD COLUMN weight_oz REAL;
ALTER TABLE Global_Catalog ADD COLUMN length_in REAL;
ALTER TABLE Global_Catalog ADD COLUMN width_in REAL;
ALTER TABLE Global_Catalog ADD COLUMN height_in REAL;
ALTER TABLE Global_Catalog ADD COLUMN upc TEXT;
ALTER TABLE Global_Catalog ADD COLUMN isbn TEXT;
ALTER TABLE Global_Catalog ADD COLUMN manufacturer TEXT;
ALTER TABLE Global_Catalog ADD COLUMN msrp REAL;
```

---

### TASK-030: Add Columns to Events
**Status:** [ ] Not Started  
**Migration:**
```sql
ALTER TABLE Events ADD COLUMN prize_pool REAL DEFAULT 0;
ALTER TABLE Events ADD COLUMN format TEXT;
ALTER TABLE Events ADD COLUMN description TEXT;
ALTER TABLE Events ADD COLUMN results_json TEXT;
ALTER TABLE Events ADD COLUMN status TEXT DEFAULT 'Scheduled' CHECK(status IN ('Scheduled', 'InProgress', 'Completed', 'Cancelled'));
ALTER TABLE Events ADD COLUMN location_uuid TEXT;
```

---

## 1.3 Schema Migrations - Indexes

### TASK-031: Add Transaction Type Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_transactions_type ON Transactions(transaction_type);
```

---

### TASK-032: Add Transaction Items Composite Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_transaction_items_product_condition ON Transaction_Items(product_uuid, condition);
```

---

### TASK-033: Add Pricing Matrix Timestamp Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_pricing_matrix_sync ON Pricing_Matrix(last_sync_timestamp);
```

---

### TASK-034: Add Inventory Location Composite Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_inventory_location_product ON Local_Inventory(location_tag, product_uuid);
```

---

### TASK-035: Add UPC Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_products_upc ON Global_Catalog(upc);
```

---

### TASK-036: Add ISBN Index
**Status:** [ ] Not Started  
```sql
CREATE INDEX IF NOT EXISTS idx_products_isbn ON Global_Catalog(isbn);
```

---

## 1.4 Repository Layer

### TASK-037: Create PaymentMethodRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/payments.rs` (new)  
**Methods:**
- `insert(payment: &PaymentMethod) -> Result<()>`
- `get_by_transaction(tx_uuid: Uuid) -> Result<Vec<PaymentMethod>>`
- `get_totals_by_method(start: DateTime, end: DateTime) -> Result<HashMap<String, f64>>`

---

### TASK-038: Create TaxRateRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/tax.rs` (new)  
**Methods:**
- `insert(rate: &TaxRate) -> Result<()>`
- `get_all_active() -> Result<Vec<TaxRate>>`
- `get_default() -> Result<Option<TaxRate>>`
- `get_for_category(category: &str) -> Result<Option<TaxRate>>`
- `update(rate: &TaxRate) -> Result<()>`
- `deactivate(rate_id: &str) -> Result<()>`

---

### TASK-039: Create LocationRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/locations.rs` (new)  

---

### TASK-040: Create HoldRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/holds.rs` (new)  
**Methods:**
- `create(hold: &Hold) -> Result<()>`
- `get_by_id(uuid: Uuid) -> Result<Option<Hold>>`
- `get_active_by_customer(customer_uuid: Uuid) -> Result<Vec<Hold>>`
- `get_expiring_soon(days: i32) -> Result<Vec<Hold>>`
- `add_payment(payment: &HoldPayment) -> Result<()>`
- `update_status(uuid: Uuid, status: HoldStatus) -> Result<()>`
- `get_items(hold_uuid: Uuid) -> Result<Vec<HoldItem>>`

---

### TASK-041: Create SupplierRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/suppliers.rs` (new)  

---

### TASK-042: Update TransactionRepository for New Columns
**Status:** [ ] Not Started  
**File:** `src/database/repositories/transactions.rs`  
**Changes:**
- Update insert to accept new columns
- Update parsing to handle new columns
- Add calculation methods for totals

---

### TASK-043: Update CustomerRepository for New Columns
**Status:** [ ] Not Started  
**File:** `src/database/repositories/customers.rs`  

---

### TASK-044: Update InventoryRepository for New Columns
**Status:** [ ] Not Started  
**File:** `src/database/repositories/inventory.rs`  

---

### TASK-045: Fix N+1 Query in TransactionRepository
**Status:** [ ] Not Started  
**File:** `src/database/repositories/transactions.rs`  
**Issue:** `get_by_customer` loads items separately per transaction  
**Fix:** Use JOIN query to load transactions with items in single query

```rust
pub async fn get_by_customer_with_items(&self, customer_uuid: Uuid) -> Result<Vec<Transaction>> {
    let query = r#"
        SELECT 
            t.transaction_uuid, t.customer_uuid, t.timestamp, t.transaction_type,
            t.subtotal, t.tax_amount, t.total,
            ti.item_uuid, ti.product_uuid, ti.quantity, ti.unit_price, ti.condition
        FROM Transactions t
        LEFT JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
        WHERE t.customer_uuid = ?
        ORDER BY t.timestamp DESC, ti.item_uuid
    "#;
    
    // Process results into grouped transactions
}
```

---

## Completion Checklist

- [ ] Migration 14 created with all new tables
- [ ] Migration 15 created with all column additions
- [ ] Migration 16 created with all new indexes
- [ ] All new repository files created
- [ ] Existing repositories updated
- [ ] All migrations tested
- [ ] N+1 query fixed and verified

---

## Notes

- Run migrations on test database first
- Backup production database before applying
- Column additions use DEFAULT to avoid NULL issues with existing data
- Indexes should be created AFTER data migration for performance

---

## Next Phase
After completing Phase 1, proceed to **Phase 2: Core Business Logic**
