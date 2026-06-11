use std::sync::Arc;

use anyhow::{Context, Result};
use scylla::{query::Query, Session, SessionBuilder};

use crate::{config::Settings, event::EventEnvelope};

pub struct Persistence {
    session: Arc<Session>,
    insert_query: Query,
}

impl Clone for Persistence {
    fn clone(&self) -> Self {
        Self {
            session: Arc::clone(&self.session),
            insert_query: self.insert_query.clone(),
        }
    }
}

impl Persistence {
    pub async fn connect(settings: &Settings) -> Result<Self> {
        let session = SessionBuilder::new()
            .known_node(&settings.scylla_host)
            .build()
            .await
            .context("failed to connect to scylla")?;

        session
            .query(
                format!(
                    "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
                    settings.scylla_keyspace
                ),
                &[],
            )
            .await
            .context("failed to create keyspace")?;

        session
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {}.{} (
                        event_id text PRIMARY KEY,
                        event_type text,
                        schema_version text,
                        producer_timestamp bigint,
                        server_timestamp bigint,
                        user_id text,
                        room_id text,
                        payload text
                    )",
                    settings.scylla_keyspace, settings.scylla_table
                ),
                &[],
            )
            .await
            .context("failed to create events table")?;

        let insert_query = Query::new(format!(
            "INSERT INTO {}.{} (event_id, event_type, schema_version, producer_timestamp, server_timestamp, user_id, room_id, payload) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            settings.scylla_keyspace, settings.scylla_table
        ));

        Ok(Self {
            session: Arc::new(session),
            insert_query,
        })
    }

    pub async fn persist_batch(&self, events: &[EventEnvelope]) -> Result<()> {
        for event in events {
            let payload = serde_json::to_string(&event.payload).context("failed to serialize payload")?;
            self.session
                .query(
                    self.insert_query.clone(),
                    (
                        &event.event_id,
                        &event.event_type,
                        &event.schema_version,
                        event.producer_timestamp,
                        event.server_timestamp,
                        &event.user_id,
                        &event.room_id,
                        payload,
                    ),
                )
                .await
                .with_context(|| format!("failed to persist event {}", event.event_id))?;
        }
        Ok(())
    }
}
