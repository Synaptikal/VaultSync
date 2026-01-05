//! Payment processing service
//!
//! Handles payment recording, split payments, store credit,
//! and cash transactions with change calculation.

use crate::database::Database;
use crate::errors::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Types of payment methods supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethodType {
    Cash,
    Card,
    StoreCredit,
    Check,
    GiftCard,
    Other,
}

impl std::fmt::Display for PaymentMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethodType::Cash => write!(f, "Cash"),
            PaymentMethodType::Card => write!(f, "Card"),
            PaymentMethodType::StoreCredit => write!(f, "StoreCredit"),
            PaymentMethodType::Check => write!(f, "Check"),
            PaymentMethodType::GiftCard => write!(f, "GiftCard"),
            PaymentMethodType::Other => write!(f, "Other"),
        }
    }
}

impl std::str::FromStr for PaymentMethodType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cash" => Ok(PaymentMethodType::Cash),
            "card" => Ok(PaymentMethodType::Card),
            "storecredit" | "store_credit" => Ok(PaymentMethodType::StoreCredit),
            "check" => Ok(PaymentMethodType::Check),
            "giftcard" | "gift_card" => Ok(PaymentMethodType::GiftCard),
            "other" => Ok(PaymentMethodType::Other),
            _ => Err(anyhow::anyhow!("Invalid payment method: {}", s)),
        }
    }
}

/// A single payment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_uuid: Uuid,
    pub transaction_uuid: Uuid,
    pub method_type: PaymentMethodType,
    pub amount: f64,
    pub reference: Option<String>,
    pub card_last_four: Option<String>,
    pub auth_code: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

/// Request to process a payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub method: PaymentMethodType,
    pub amount: f64,
    pub reference: Option<String>,
    pub card_last_four: Option<String>,
}

/// Result of processing a payment
#[derive(Debug, Clone, Serialize)]
pub struct PaymentResult {
    pub success: bool,
    pub payment_uuid: Uuid,
    pub method: PaymentMethodType,
    pub amount: f64,
    pub reference: Option<String>,
    pub error: Option<String>,
}

/// Result of a cash payment with change
#[derive(Debug, Clone, Serialize)]
pub struct CashPaymentResult {
    pub payment: PaymentResult,
    pub cash_tendered: f64,
    pub change_due: f64,
}

/// Result of processing multiple payments (split payment)
#[derive(Debug, Clone, Serialize)]
pub struct SplitPaymentResult {
    pub payments: Vec<PaymentResult>,
    pub total_paid: f64,
    pub total_due: f64,
    pub fully_paid: bool,
}

/// Service for handling payments
pub struct PaymentService {
    db: Arc<Database>,
}

impl PaymentService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Record a payment for a transaction
    pub async fn record_payment(
        &self,
        transaction_uuid: Uuid,
        request: PaymentRequest,
    ) -> Result<PaymentResult> {
        let payment_uuid = Uuid::new_v4();
        let now = Utc::now();

        // Insert payment record
        sqlx::query(
            "INSERT INTO Payment_Methods 
             (payment_uuid, transaction_uuid, method_type, amount, reference, card_last_four, auth_code, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(payment_uuid.to_string())
        .bind(transaction_uuid.to_string())
        .bind(request.method.to_string())
        .bind(request.amount)
        .bind(&request.reference)
        .bind(&request.card_last_four)
        .bind(None::<String>) // auth_code not used for now
        .bind(now.to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(PaymentResult {
            success: true,
            payment_uuid,
            method: request.method,
            amount: request.amount,
            reference: request.reference,
            error: None,
        })
    }

    /// Process a cash payment with change calculation
    pub async fn process_cash_payment(
        &self,
        transaction_uuid: Uuid,
        amount_due: f64,
        cash_tendered: f64,
    ) -> Result<CashPaymentResult> {
        // Validate sufficient cash
        if cash_tendered < amount_due {
            return Err(anyhow::anyhow!(
                "Insufficient cash. Due: ${:.2}, Tendered: ${:.2}",
                amount_due,
                cash_tendered
            ));
        }

        let change_due = ((cash_tendered - amount_due) * 100.0).round() / 100.0;

        // Record the payment
        let payment = self
            .record_payment(
                transaction_uuid,
                PaymentRequest {
                    method: PaymentMethodType::Cash,
                    amount: amount_due,
                    reference: Some(format!(
                        "Tendered: ${:.2}, Change: ${:.2}",
                        cash_tendered, change_due
                    )),
                    card_last_four: None,
                },
            )
            .await?;

        // Update transaction with cash details
        sqlx::query(
            "UPDATE Transactions SET cash_tendered = ?, change_given = ? WHERE transaction_uuid = ?",
        )
        .bind(cash_tendered)
        .bind(change_due)
        .bind(transaction_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(CashPaymentResult {
            payment,
            cash_tendered,
            change_due,
        })
    }

    /// Process a store credit payment
    pub async fn process_store_credit_payment(
        &self,
        transaction_uuid: Uuid,
        customer_uuid: Uuid,
        amount: f64,
    ) -> Result<PaymentResult> {
        // Get current customer credit balance
        let row = sqlx::query(
            "SELECT store_credit FROM Customers WHERE customer_uuid = ? AND deleted_at IS NULL",
        )
        .bind(customer_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let current_credit: f64 = match row {
            Some(r) => sqlx::Row::try_get(&r, "store_credit").unwrap_or(0.0),
            None => {
                return Err(anyhow::anyhow!("Customer {} not found", customer_uuid));
            }
        };

        // Check sufficient credit
        if current_credit < amount {
            return Err(anyhow::anyhow!(
                "Insufficient store credit. Available: ${:.2}, Requested: ${:.2}",
                current_credit,
                amount
            ));
        }

        // Deduct credit
        let new_balance = ((current_credit - amount) * 100.0).round() / 100.0;
        sqlx::query("UPDATE Customers SET store_credit = ? WHERE customer_uuid = ?")
            .bind(new_balance)
            .bind(customer_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        // Record payment
        let payment = self
            .record_payment(
                transaction_uuid,
                PaymentRequest {
                    method: PaymentMethodType::StoreCredit,
                    amount,
                    reference: Some(format!("New balance: ${:.2}", new_balance)),
                    card_last_four: None,
                },
            )
            .await?;

        tracing::info!(
            "Store credit payment: {} paid ${:.2}, new balance ${:.2}",
            customer_uuid,
            amount,
            new_balance
        );

        Ok(payment)
    }

    /// Process a split payment (multiple payment methods)
    pub async fn process_split_payment(
        &self,
        transaction_uuid: Uuid,
        customer_uuid: Option<Uuid>,
        payments: Vec<PaymentRequest>,
        total_due: f64,
    ) -> Result<SplitPaymentResult> {
        // Validate total
        let total_payments: f64 = payments.iter().map(|p| p.amount).sum();
        let tolerance = 0.01; // 1 cent tolerance for floating point

        if (total_payments - total_due).abs() > tolerance && total_payments < total_due {
            return Err(anyhow::anyhow!(
                "Payment total ${:.2} does not match amount due ${:.2}",
                total_payments,
                total_due
            ));
        }

        let mut results = Vec::new();
        let mut actual_total = 0.0;

        for payment_request in payments {
            let result = match payment_request.method {
                PaymentMethodType::StoreCredit => {
                    let cust_uuid = customer_uuid.ok_or_else(|| {
                        anyhow::anyhow!("Customer required for store credit payment")
                    })?;
                    self.process_store_credit_payment(
                        transaction_uuid,
                        cust_uuid,
                        payment_request.amount,
                    )
                    .await?
                }
                PaymentMethodType::Cash => {
                    // For split, cash is exact (no change on partial)
                    self.record_payment(transaction_uuid, payment_request)
                        .await?
                }
                _ => {
                    self.record_payment(transaction_uuid, payment_request)
                        .await?
                }
            };

            actual_total += result.amount;
            results.push(result);
        }

        Ok(SplitPaymentResult {
            payments: results,
            total_paid: actual_total,
            total_due,
            fully_paid: (actual_total - total_due).abs() <= tolerance,
        })
    }

    /// Get all payments for a transaction
    pub async fn get_payments_for_transaction(
        &self,
        transaction_uuid: Uuid,
    ) -> Result<Vec<PaymentRecord>> {
        let rows = sqlx::query(
            "SELECT payment_uuid, transaction_uuid, method_type, amount, reference, card_last_four, auth_code, created_at
             FROM Payment_Methods WHERE transaction_uuid = ?
             ORDER BY created_at ASC",
        )
        .bind(transaction_uuid.to_string())
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut payments = Vec::new();
        for row in rows {
            let method_str: String = sqlx::Row::try_get(&row, "method_type").unwrap_or_default();
            payments.push(PaymentRecord {
                payment_uuid: Uuid::parse_str(
                    &sqlx::Row::try_get::<String, _>(&row, "payment_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                transaction_uuid,
                method_type: method_str.parse().unwrap_or(PaymentMethodType::Other),
                amount: sqlx::Row::try_get(&row, "amount").unwrap_or(0.0),
                reference: sqlx::Row::try_get(&row, "reference").ok(),
                card_last_four: sqlx::Row::try_get(&row, "card_last_four").ok(),
                auth_code: sqlx::Row::try_get(&row, "auth_code").ok(),
                created_at: sqlx::Row::try_get::<String, _>(&row, "created_at")
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            });
        }

        Ok(payments)
    }

    /// Get payment totals by method for a date range
    pub async fn get_payment_totals_by_method(
        &self,
        start_date: chrono::DateTime<Utc>,
        end_date: chrono::DateTime<Utc>,
    ) -> Result<std::collections::HashMap<String, f64>> {
        let rows = sqlx::query(
            "SELECT method_type, SUM(amount) as total
             FROM Payment_Methods
             WHERE created_at >= ? AND created_at <= ?
             GROUP BY method_type",
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut totals = std::collections::HashMap::new();
        for row in rows {
            let method: String = sqlx::Row::try_get(&row, "method_type").unwrap_or_default();
            let total: f64 = sqlx::Row::try_get(&row, "total").unwrap_or(0.0);
            totals.insert(method, total);
        }

        Ok(totals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_display() {
        assert_eq!(PaymentMethodType::Cash.to_string(), "Cash");
        assert_eq!(PaymentMethodType::StoreCredit.to_string(), "StoreCredit");
    }

    #[test]
    fn test_payment_method_parse() {
        assert_eq!(
            "cash".parse::<PaymentMethodType>().unwrap(),
            PaymentMethodType::Cash
        );
        assert_eq!(
            "StoreCredit".parse::<PaymentMethodType>().unwrap(),
            PaymentMethodType::StoreCredit
        );
        assert_eq!(
            "store_credit".parse::<PaymentMethodType>().unwrap(),
            PaymentMethodType::StoreCredit
        );
    }
}

// Extension to support explicit transactions
impl PaymentService {
    /// Record a payment using an existing transaction
    pub async fn record_payment_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        transaction_uuid: Uuid,
        request: PaymentRequest,
    ) -> Result<PaymentResult> {
        let payment_uuid = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO Payment_Methods 
             (payment_uuid, transaction_uuid, method_type, amount, reference, card_last_four, auth_code, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(payment_uuid.to_string())
        .bind(transaction_uuid.to_string())
        .bind(request.method.to_string())
        .bind(request.amount)
        .bind(&request.reference)
        .bind(&request.card_last_four)
        .bind(None::<String>)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(PaymentResult {
            success: true,
            payment_uuid,
            method: request.method,
            amount: request.amount,
            reference: request.reference,
            error: None,
        })
    }

    /// Process a store credit payment using an existing transaction
    pub async fn process_store_credit_payment_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        transaction_uuid: Uuid,
        customer_uuid: Uuid,
        amount: f64,
    ) -> Result<PaymentResult> {
        // Get current customer credit balance - MUST acquire row explicitly or rely on app logic
        // For strict correctness in atomic TX, we should re-read within TX.
        let row = sqlx::query(
            "SELECT store_credit FROM Customers WHERE customer_uuid = ? AND deleted_at IS NULL",
        )
        .bind(customer_uuid.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let current_credit: f64 = match row {
            Some(r) => sqlx::Row::try_get(&r, "store_credit").unwrap_or(0.0),
            None => {
                return Err(anyhow::anyhow!("Customer {} not found", customer_uuid));
            }
        };

        if current_credit < amount {
            return Err(anyhow::anyhow!(
                "Insufficient store credit. Available: ${:.2}, Requested: ${:.2}",
                current_credit,
                amount
            ));
        }

        let new_balance = ((current_credit - amount) * 100.0).round() / 100.0;
        sqlx::query("UPDATE Customers SET store_credit = ? WHERE customer_uuid = ?")
            .bind(new_balance)
            .bind(customer_uuid.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        // Use internal helper so we don't need self.record_payment_with_tx public if we didn't want to
        // But here we use the one we defined above
        let payment = self
            .record_payment_with_tx(
                tx,
                transaction_uuid,
                PaymentRequest {
                    method: PaymentMethodType::StoreCredit,
                    amount,
                    reference: Some(format!("New balance: ${:.2}", new_balance)),
                    card_last_four: None,
                },
            )
            .await?;

        Ok(payment)
    }

    /// Process split payment using an existing transaction
    pub async fn process_split_payment_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        transaction_uuid: Uuid,
        customer_uuid: Option<Uuid>,
        payments: Vec<PaymentRequest>,
        total_due: f64,
    ) -> Result<SplitPaymentResult> {
        // Validate total
        let total_payments: f64 = payments.iter().map(|p| p.amount).sum();
        let tolerance = 0.01;

        if (total_payments - total_due).abs() > tolerance && total_payments < total_due {
            return Err(anyhow::anyhow!(
                "Payment total ${:.2} does not match amount due ${:.2}",
                total_payments,
                total_due
            ));
        }

        let mut results = Vec::new();
        let mut actual_total = 0.0;

        for payment_request in payments {
            let result = match payment_request.method {
                PaymentMethodType::StoreCredit => {
                    let cust_uuid = customer_uuid.ok_or_else(|| {
                        anyhow::anyhow!("Customer required for store credit payment")
                    })?;
                    self.process_store_credit_payment_with_tx(
                        tx,
                        transaction_uuid,
                        cust_uuid,
                        payment_request.amount,
                    )
                    .await?
                }
                _ => {
                    self.record_payment_with_tx(tx, transaction_uuid, payment_request)
                        .await?
                }
            };

            actual_total += result.amount;
            results.push(result);
        }

        Ok(SplitPaymentResult {
            payments: results,
            total_paid: actual_total,
            total_due,
            fully_paid: (actual_total - total_due).abs() <= tolerance,
        })
    }
}
