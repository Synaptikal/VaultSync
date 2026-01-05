use crate::core::{Event, EventParticipant};
use crate::database::Database;
use crate::errors::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct EventService {
    db: Arc<Database>,
}

impl EventService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_event(
        &self,
        name: String,
        event_type: String,
        date: DateTime<Utc>,
        entry_fee: f64,
        max_participants: Option<i32>,
    ) -> Result<Event> {
        let event = Event {
            event_uuid: Uuid::new_v4(),
            name,
            event_type,
            date,
            entry_fee,
            max_participants,
            created_at: Utc::now(),
        };

        self.db.events.insert(&event).await?;
        Ok(event)
    }

    pub async fn get_upcoming_events(&self) -> Result<Vec<Event>> {
        let events = self.db.events.get_all().await?;
        // Filter for upcoming (or all, sorted DESC)
        // For now, returning all as per DB method
        Ok(events)
    }

    /// HIGH-006 FIX: Register a player with proper validation
    /// - Validates event exists
    /// - Enforces max_participants limit
    /// - Handles store credit payment
    pub async fn register_player(
        &self,
        event_uuid: Uuid,
        player_name: String,
        customer_uuid: Option<Uuid>,
        pay_with_credit: bool,
    ) -> Result<EventParticipant> {
        // 1. Validate event exists and get details
        let event = self
            .db
            .events
            .get_by_id(event_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Event {} not found", event_uuid))?;

        // 2. Check max participants limit
        if let Some(max) = event.max_participants {
            let current_participants = self.db.events.get_participants(event_uuid).await?;
            let current_count = current_participants.len() as i32;

            if current_count >= max {
                return Err(anyhow::anyhow!(
                    "Event '{}' is full ({}/{} participants)",
                    event.name,
                    current_count,
                    max
                ));
            }

            tracing::debug!(
                "Event '{}' has capacity: {}/{} participants",
                event.name,
                current_count,
                max
            );
        }

        // 3. Handle payment
        let mut paid = false;

        if pay_with_credit {
            if let Some(c_uuid) = customer_uuid {
                if event.entry_fee > 0.0 {
                    // Check customer has sufficient credit
                    let customer = self
                        .db
                        .customers
                        .get_by_id(c_uuid)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Customer {} not found", c_uuid))?;

                    if customer.store_credit < event.entry_fee {
                        return Err(anyhow::anyhow!(
                            "Insufficient store credit: ${:.2} available, ${:.2} required",
                            customer.store_credit,
                            event.entry_fee
                        ));
                    }

                    self.db
                        .customers
                        .update_store_credit(c_uuid, -event.entry_fee)
                        .await?;
                    paid = true;
                    tracing::info!(
                        "Deducted ${:.2} store credit from customer {} for event '{}'",
                        event.entry_fee,
                        c_uuid,
                        event.name
                    );
                } else {
                    paid = true; // Free event
                }
            }
        }
        // If not pay_with_credit, paid remains false (cash payment to be collected manually)

        // 4. Create and save participant
        let participant = EventParticipant {
            participant_uuid: Uuid::new_v4(),
            event_uuid,
            customer_uuid,
            name: player_name.clone(),
            paid,
            placement: None,
            created_at: Utc::now(),
        };

        self.db.events.register_participant(&participant).await?;

        tracing::info!(
            "Registered '{}' for event '{}' (paid: {})",
            player_name,
            event.name,
            paid
        );

        Ok(participant)
    }

    pub async fn record_placement(&self, _participant_uuid: Uuid, _placement: i32) -> Result<()> {
        // Update placement logic would go here.
        // Need `update_participant_placement` in DB.
        // Skipping for this MVP phase.
        Ok(())
    }
}
