use crate::database::Database;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Cash drawer service for managing physical cash operations
pub struct CashDrawerService {
    db: Arc<Database>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashCount {
    pub count_uuid: Uuid,
    pub shift_uuid: Option<Uuid>,
    pub count_type: CashCountType,
    pub pennies: i32,
    pub nickels: i32,
    pub dimes: i32,
    pub quarters: i32,
    pub ones: i32,
    pub fives: i32,
    pub tens: i32,
    pub twenties: i32,
    pub fifties: i32,
    pub hundreds: i32,
    pub total_amount: f64,
    pub counted_by: Option<Uuid>,
    pub counted_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CashCountType {
    ShiftOpen,
    ShiftClose,
    DropSafe,
    Audit,
    Adjustment,
}

impl std::fmt::Display for CashCountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CashCountType::ShiftOpen => write!(f, "shift_open"),
            CashCountType::ShiftClose => write!(f, "shift_close"),
            CashCountType::DropSafe => write!(f, "drop_safe"),
            CashCountType::Audit => write!(f, "audit"),
            CashCountType::Adjustment => write!(f, "adjustment"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shift {
    pub shift_uuid: Uuid,
    pub user_uuid: Uuid,
    pub terminal_id: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub opening_count_uuid: Option<Uuid>,
    pub closing_count_uuid: Option<Uuid>,
    pub expected_cash: f64,
    pub actual_cash: Option<f64>,
    pub variance: Option<f64>,
    pub status: ShiftStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShiftStatus {
    Open,
    Closed,
    Reconciled,
}

impl std::fmt::Display for ShiftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftStatus::Open => write!(f, "open"),
            ShiftStatus::Closed => write!(f, "closed"),
            ShiftStatus::Reconciled => write!(f, "reconciled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashVarianceReport {
    pub report_date: DateTime<Utc>,
    pub shifts: Vec<ShiftVariance>,
    pub total_expected: f64,
    pub total_actual: f64,
    pub total_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftVariance {
    pub shift_uuid: Uuid,
    pub user_name: String,
    pub terminal_id: String,
    pub expected: f64,
    pub actual: f64,
    pub variance: f64,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl CashDrawerService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// TASK-138: Open the cash drawer (returns command for hardware)
    pub fn get_open_drawer_command(&self) -> Vec<u8> {
        // ESC/POS command to open cash drawer
        // Pulse pin 2 for 100ms
        vec![0x1B, 0x70, 0x00, 0x19, 0xFA]
    }

    /// TASK-139: Record a cash count
    pub async fn record_count(&self, count: &CashCount) -> Result<Uuid> {
        sqlx::query(
            "INSERT INTO Cash_Counts (count_uuid, shift_uuid, count_type, pennies, nickels, dimes, quarters, ones, fives, tens, twenties, fifties, hundreds, total_amount, counted_by, counted_at, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(count.count_uuid.to_string())
        .bind(count.shift_uuid.map(|u| u.to_string()))
        .bind(count.count_type.to_string())
        .bind(count.pennies)
        .bind(count.nickels)
        .bind(count.dimes)
        .bind(count.quarters)
        .bind(count.ones)
        .bind(count.fives)
        .bind(count.tens)
        .bind(count.twenties)
        .bind(count.fifties)
        .bind(count.hundreds)
        .bind(count.total_amount)
        .bind(count.counted_by.map(|u| u.to_string()))
        .bind(count.counted_at.to_rfc3339())
        .bind(&count.notes)
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::info!(
            "Recorded cash count: {} (${:.2})",
            count.count_uuid,
            count.total_amount
        );
        Ok(count.count_uuid)
    }

    /// TASK-141: Calculate total from denomination counts
    pub fn calculate_total(
        pennies: i32,
        nickels: i32,
        dimes: i32,
        quarters: i32,
        ones: i32,
        fives: i32,
        tens: i32,
        twenties: i32,
        fifties: i32,
        hundreds: i32,
    ) -> f64 {
        (pennies as f64 * 0.01)
            + (nickels as f64 * 0.05)
            + (dimes as f64 * 0.10)
            + (quarters as f64 * 0.25)
            + (ones as f64 * 1.00)
            + (fives as f64 * 5.00)
            + (tens as f64 * 10.00)
            + (twenties as f64 * 20.00)
            + (fifties as f64 * 50.00)
            + (hundreds as f64 * 100.00)
    }

    /// TASK-143: Open a new shift
    pub async fn open_shift(
        &self,
        user_uuid: Uuid,
        terminal_id: &str,
        opening_count: Option<&CashCount>,
    ) -> Result<Shift> {
        // Check if there's already an open shift for this terminal
        let existing =
            sqlx::query("SELECT shift_uuid FROM Shifts WHERE terminal_id = ? AND status = 'open'")
                .bind(terminal_id)
                .fetch_optional(&self.db.pool)
                .await
                .context("Database error")?;

        if existing.is_some() {
            bail!("A shift is already open for this terminal");
        }

        let shift_uuid = Uuid::new_v4();
        let now = Utc::now();

        // Record opening count if provided
        let opening_count_uuid = if let Some(count) = opening_count {
            let mut count = count.clone();
            count.shift_uuid = Some(shift_uuid);
            self.record_count(&count).await?;
            Some(count.count_uuid)
        } else {
            None
        };

        let opening_amount = opening_count.map(|c| c.total_amount).unwrap_or(0.0);

        sqlx::query(
            "INSERT INTO Shifts (shift_uuid, user_uuid, terminal_id, opened_at, opening_count_uuid, expected_cash, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(shift_uuid.to_string())
        .bind(user_uuid.to_string())
        .bind(terminal_id)
        .bind(now.to_rfc3339())
        .bind(opening_count_uuid.map(|u| u.to_string()))
        .bind(opening_amount)
        .bind("open")
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::info!(
            "Opened shift {} for terminal {} by user {}",
            shift_uuid,
            terminal_id,
            user_uuid
        );

        Ok(Shift {
            shift_uuid,
            user_uuid,
            terminal_id: terminal_id.to_string(),
            opened_at: now,
            closed_at: None,
            opening_count_uuid,
            closing_count_uuid: None,
            expected_cash: opening_amount,
            actual_cash: None,
            variance: None,
            status: ShiftStatus::Open,
        })
    }

    /// TASK-143: Close a shift with reconciliation
    pub async fn close_shift(&self, shift_uuid: Uuid, closing_count: &CashCount) -> Result<Shift> {
        // Get shift
        let row = sqlx::query(
            "SELECT shift_uuid, user_uuid, terminal_id, opened_at, opening_count_uuid, expected_cash 
             FROM Shifts WHERE shift_uuid = ? AND status = 'open'"
        )
        .bind(shift_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        let row = row.context("Open shift not found")?;

        use sqlx::Row;
        let user_uuid_str: String = row.try_get("user_uuid").unwrap_or_default();
        let terminal_id: String = row.try_get("terminal_id").unwrap_or_default();
        let opened_at_str: String = row.try_get("opened_at").unwrap_or_default();
        let opening_count_uuid_str: Option<String> = row.try_get("opening_count_uuid").ok();
        let expected_from_open: f64 = row.try_get("expected_cash").unwrap_or(0.0);

        // Calculate expected cash = opening + cash sales - cash refunds
        let cash_sales: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM Payment_Methods 
             WHERE shift_uuid = ? AND payment_type = 'Cash'",
        )
        .bind(shift_uuid.to_string())
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        let expected_cash = expected_from_open + cash_sales;
        let actual_cash = closing_count.total_amount;
        let variance = actual_cash - expected_cash;

        // Record closing count
        let mut count = closing_count.clone();
        count.shift_uuid = Some(shift_uuid);
        self.record_count(&count).await?;

        let now = Utc::now();

        // Update shift
        sqlx::query(
            "UPDATE Shifts SET closed_at = ?, closing_count_uuid = ?, expected_cash = ?, actual_cash = ?, variance = ?, status = ?
             WHERE shift_uuid = ?"
        )
        .bind(now.to_rfc3339())
        .bind(count.count_uuid.to_string())
        .bind(expected_cash)
        .bind(actual_cash)
        .bind(variance)
        .bind("closed")
        .bind(shift_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::info!(
            "Closed shift {} - Expected: ${:.2}, Actual: ${:.2}, Variance: ${:.2}",
            shift_uuid,
            expected_cash,
            actual_cash,
            variance
        );

        Ok(Shift {
            shift_uuid,
            user_uuid: Uuid::parse_str(&user_uuid_str).unwrap_or_default(),
            terminal_id,
            opened_at: chrono::DateTime::parse_from_rfc3339(&opened_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
            closed_at: Some(now),
            opening_count_uuid: opening_count_uuid_str.and_then(|s| Uuid::parse_str(&s).ok()),
            closing_count_uuid: Some(count.count_uuid),
            expected_cash,
            actual_cash: Some(actual_cash),
            variance: Some(variance),
            status: ShiftStatus::Closed,
        })
    }

    /// TASK-142: Get current open shift for terminal
    pub async fn get_open_shift(&self, terminal_id: &str) -> Result<Option<Shift>> {
        let row = sqlx::query(
            "SELECT shift_uuid, user_uuid, terminal_id, opened_at, opening_count_uuid, expected_cash 
             FROM Shifts WHERE terminal_id = ? AND status = 'open'"
        )
        .bind(terminal_id)
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(Some(Shift {
                shift_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("shift_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                user_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("user_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                terminal_id: row.try_get("terminal_id").unwrap_or_default(),
                opened_at: chrono::DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("opened_at").unwrap_or_default(),
                )
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
                closed_at: None,
                opening_count_uuid: row
                    .try_get::<String, _>("opening_count_uuid")
                    .ok()
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                closing_count_uuid: None,
                expected_cash: row.try_get("expected_cash").unwrap_or(0.0),
                actual_cash: None,
                variance: None,
                status: ShiftStatus::Open,
            }))
        } else {
            Ok(None)
        }
    }

    /// TASK-144: Generate cash variance report for a date range
    pub async fn get_variance_report(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<CashVarianceReport> {
        let rows = sqlx::query(
            "SELECT s.shift_uuid, s.user_uuid, s.terminal_id, s.opened_at, s.closed_at, s.expected_cash, s.actual_cash, s.variance,
                    u.username as user_name
             FROM Shifts s
             LEFT JOIN Users u ON s.user_uuid = u.user_uuid
             WHERE s.closed_at BETWEEN ? AND ? AND s.status IN ('closed', 'reconciled')
             ORDER BY s.closed_at DESC"
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.db.pool)
        .await
        .context("Database error")?;

        let mut shifts = Vec::new();
        let mut total_expected = 0.0;
        let mut total_actual = 0.0;

        for row in rows {
            use sqlx::Row;
            let expected: f64 = row.try_get("expected_cash").unwrap_or(0.0);
            let actual: f64 = row.try_get("actual_cash").unwrap_or(0.0);
            let variance: f64 = row.try_get("variance").unwrap_or(0.0);

            total_expected += expected;
            total_actual += actual;

            shifts.push(ShiftVariance {
                shift_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("shift_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                user_name: row
                    .try_get("user_name")
                    .unwrap_or_else(|_| "Unknown".to_string()),
                terminal_id: row.try_get("terminal_id").unwrap_or_default(),
                expected,
                actual,
                variance,
                opened_at: chrono::DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("opened_at").unwrap_or_default(),
                )
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
                closed_at: row.try_get::<String, _>("closed_at").ok().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
            });
        }

        Ok(CashVarianceReport {
            report_date: Utc::now(),
            shifts,
            total_expected,
            total_actual,
            total_variance: total_actual - total_expected,
        })
    }
}
