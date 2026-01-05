use crate::database::Database;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Trade-in fraud protection service
pub struct TradeInProtectionService {
    db: Arc<Database>,
    /// Maximum trade-in value per customer per day
    daily_limit: f64,
    /// Maximum trade-in value per customer per week
    weekly_limit: f64,
    /// Value threshold requiring ID verification
    id_verification_threshold: f64,
    /// Hold period for trade-ins before they can be resold (days)
    hold_period_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInCheck {
    pub customer_uuid: Uuid,
    pub proposed_value: f64,
    pub is_allowed: bool,
    pub requires_id: bool,
    pub requires_manager_approval: bool,
    pub hold_until: Option<DateTime<Utc>>,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerTradeInHistory {
    pub customer_uuid: Uuid,
    pub daily_total: f64,
    pub weekly_total: f64,
    pub monthly_total: f64,
    pub total_trade_ins: i32,
    pub last_trade_in: Option<DateTime<Utc>>,
    pub is_blacklisted: bool,
    pub blacklist_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInBlacklistEntry {
    pub blacklist_uuid: Uuid,
    pub customer_uuid: Uuid,
    pub reason: String,
    pub added_by: Option<Uuid>,
    pub added_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivity {
    pub alert_uuid: Uuid,
    pub customer_uuid: Uuid,
    pub activity_type: SuspiciousActivityType,
    pub description: String,
    pub severity: AlertSeverity,
    pub detected_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SuspiciousActivityType {
    HighVelocity,         // Too many trade-ins in short time
    HighValue,            // Single high-value trade
    PatternMatch,         // Matches known fraud patterns
    BlacklistedAssociate, // Related to blacklisted customer
    IdMismatch,           // ID verification failed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl TradeInProtectionService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            daily_limit: 500.0,
            weekly_limit: 2000.0,
            id_verification_threshold: 100.0,
            hold_period_days: 7,
        }
    }

    /// Configure limits
    pub fn with_limits(
        mut self,
        daily_limit: f64,
        weekly_limit: f64,
        id_threshold: f64,
        hold_days: i32,
    ) -> Self {
        self.daily_limit = daily_limit;
        self.weekly_limit = weekly_limit;
        self.id_verification_threshold = id_threshold;
        self.hold_period_days = hold_days;
        self
    }

    /// TASK-162/163: Check if a trade-in is allowed based on limits and velocity
    pub async fn check_trade_in(
        &self,
        customer_uuid: Uuid,
        proposed_value: f64,
    ) -> Result<TradeInCheck> {
        let history = self.get_customer_history(customer_uuid).await?;

        let mut check = TradeInCheck {
            customer_uuid,
            proposed_value,
            is_allowed: true,
            requires_id: proposed_value >= self.id_verification_threshold,
            requires_manager_approval: false,
            hold_until: Some(Utc::now() + Duration::days(self.hold_period_days as i64)),
            warnings: Vec::new(),
            blockers: Vec::new(),
        };

        // TASK-164: Check blacklist
        if history.is_blacklisted {
            check.is_allowed = false;
            check.blockers.push(format!(
                "Customer is blacklisted: {}",
                history
                    .blacklist_reason
                    .as_deref()
                    .unwrap_or("No reason provided")
            ));
            return Ok(check);
        }

        // TASK-162: Check daily limit
        let new_daily_total = history.daily_total + proposed_value;
        if new_daily_total > self.daily_limit {
            check.requires_manager_approval = true;
            check.warnings.push(format!(
                "Daily limit exceeded: ${:.2} of ${:.2} limit",
                new_daily_total, self.daily_limit
            ));
        }

        // TASK-162: Check weekly limit
        let new_weekly_total = history.weekly_total + proposed_value;
        if new_weekly_total > self.weekly_limit {
            check.is_allowed = false;
            check.blockers.push(format!(
                "Weekly limit exceeded: ${:.2} of ${:.2} limit",
                new_weekly_total, self.weekly_limit
            ));
        }

        // TASK-163: Check velocity (frequency)
        if history.total_trade_ins >= 3 {
            if let Some(last) = history.last_trade_in {
                let hours_since_last = (Utc::now() - last).num_hours();
                if hours_since_last < 24 {
                    check.warnings.push(format!(
                        "High frequency: {} trade-ins, last was {} hours ago",
                        history.total_trade_ins, hours_since_last
                    ));
                    if history.total_trade_ins >= 5 {
                        check.requires_manager_approval = true;
                    }
                }
            }
        }

        // TASK-165: High value requires ID
        if proposed_value >= self.id_verification_threshold {
            check.requires_id = true;
        }

        Ok(check)
    }

    /// Get customer trade-in history
    pub async fn get_customer_history(
        &self,
        customer_uuid: Uuid,
    ) -> Result<CustomerTradeInHistory> {
        let now = Utc::now();
        let day_ago = now - Duration::days(1);
        let week_ago = now - Duration::days(7);
        let month_ago = now - Duration::days(30);

        // Get trade-in totals
        let daily_total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM Transactions 
             WHERE customer_uuid = ? AND transaction_type = 'Buy' AND created_at > ?",
        )
        .bind(customer_uuid.to_string())
        .bind(day_ago.to_rfc3339())
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        let weekly_total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM Transactions 
             WHERE customer_uuid = ? AND transaction_type = 'Buy' AND created_at > ?",
        )
        .bind(customer_uuid.to_string())
        .bind(week_ago.to_rfc3339())
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        let monthly_total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM Transactions 
             WHERE customer_uuid = ? AND transaction_type = 'Buy' AND created_at > ?",
        )
        .bind(customer_uuid.to_string())
        .bind(month_ago.to_rfc3339())
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        let total_trade_ins: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM Transactions 
             WHERE customer_uuid = ? AND transaction_type = 'Buy' AND created_at > ?",
        )
        .bind(customer_uuid.to_string())
        .bind(week_ago.to_rfc3339())
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0) as i32;

        // Get last trade-in date
        let last_row = sqlx::query(
            "SELECT created_at FROM Transactions 
             WHERE customer_uuid = ? AND transaction_type = 'Buy'
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(customer_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        let last_trade_in = if let Some(row) = last_row {
            use sqlx::Row;
            let ts: String = row
                .try_get("created_at")
                .map_err(|e| anyhow::anyhow!("Missing created_at for trade-in: {}", e))?;
            DateTime::parse_from_rfc3339(&ts)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            None
        };

        // Check blacklist
        let blacklist = self.get_blacklist_entry(customer_uuid).await?;

        Ok(CustomerTradeInHistory {
            customer_uuid,
            daily_total,
            weekly_total,
            monthly_total,
            total_trade_ins,
            last_trade_in,
            is_blacklisted: blacklist.is_some(),
            blacklist_reason: blacklist.map(|b| b.reason),
        })
    }

    /// TASK-164: Add customer to blacklist
    pub async fn add_to_blacklist(
        &self,
        customer_uuid: Uuid,
        reason: &str,
        added_by: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<TradeInBlacklistEntry> {
        let entry = TradeInBlacklistEntry {
            blacklist_uuid: Uuid::new_v4(),
            customer_uuid,
            reason: reason.to_string(),
            added_by,
            added_at: Utc::now(),
            expires_at,
            is_active: true,
        };

        sqlx::query(
            "INSERT INTO Trade_In_Blacklist (blacklist_uuid, customer_uuid, reason, added_by, added_at, expires_at, is_active)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(entry.blacklist_uuid.to_string())
        .bind(entry.customer_uuid.to_string())
        .bind(&entry.reason)
        .bind(entry.added_by.map(|u| u.to_string()))
        .bind(entry.added_at.to_rfc3339())
        .bind(entry.expires_at.map(|d| d.to_rfc3339()))
        .bind(entry.is_active)
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::warn!(
            "Added customer {} to trade-in blacklist: {}",
            customer_uuid,
            reason
        );
        Ok(entry)
    }

    /// TASK-164: Remove from blacklist
    pub async fn remove_from_blacklist(&self, customer_uuid: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE Trade_In_Blacklist SET is_active = 0 WHERE customer_uuid = ? AND is_active = 1",
        )
        .bind(customer_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        Ok(result.rows_affected() > 0)
    }

    /// Get blacklist entry for customer
    async fn get_blacklist_entry(
        &self,
        customer_uuid: Uuid,
    ) -> Result<Option<TradeInBlacklistEntry>> {
        let row = sqlx::query(
            "SELECT blacklist_uuid, customer_uuid, reason, added_by, added_at, expires_at, is_active
             FROM Trade_In_Blacklist 
             WHERE customer_uuid = ? AND is_active = 1 
             AND (expires_at IS NULL OR expires_at > ?)"
        )
        .bind(customer_uuid.to_string())
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(Some(TradeInBlacklistEntry {
                blacklist_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("blacklist_uuid")
                        .map_err(|e| anyhow::anyhow!("Missing blacklist_uuid: {}", e))?,
                )
                .unwrap_or_default(),
                customer_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("customer_uuid")
                        .map_err(|e| anyhow::anyhow!("Missing customer_uuid: {}", e))?,
                )
                .unwrap_or_default(),
                reason: row.try_get("reason").unwrap_or_default(),
                added_by: row
                    .try_get::<String, _>("added_by")
                    .ok()
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                added_at: DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("added_at")
                        .map_err(|e| anyhow::anyhow!("Missing added_at: {}", e))?,
                )
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
                expires_at: row.try_get::<String, _>("expires_at").ok().and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                is_active: row.try_get("is_active").unwrap_or(true),
            }))
        } else {
            Ok(None)
        }
    }

    /// TASK-167: Log suspicious activity
    pub async fn log_suspicious_activity(
        &self,
        customer_uuid: Uuid,
        activity_type: SuspiciousActivityType,
        description: &str,
        severity: AlertSeverity,
    ) -> Result<SuspiciousActivity> {
        let alert = SuspiciousActivity {
            alert_uuid: Uuid::new_v4(),
            customer_uuid,
            activity_type: activity_type.clone(),
            description: description.to_string(),
            severity: severity.clone(),
            detected_at: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
        };

        sqlx::query(
            "INSERT INTO Suspicious_Activity (alert_uuid, customer_uuid, activity_type, description, severity, detected_at, acknowledged)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(alert.alert_uuid.to_string())
        .bind(alert.customer_uuid.to_string())
        .bind(format!("{:?}", activity_type))
        .bind(&alert.description)
        .bind(format!("{:?}", severity))
        .bind(alert.detected_at.to_rfc3339())
        .bind(alert.acknowledged)
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::warn!(
            "Suspicious activity detected for customer {}: {:?} - {}",
            customer_uuid,
            activity_type,
            description
        );

        Ok(alert)
    }

    /// TASK-167: Get unacknowledged alerts
    pub async fn get_pending_alerts(&self) -> Result<Vec<SuspiciousActivity>> {
        let rows = sqlx::query(
            "SELECT alert_uuid, customer_uuid, activity_type, description, severity, detected_at
             FROM Suspicious_Activity WHERE acknowledged = 0 ORDER BY detected_at DESC",
        )
        .fetch_all(&self.db.pool)
        .await
        .context("Database error")?;

        let mut alerts = Vec::new();
        for row in rows {
            use sqlx::Row;
            let activity_type_str: String = row.try_get("activity_type").unwrap_or_default();
            let severity_str: String = row.try_get("severity").unwrap_or_default();

            alerts.push(SuspiciousActivity {
                alert_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("alert_uuid")
                        .map_err(|e| anyhow::anyhow!("Missing alert_uuid: {}", e))?,
                )
                .unwrap_or_default(),
                customer_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("customer_uuid")
                        .map_err(|e| anyhow::anyhow!("Missing customer_uuid: {}", e))?,
                )
                .unwrap_or_default(),
                activity_type: match activity_type_str.as_str() {
                    "HighVelocity" => SuspiciousActivityType::HighVelocity,
                    "HighValue" => SuspiciousActivityType::HighValue,
                    "PatternMatch" => SuspiciousActivityType::PatternMatch,
                    "BlacklistedAssociate" => SuspiciousActivityType::BlacklistedAssociate,
                    "IdMismatch" => SuspiciousActivityType::IdMismatch,
                    _ => SuspiciousActivityType::PatternMatch,
                },
                description: row.try_get("description").unwrap_or_default(),
                severity: match severity_str.as_str() {
                    "Low" => AlertSeverity::Low,
                    "Medium" => AlertSeverity::Medium,
                    "High" => AlertSeverity::High,
                    "Critical" => AlertSeverity::Critical,
                    _ => AlertSeverity::Medium,
                },
                detected_at: DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("detected_at")
                        .map_err(|e| anyhow::anyhow!("Missing detected_at: {}", e))?,
                )
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
                acknowledged: false,
                acknowledged_by: None,
            });
        }

        Ok(alerts)
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_uuid: Uuid, acknowledged_by: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE Suspicious_Activity SET acknowledged = 1, acknowledged_by = ? WHERE alert_uuid = ?"
        )
        .bind(acknowledged_by.to_string())
        .bind(alert_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;
        Ok(())
    }
}
