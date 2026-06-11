use std::sync::Arc;

use anyhow::{Context, Result};
use scylla::{query::Query, Session, SessionBuilder};

use crate::{config::Settings, domain::event::EventEnvelope};

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

        let insert_query = Query::new(format!(
            "INSERT INTO {}.{} (event_id, event_type, user_id, room_id, payload, producer_timestamp, server_timestamp, schema_version) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            settings.scylla_keyspace, settings.scylla_table
        ));

        Ok(Self {
            session: Arc::new(session),
            insert_query,
        })
    }

    pub async fn persist_batch(&self, events: &[EventEnvelope]) -> Result<()> {
        for event in events {
            let payload =
                serde_json::to_string(&event.payload).context("failed to serialize payload")?;
            let schema_version = parse_schema_version(&event.schema_version)
                .with_context(|| format!("failed to parse schema version for event {}", event.event_id))?;
            self.session
                .query(
                    self.insert_query.clone(),
                    (
                        &event.event_id,
                        &event.event_type,
                        &event.user_id,
                        &event.room_id,
                        payload,
                        event.producer_timestamp,
                        event.server_timestamp,
                        schema_version,
                    ),
                )
                .await
                .with_context(|| format!("failed to persist event {}", event.event_id))?;
        }
        Ok(())
    }
}

fn parse_schema_version(value: &str) -> Result<i32> {
    let trimmed = value.trim();
    let numeric = trimmed.strip_prefix('v').unwrap_or(trimmed);
    numeric
        .parse::<i32>()
        .context("schema_version must be numeric or prefixed with 'v'")
}
