use crate::core::{Event, EventParticipant};
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::sync::SyncRepository;

#[derive(Clone)]
pub struct EventRepository {
    pool: SqlitePool,
    sync: SyncRepository,
}

impl EventRepository {
    pub fn new(pool: SqlitePool, sync: SyncRepository) -> Self {
        Self { pool, sync }
    }

    pub async fn insert(&self, event: &Event) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO Events (event_uuid, name, event_type, date, entry_fee, max_participants, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.event_uuid.to_string())
        .bind(&event.name)
        .bind(&event.event_type)
        .bind(event.date.to_rfc3339())
        .bind(event.entry_fee)
        .bind(event.max_participants)
        .bind(event.created_at.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &event.event_uuid.to_string(),
                "Event",
                "Update",
                &serde_json::to_value(event).unwrap_or_default(),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<Event>> {
        let rows = sqlx::query("SELECT event_uuid, name, event_type, date, entry_fee, max_participants, created_at FROM Events ORDER BY date DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_uuid_str: String = row.try_get("event_uuid").unwrap_or_default();
            let event_uuid = Uuid::parse_str(&event_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let event_type: String = row.try_get("event_type").unwrap_or_default();
            let date_str: String = row.try_get("date").unwrap_or_default();
            let date = chrono::DateTime::parse_from_rfc3339(&date_str)?.with_timezone(&chrono::Utc);
            let entry_fee: f64 = row.try_get("entry_fee").unwrap_or_default();
            let max_participants: Option<i32> = row.try_get("max_participants").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            events.push(Event {
                event_uuid,
                name,
                event_type,
                date,
                entry_fee,
                max_participants,
                created_at,
            });
        }
        Ok(events)
    }

    pub async fn get_by_id(&self, event_uuid: Uuid) -> Result<Option<Event>> {
        let row = sqlx::query("SELECT event_uuid, name, event_type, date, entry_fee, max_participants, created_at FROM Events WHERE event_uuid = ?")
            .bind(event_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let event_uuid_str: String = row.try_get("event_uuid").unwrap_or_default();
            let event_uuid = Uuid::parse_str(&event_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let event_type: String = row.try_get("event_type").unwrap_or_default();
            let date_str: String = row.try_get("date").unwrap_or_default();
            let date = chrono::DateTime::parse_from_rfc3339(&date_str)?.with_timezone(&chrono::Utc);
            let entry_fee: f64 = row.try_get("entry_fee").unwrap_or_default();
            let max_participants: Option<i32> = row.try_get("max_participants").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            Ok(Some(Event {
                event_uuid,
                name,
                event_type,
                date,
                entry_fee,
                max_participants,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn register_participant(&self, participant: &EventParticipant) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO Event_Participants (participant_uuid, event_uuid, customer_uuid, name, paid, placement, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(participant.participant_uuid.to_string())
        .bind(participant.event_uuid.to_string())
        .bind(participant.customer_uuid.map(|id| id.to_string()))
        .bind(&participant.name)
        .bind(participant.paid)
        .bind(participant.placement)
        .bind(participant.created_at.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &participant.participant_uuid.to_string(),
                "EventParticipant",
                "Update",
                &serde_json::to_value(participant).unwrap_or_default(),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_participants(&self, event_uuid: Uuid) -> Result<Vec<EventParticipant>> {
        let rows = sqlx::query("SELECT participant_uuid, event_uuid, customer_uuid, name, paid, placement, created_at FROM Event_Participants WHERE event_uuid = ?")
            .bind(event_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut participants = Vec::new();
        for row in rows {
            let participant_uuid_str: String = row.try_get("participant_uuid").unwrap_or_default();
            let participant_uuid = Uuid::parse_str(&participant_uuid_str).unwrap_or_default();
            // event_uuid is known
            let customer_uuid_str: Option<String> = row.try_get("customer_uuid").ok();
            let customer_uuid = customer_uuid_str.map(|s| Uuid::parse_str(&s).unwrap_or_default());
            let name: String = row.try_get("name").unwrap_or_default();
            let paid: bool = row.try_get("paid").unwrap_or_default();
            let placement: Option<i32> = row.try_get("placement").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            participants.push(EventParticipant {
                participant_uuid,
                event_uuid,
                customer_uuid,
                name,
                paid,
                placement,
                created_at,
            });
        }
        Ok(participants)
    }
}
