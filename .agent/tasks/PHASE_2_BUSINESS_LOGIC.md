# Phase 2: Core Business Logic

**Priority:** P0/P1 - CRITICAL/HIGH  
**Duration:** Weeks 3-5  
**Developers:** 2  
**Status:** ✅ MAJOR PROGRESS (Services Implemented)  
**Depends On:** Phase 1 Complete

---

## Overview
Implement essential business operations including tax calculation, payment processing, and transaction validation. These are fundamental features that every POS system must have.

**IMPLEMENTATION STATUS:**
- ✅ TaxService created (`src/services/tax.rs`)
- ✅ PaymentService created (`src/services/payment.rs`) 
- ✅ HoldsService created (`src/services/holds.rs`)
- ✅ TransactionValidationService created (`src/services/transaction.rs`)
- ✅ API endpoints added for Tax and Holds
- ✅ Services integrated into AppState and main.rs

---

## 2.1 Tax Calculation System

### TASK-046: Create TaxService
**Status:** ✅ COMPLETE  
**File:** `src/services/tax.rs`  
**Implementation:**
```rust
pub struct TaxService {
    db: Arc<Database>,
}

impl TaxService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    /// Get applicable tax rate for a transaction item
    pub async fn get_rate_for_item(&self, item: &TransactionItem, customer: Option<&Customer>) -> Result<f64> {
        // Check customer tax exempt status first
        if let Some(c) = customer {
            if c.tax_exempt {
                return Ok(0.0);
            }
        }
        
        // Check for category-specific rate
        if let Some(product) = self.db.get_product(item.product_uuid).await? {
            if let Some(rate) = self.db.tax.get_for_category(&product.category.to_string()).await? {
                return Ok(rate.rate);
            }
        }
        
        // Fall back to default rate
        if let Some(default) = self.db.tax.get_default().await? {
            return Ok(default.rate);
        }
        
        // No tax configured
        Ok(0.0)
    }
    
    /// Calculate tax for entire transaction
    pub async fn calculate_transaction_tax(
        &self, 
        items: &[TransactionItem], 
        customer: Option<&Customer>
    ) -> Result<TaxBreakdown> {
        let mut total_tax = 0.0;
        let mut item_taxes = Vec::new();
        
        for item in items {
            let rate = self.get_rate_for_item(item, customer).await?;
            let item_total = item.quantity as f64 * item.unit_price;
            let tax = item_total * rate;
            
            item_taxes.push(ItemTax {
                item_uuid: item.item_uuid,
                taxable_amount: item_total,
                tax_rate: rate,
                tax_amount: tax,
            });
            
            total_tax += tax;
        }
        
        Ok(TaxBreakdown {
            items: item_taxes,
            total_tax,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxBreakdown {
    pub items: Vec<ItemTax>,
    pub total_tax: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemTax {
    pub item_uuid: Uuid,
    pub taxable_amount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
}
```

**Acceptance Criteria:**
- [ ] Tax rates loaded from database
- [ ] Customer tax-exempt status respected
- [ ] Category-specific rates work
- [ ] Default rate fallback works
- [ ] Correct rounding (2 decimal places)

---

### TASK-047: Integrate Tax Calculation into Transactions
**Status:** [ ] Not Started  
**File:** `src/transactions/mod.rs`  
**Changes:**
- Add TaxService to TransactionService
- Calculate tax in process_sale
- Store tax_amount in transaction record
- Include tax breakdown in response

---

### TASK-048: Add Customer Tax-Exempt Handling
**Status:** [ ] Not Started  
**Files:** 
- `src/core/mod.rs` - Add tax_exempt field to Customer struct
- `src/services/tax.rs` - Check exempt status
- `src/api/handlers.rs` - Accept tax exempt info in customer creation

---

### TASK-049: Add Category-Based Tax Rate Support
**Status:** [ ] Not Started  
**Already part of TASK-046 implementation**

---

### TASK-050: Create Tax Rate CRUD API Endpoints
**Status:** [ ] Not Started  
**File:** `src/api/handlers.rs`  
**Add:**
```rust
// GET /api/settings/tax-rates
pub async fn get_tax_rates(State(state): State<AppState>) -> impl IntoResponse { }

// POST /api/settings/tax-rates
pub async fn create_tax_rate(State(state): State<AppState>, Json(rate): Json<TaxRate>) -> impl IntoResponse { }

// PUT /api/settings/tax-rates/:rate_id
pub async fn update_tax_rate(...) -> impl IntoResponse { }

// DELETE /api/settings/tax-rates/:rate_id
pub async fn delete_tax_rate(...) -> impl IntoResponse { }
```

**File:** `src/api/mod.rs`  
**Add routes to manager_routes (admin only)**

---

### TASK-051: Add Tax Amount to Transaction Response
**Status:** [ ] Not Started  
**File:** `src/core/mod.rs`  
**Update Transaction struct:**
```rust
pub struct Transaction {
    // ... existing fields
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub tax_breakdown: Option<Vec<ItemTax>>,
}
```

---

## 2.2 Payment Processing

### TASK-052: Create PaymentService Interface
**Status:** [ ] Not Started  
**File:** `src/services/payment.rs` (new)  
```rust
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethodType {
    Cash,
    Card,
    StoreCredit,
    Check,
    GiftCard,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub method: PaymentMethodType,
    pub amount: f64,
    pub reference: Option<String>,
    pub card_info: Option<CardInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub payment_uuid: Uuid,
    pub method: PaymentMethodType,
    pub amount: f64,
    pub reference: Option<String>,
    pub error: Option<String>,
}

pub struct PaymentService {
    db: Arc<Database>,
}

impl PaymentService {
    pub async fn process_payment(
        &self,
        transaction_uuid: Uuid,
        request: PaymentRequest,
    ) -> Result<PaymentResult> {
        match request.method {
            PaymentMethodType::Cash => self.process_cash(transaction_uuid, request).await,
            PaymentMethodType::StoreCredit => self.process_store_credit(transaction_uuid, request).await,
            PaymentMethodType::Card => self.process_card(transaction_uuid, request).await,
            PaymentMethodType::Split => Err(VaultSyncError::PaymentError("Use process_split_payment for split payments".into())),
            _ => Err(VaultSyncError::PaymentError("Unsupported payment method".into())),
        }
    }
    
    pub async fn process_split_payment(
        &self,
        transaction_uuid: Uuid,
        payments: Vec<PaymentRequest>,
        total_due: f64,
    ) -> Result<Vec<PaymentResult>> {
        // Validate total matches
        let payment_total: f64 = payments.iter().map(|p| p.amount).sum();
        if (payment_total - total_due).abs() > 0.01 {
            return Err(VaultSyncError::PaymentError(
                format!("Payment total {} does not match amount due {}", payment_total, total_due)
            ));
        }
        
        // Process each payment
        let mut results = Vec::new();
        for payment in payments {
            let result = self.process_payment(transaction_uuid, payment).await?;
            results.push(result);
        }
        
        Ok(results)
    }
}
```

---

### TASK-053: Implement Cash Payment Handler
**Status:** [ ] Not Started  
**File:** `src/services/payment.rs`  
```rust
impl PaymentService {
    async fn process_cash(
        &self,
        transaction_uuid: Uuid,
        request: PaymentRequest,
    ) -> Result<PaymentResult> {
        let payment = PaymentMethod {
            payment_uuid: Uuid::new_v4(),
            transaction_uuid,
            method_type: "Cash".to_string(),
            amount: request.amount,
            reference: None,
            card_last_four: None,
            auth_code: None,
            created_at: Utc::now(),
        };
        
        self.db.payments.insert(&payment).await?;
        
        Ok(PaymentResult {
            success: true,
            payment_uuid: payment.payment_uuid,
            method: PaymentMethodType::Cash,
            amount: request.amount,
            reference: None,
            error: None,
        })
    }
}
```

---

### TASK-054: Implement Store Credit Payment Handler
**Status:** [ ] Not Started  
**File:** `src/services/payment.rs`  
```rust
async fn process_store_credit(
    &self,
    transaction_uuid: Uuid,
    request: PaymentRequest,
    customer_uuid: Uuid,
) -> Result<PaymentResult> {
    // Get customer and verify credit balance
    let customer = self.db.get_customer(customer_uuid).await?
        .ok_or(VaultSyncError::NotFound("Customer not found"))?;
    
    if customer.store_credit < request.amount {
        return Err(VaultSyncError::PaymentError(
            format!("Insufficient store credit. Available: ${:.2}", customer.store_credit)
        ));
    }
    
    // Deduct credit
    let new_balance = customer.store_credit - request.amount;
    self.db.update_customer_store_credit(customer_uuid, new_balance).await?;
    
    // Record payment
    let payment = PaymentMethod {
        payment_uuid: Uuid::new_v4(),
        transaction_uuid,
        method_type: "StoreCredit".to_string(),
        amount: request.amount,
        reference: Some(format!("Balance: ${:.2}", new_balance)),
        card_last_four: None,
        auth_code: None,
        created_at: Utc::now(),
    };
    
    self.db.payments.insert(&payment).await?;
    
    Ok(PaymentResult {
        success: true,
        payment_uuid: payment.payment_uuid,
        method: PaymentMethodType::StoreCredit,
        amount: request.amount,
        reference: Some(format!("New balance: ${:.2}", new_balance)),
        error: None,
    })
}
```

---

### TASK-055: Implement Split Payment Support
**Status:** [ ] Not Started  
**See TASK-052 for outline**  
**Additional:**
- Create API endpoint for split payment
- Frontend UI for entering split amounts
- Validation that total matches

---

### TASK-056: Add Payment Recording to Transactions
**Status:** [ ] Not Started  
**File:** `src/transactions/mod.rs`  
**Update process_sale to:**
1. Create transaction record
2. Process payment(s)
3. Link payment records to transaction
4. Return complete transaction with payments

---

### TASK-057: Create Payment Reconciliation Logic
**Status:** [ ] Not Started  
**File:** `src/services/payment.rs`  
```rust
pub struct PaymentReconciliation {
    pub date: NaiveDate,
    pub cash_total: f64,
    pub card_total: f64,
    pub credit_total: f64,
    pub check_total: f64,
    pub expected_total: f64,
    pub actual_total: f64,
    pub variance: f64,
}

impl PaymentService {
    pub async fn get_daily_reconciliation(&self, date: NaiveDate) -> Result<PaymentReconciliation> {
        // Query payments by type for date
        // Compare against transaction totals
        // Calculate variance
    }
}
```

---

### TASK-058: Add Change Calculation for Cash Payments
**Status:** [ ] Not Started  
**File:** `src/services/payment.rs`  
```rust
#[derive(Debug, Serialize)]
pub struct CashPaymentResult {
    pub payment_result: PaymentResult,
    pub cash_tendered: f64,
    pub change_due: f64,
}

impl PaymentService {
    pub async fn process_cash_with_change(
        &self,
        transaction_uuid: Uuid,
        amount_due: f64,
        cash_tendered: f64,
    ) -> Result<CashPaymentResult> {
        if cash_tendered < amount_due {
            return Err(VaultSyncError::PaymentError(
                format!("Insufficient cash. Due: ${:.2}, Tendered: ${:.2}", amount_due, cash_tendered)
            ));
        }
        
        let change_due = cash_tendered - amount_due;
        
        let payment_result = self.process_cash(
            transaction_uuid, 
            PaymentRequest { method: PaymentMethodType::Cash, amount: amount_due, .. }
        ).await?;
        
        // Update transaction with cash tendered and change
        self.db.transactions.update_cash_info(transaction_uuid, cash_tendered, change_due).await?;
        
        Ok(CashPaymentResult {
            payment_result,
            cash_tendered,
            change_due,
        })
    }
}
```

---

## 2.3 Transaction Validation Fixes

### TASK-059: Add Quantity Validation
**Status:** [ ] Not Started  
**File:** `src/transactions/mod.rs`  
```rust
fn validate_transaction_items(items: &[TransactionItem]) -> Result<()> {
    for item in items {
        if item.quantity <= 0 {
            return Err(VaultSyncError::ValidationError(
                format!("Invalid quantity {} for item {}", item.quantity, item.product_uuid)
            ));
        }
        
        if item.quantity > 10000 {
            return Err(VaultSyncError::ValidationError(
                "Quantity exceeds maximum allowed (10000)".into()
            ));
        }
    }
    Ok(())
}
```

---

### TASK-060: Add Price Validation
**Status:** [ ] Not Started  
**File:** `src/transactions/mod.rs`  
```rust
fn validate_item_prices(items: &[TransactionItem]) -> Result<()> {
    for item in items {
        if item.unit_price < 0.0 {
            return Err(VaultSyncError::ValidationError(
                format!("Negative price not allowed: ${}", item.unit_price)
            ));
        }
        
        if item.unit_price > 100000.0 {
            return Err(VaultSyncError::ValidationError(
                format!("Price ${} exceeds maximum. Manager approval required.", item.unit_price)
            ));
        }
    }
    Ok(())
}
```

---

### TASK-061: Implement Atomic Transaction with Inventory Update
**Status:** [ ] Not Started  
**File:** `src/database/repositories/transactions.rs`  
**Use SQL transaction for atomicity:**
```rust
pub async fn execute_sale_atomic(
    &self,
    customer_uuid: Option<Uuid>,
    items: Vec<TransactionItem>,
) -> Result<Transaction> {
    let mut tx = self.pool.begin().await?;
    
    // 1. Validate and reserve inventory
    for item in &items {
        let inventory = sqlx::query("SELECT quantity_on_hand FROM Local_Inventory WHERE product_uuid = ? AND condition = ?")
            .bind(item.product_uuid.to_string())
            .bind(item.condition.to_string())
            .fetch_one(&mut *tx)
            .await?;
        
        let available: i32 = inventory.try_get("quantity_on_hand")?;
        if available < item.quantity {
            // Rollback will happen automatically when tx is dropped
            return Err(VaultSyncError::InsufficientStock(
                format!("Only {} available", available)
            ));
        }
        
        // Deduct inventory
        sqlx::query("UPDATE Local_Inventory SET quantity_on_hand = quantity_on_hand - ? WHERE ...")
            .bind(item.quantity)
            .execute(&mut *tx)
            .await?;
    }
    
    // 2. Create transaction record
    let transaction = Transaction::new(customer_uuid, items, TransactionType::Sale);
    sqlx::query("INSERT INTO Transactions ...")
        .execute(&mut *tx)
        .await?;
    
    // 3. Create transaction items
    for item in &transaction.items {
        sqlx::query("INSERT INTO Transaction_Items ...")
            .execute(&mut *tx)
            .await?;
    }
    
    // 4. Commit
    tx.commit().await?;
    
    Ok(transaction)
}
```

---

### TASK-062: Add Transaction Rollback on Partial Failure
**Status:** [ ] Not Started  
**Part of TASK-061 - SQL transactions auto-rollback on error**

---

### TASK-063: Fix Bulk Operations to Use Transactions
**Status:** [ ] Not Started  
**File:** `src/api/handlers.rs`  
**Current:** Loop with individual inserts (line 128-136)  
**Fix:** Use database transaction for all-or-nothing

---

### TASK-064: Add Proper Error Messages for Validation
**Status:** [ ] Not Started  
**File:** `src/errors/mod.rs`  
**Add specific error types:**
```rust
pub enum VaultSyncError {
    // ... existing
    ValidationError(String),
    InsufficientStock(String),
    InsufficientCredit(String),
    PaymentError(String),
    PriceError(String),
}
```

---

## 2.4 Inventory Validation

### TASK-065: Add Stock Level Validation Before Sale
**Status:** [ ] Not Started  
**Part of TASK-061**

---

### TASK-066: Implement Negative Stock Prevention
**Status:** [ ] Not Started  
**File:** `src/database/repositories/inventory.rs`  
```rust
pub async fn update_quantity(&self, uuid: Uuid, delta: i32) -> Result<()> {
    // First check if result would be negative
    let current = sqlx::query("SELECT quantity_on_hand FROM Local_Inventory WHERE inventory_uuid = ?")
        .bind(uuid.to_string())
        .fetch_one(&self.pool)
        .await?;
    
    let current_qty: i32 = current.try_get("quantity_on_hand")?;
    let new_qty = current_qty + delta;
    
    if new_qty < 0 {
        return Err(VaultSyncError::InsufficientStock(
            format!("Cannot reduce below zero. Current: {}, Requested: {}", current_qty, delta.abs())
        ));
    }
    
    sqlx::query("UPDATE Local_Inventory SET quantity_on_hand = ? WHERE inventory_uuid = ?")
        .bind(new_qty)
        .bind(uuid.to_string())
        .execute(&self.pool)
        .await?;
    
    Ok(())
}
```

---

### TASK-067: Add Low Stock Warnings on Sale
**Status:** [ ] Not Started  
**File:** `src/transactions/mod.rs`  
**After sale completion, check remaining stock:**
```rust
pub struct SaleResult {
    pub transaction: Transaction,
    pub low_stock_warnings: Vec<LowStockWarning>,
}

#[derive(Debug, Serialize)]
pub struct LowStockWarning {
    pub product_uuid: Uuid,
    pub product_name: String,
    pub remaining_quantity: i32,
    pub reorder_point: i32,
}
```

---

### TASK-068: Implement Reserved Quantity for Holds
**Status:** [ ] Not Started  
**File:** `src/database/repositories/inventory.rs`  
**Add reserved_quantity column and logic**

---

### TASK-069: Add Inventory Adjustment Audit Logging
**Status:** [ ] Not Started  
**File:** `src/audit/mod.rs`  
```rust
pub async fn log_inventory_adjustment(
    &self,
    inventory_uuid: Uuid,
    old_quantity: i32,
    new_quantity: i32,
    reason: &str,
    user_uuid: Option<Uuid>,
) -> Result<()> {
    // Log to audit table
}
```

---

## Completion Checklist

- [ ] TaxService fully implemented
- [ ] Tax calculation integrated into transactions
- [ ] PaymentService fully implemented
- [ ] All payment types working
- [ ] Split payments working
- [ ] All validation implemented
- [ ] Atomic transactions working
- [ ] Rollback on failure verified
- [ ] Low stock warnings working
- [ ] Audit logging for inventory changes

---

## Next Phase
After completing Phase 2, proceed to **Phase 3: Pricing System Completion**
