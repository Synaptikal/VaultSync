//! Notification Scheduler Service
//!
//! Handles scheduled/background notification tasks:
//! - Wants list match notifications
//! - Event reminders (email and SMS)
//! - Hold expiration reminders

use crate::database::Database;
use crate::services::notification::sms::SmsProvider;
use crate::services::notification::{EmailMessage, EmailProvider};
use anyhow::Result;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct NotificationScheduler {
    db: Arc<Database>,
    email_provider: Arc<Box<dyn EmailProvider>>,
    sms_provider: Arc<Box<dyn SmsProvider>>,
}

impl NotificationScheduler {
    pub fn new(
        db: Arc<Database>,
        email_provider: Arc<Box<dyn EmailProvider>>,
        sms_provider: Arc<Box<dyn SmsProvider>>,
    ) -> Self {
        Self {
            db,
            email_provider,
            sms_provider,
        }
    }

    /// TASK-194: Check for wants list matches against new inventory
    /// Call this when new inventory is added
    pub async fn check_wants_list_matches(&self, product_uuid: Uuid) -> Result<Vec<Uuid>> {
        tracing::info!("Checking wants list matches for product {}", product_uuid);

        // Find all wants list items that match this product
        let matching_wants = sqlx::query(
            r#"
            SELECT wl.customer_uuid, wli.wants_item_uuid, wli.product_uuid
            FROM Wants_List_Items wli
            JOIN Wants_Lists wl ON wli.wants_list_uuid = wl.wants_list_uuid
            WHERE wli.product_uuid = ? AND wli.fulfilled = 0
            "#,
        )
        .bind(product_uuid.to_string())
        .fetch_all(&self.db.pool)
        .await?;

        let mut notified_customers = Vec::new();

        for row in matching_wants {
            let customer_uuid_str: String =
                sqlx::Row::try_get(&row, "customer_uuid").unwrap_or_default();
            let customer_uuid = match Uuid::parse_str(&customer_uuid_str) {
                Ok(u) => u,
                Err(_) => continue,
            };

            // Get customer details
            if let Ok(Some(customer)) = self.db.customers.get_by_id(customer_uuid).await {
                // Get product name
                let product_name = match self.db.products.get_by_id(product_uuid).await {
                    Ok(Some(p)) => p.name,
                    _ => "an item on your wants list".to_string(),
                };

                // Send email if available
                if let Some(email) = &customer.email {
                    if !email.is_empty() {
                        let msg = EmailMessage {
                            to: email.clone(),
                            subject: format!("🎉 {} is now available!", product_name),
                            body: format!(
                                r#"<!DOCTYPE html>
<html>
<body style="font-family: Arial, sans-serif;">
    <h2>Great news, {}!</h2>
    <p>An item from your wants list is now available:</p>
    <div style="background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0;">
        <strong>{}</strong>
    </div>
    <p>Visit us soon to grab it before it's gone!</p>
    <br>
    <p>- The VaultSync Team</p>
</body>
</html>"#,
                                customer.name, product_name
                            ),
                            attachment_path: None,
                        };

                        if let Err(e) = self.email_provider.send_email(&msg).await {
                            tracing::warn!(
                                "Failed to send wants list email to {}: {}",
                                customer.name,
                                e
                            );
                        } else {
                            notified_customers.push(customer_uuid);
                        }
                    }
                }

                // Also send SMS if available
                if let Some(phone) = &customer.phone {
                    if !phone.is_empty() {
                        let sms_body = format!(
                            "VaultSync: {} is now available! Come grab it before it's gone.",
                            product_name
                        );
                        if let Err(e) = self.sms_provider.send_sms(phone, &sms_body).await {
                            tracing::warn!(
                                "Failed to send wants list SMS to {}: {}",
                                customer.name,
                                e
                            );
                        }
                    }
                }
            }
        }

        if !notified_customers.is_empty() {
            tracing::info!(
                "Notified {} customers about wants list match for product {}",
                notified_customers.len(),
                product_uuid
            );
        }

        Ok(notified_customers)
    }

    /// TASK-195 & TASK-200: Send event reminders 24 hours before
    pub async fn send_event_reminders(&self) -> Result<i32> {
        let now = Utc::now();
        let reminder_window_start = now + Duration::hours(23);
        let reminder_window_end = now + Duration::hours(25);

        tracing::info!("Checking for events needing reminders...");

        // Find events happening in ~24 hours that haven't had reminders sent
        let upcoming_events = sqlx::query(
            r#"
            SELECT event_uuid, name, date, entry_fee
            FROM Events
            WHERE date BETWEEN ? AND ?
            AND reminder_sent = 0
            "#,
        )
        .bind(reminder_window_start.to_rfc3339())
        .bind(reminder_window_end.to_rfc3339())
        .fetch_all(&self.db.pool)
        .await;

        // If query fails (table doesn't have reminder_sent column), try without it
        let upcoming_events = match upcoming_events {
            Ok(rows) => rows,
            Err(_) => {
                // Fallback query without reminder_sent
                sqlx::query(
                    r#"
                    SELECT event_uuid, name, date, entry_fee
                    FROM Events
                    WHERE date BETWEEN ? AND ?
                    "#,
                )
                .bind(reminder_window_start.to_rfc3339())
                .bind(reminder_window_end.to_rfc3339())
                .fetch_all(&self.db.pool)
                .await?
            }
        };

        let mut total_sent = 0;

        for event_row in upcoming_events {
            let event_uuid_str: String =
                sqlx::Row::try_get(&event_row, "event_uuid").unwrap_or_default();
            let event_uuid = match Uuid::parse_str(&event_uuid_str) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let event_name: String = sqlx::Row::try_get(&event_row, "name").unwrap_or_default();
            let event_date: String = sqlx::Row::try_get(&event_row, "date").unwrap_or_default();

            // Get all participants for this event
            let participants = sqlx::query(
                r#"
                SELECT ep.customer_uuid, ep.name as participant_name
                FROM Event_Participants ep
                WHERE ep.event_uuid = ?
                "#,
            )
            .bind(event_uuid.to_string())
            .fetch_all(&self.db.pool)
            .await?;

            for participant in participants {
                let customer_uuid_str: String =
                    sqlx::Row::try_get(&participant, "customer_uuid").unwrap_or_default();
                let participant_name: String =
                    sqlx::Row::try_get(&participant, "participant_name").unwrap_or_default();

                // Try to get customer contact info
                if let Ok(customer_uuid) = Uuid::parse_str(&customer_uuid_str) {
                    if let Ok(Some(customer)) = self.db.customers.get_by_id(customer_uuid).await {
                        // Send email reminder
                        if let Some(email) = &customer.email {
                            if !email.is_empty() {
                                let msg = EmailMessage {
                                    to: email.clone(),
                                    subject: format!("⏰ Reminder: {} is tomorrow!", event_name),
                                    body: format!(
                                        r#"<!DOCTYPE html>
<html>
<body style="font-family: Arial, sans-serif;">
    <h2>Event Reminder</h2>
    <p>Hi {},</p>
    <p>This is a friendly reminder that <strong>{}</strong> is happening tomorrow!</p>
    <p><strong>Date:</strong> {}</p>
    <p>We look forward to seeing you there!</p>
    <br>
    <p>- The VaultSync Team</p>
</body>
</html>"#,
                                        participant_name, event_name, event_date
                                    ),
                                    attachment_path: None,
                                };

                                if self.email_provider.send_email(&msg).await.is_ok() {
                                    total_sent += 1;
                                }
                            }
                        }

                        // Send SMS reminder
                        if let Some(phone) = &customer.phone {
                            if !phone.is_empty() {
                                let sms_body = format!(
                                    "Reminder: {} is tomorrow! See you there. - VaultSync",
                                    event_name
                                );
                                let _ = self.sms_provider.send_sms(phone, &sms_body).await;
                            }
                        }
                    }
                }
            }

            // Mark reminder as sent (if column exists)
            let _ = sqlx::query("UPDATE Events SET reminder_sent = 1 WHERE event_uuid = ?")
                .bind(event_uuid.to_string())
                .execute(&self.db.pool)
                .await;
        }

        if total_sent > 0 {
            tracing::info!("Sent {} event reminder notifications", total_sent);
        }

        Ok(total_sent)
    }

    /// TASK-199: Send hold expiration reminders (X days before expiration)
    pub async fn send_hold_expiration_reminders(&self, days_before: i64) -> Result<i32> {
        let now = Utc::now();
        let warning_date = now + Duration::days(days_before);
        let warning_date_end = warning_date + Duration::hours(24);

        tracing::info!("Checking for holds expiring in {} days...", days_before);

        // Find active holds expiring soon
        let expiring_holds = sqlx::query(
            r#"
            SELECT h.hold_uuid, h.customer_uuid, h.balance_due, h.expiration_date
            FROM Holds h
            WHERE h.status = 'Active'
            AND h.expiration_date BETWEEN ? AND ?
            "#,
        )
        .bind(warning_date.to_rfc3339())
        .bind(warning_date_end.to_rfc3339())
        .fetch_all(&self.db.pool)
        .await?;

        let mut total_sent = 0;

        for hold_row in expiring_holds {
            let customer_uuid_str: String =
                sqlx::Row::try_get(&hold_row, "customer_uuid").unwrap_or_default();
            let balance_due: f64 = sqlx::Row::try_get(&hold_row, "balance_due").unwrap_or(0.0);
            let expiration_date: String =
                sqlx::Row::try_get(&hold_row, "expiration_date").unwrap_or_default();

            if let Ok(customer_uuid) = Uuid::parse_str(&customer_uuid_str) {
                if let Ok(Some(customer)) = self.db.customers.get_by_id(customer_uuid).await {
                    // Send SMS reminder (primary for hold reminders)
                    if let Some(phone) = &customer.phone {
                        if !phone.is_empty() {
                            let sms_body = format!(
                                "VaultSync: Your layaway expires on {}. Balance due: ${:.2}. Visit us to make a payment!",
                                expiration_date.split('T').next().unwrap_or(&expiration_date),
                                balance_due
                            );

                            if self.sms_provider.send_sms(phone, &sms_body).await.is_ok() {
                                total_sent += 1;
                            }
                        }
                    }

                    // Also send email
                    if let Some(email) = &customer.email {
                        if !email.is_empty() {
                            let msg = EmailMessage {
                                to: email.clone(),
                                subject: "⚠️ Your layaway is expiring soon".to_string(),
                                body: format!(
                                    r#"<!DOCTYPE html>
<html>
<body style="font-family: Arial, sans-serif;">
    <h2>Layaway Expiration Notice</h2>
    <p>Hi {},</p>
    <p>This is a reminder that your layaway will expire on <strong>{}</strong>.</p>
    <p><strong>Balance Due:</strong> ${:.2}</p>
    <p>Please visit us to make a payment and avoid losing your items.</p>
    <br>
    <p>- The VaultSync Team</p>
</body>
</html>"#,
                                    customer.name,
                                    expiration_date
                                        .split('T')
                                        .next()
                                        .unwrap_or(&expiration_date),
                                    balance_due
                                ),
                                attachment_path: None,
                            };

                            let _ = self.email_provider.send_email(&msg).await;
                        }
                    }
                }
            }
        }

        if total_sent > 0 {
            tracing::info!("Sent {} hold expiration reminder notifications", total_sent);
        }

        Ok(total_sent)
    }

    /// Run all scheduled notification checks
    /// Call this from a background task/timer
    pub async fn run_scheduled_tasks(&self) -> Result<()> {
        tracing::info!("Running scheduled notification tasks...");

        // Event reminders (24h before)
        if let Err(e) = self.send_event_reminders().await {
            tracing::error!("Failed to send event reminders: {}", e);
        }

        // Hold expiration reminders (3 days before)
        if let Err(e) = self.send_hold_expiration_reminders(3).await {
            tracing::error!("Failed to send hold expiration reminders: {}", e);
        }

        // Hold expiration reminders (1 day before - final warning)
        if let Err(e) = self.send_hold_expiration_reminders(1).await {
            tracing::error!("Failed to send final hold expiration reminders: {}", e);
        }

        Ok(())
    }
}
