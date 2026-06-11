use anyhow::Result;
use scylla::{Session, prepared_statement::PreparedStatement};
use scylla::batch::{Batch, BatchType};
use crate::model::ProcessedEvent;

/// Insert a single processed event
pub async fn insert_event(session: &Session, evt: &ProcessedEvent) -> Result<()> {
    let query = r#"
        INSERT INTO messages (
            room_id,
            timestamp,
            event_id,
            event_type,
            user_id,
            payload,
            metadata_json,
            server_timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    let prepared: PreparedStatement = session.prepare(query).await?;

    session
        .execute(
            &prepared,
            (
                &evt.room_id,
                evt.timestamp,
                &evt.event_id,
                &evt.event_type,
                &evt.user_id,
                &evt.payload,
                &evt.metadata_json,
                evt.server_timestamp,
            ),
        )
        .await?;

    Ok(())
}

/// Insert events in unlogged batches for throughput
pub async fn insert_batch(session: &Session, events: &[ProcessedEvent]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }

    let query = r#"
        INSERT INTO messages (
            room_id,
            timestamp,
            event_id,
            event_type,
            user_id,
            payload,
            metadata_json,
            server_timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    let prepared: PreparedStatement = session.prepare(query).await?;

    const BATCH_SIZE: usize = 100;

    for chunk in events.chunks(BATCH_SIZE) {
        let mut batch = Batch::new(BatchType::Unlogged);

        for _ in chunk {
            batch.append_statement(prepared.clone());
        }

        let values: Vec<_> = chunk
            .iter()
            .map(|evt| {
                (
                    evt.room_id.as_str(),
                    evt.timestamp,
                    evt.event_id.as_str(),
                    evt.event_type.as_str(),
                    evt.user_id.as_str(),
                    evt.payload.as_str(),
                    evt.metadata_json.as_str(),
                    evt.server_timestamp,
                )
            })
            .collect();

        session.batch(&batch, values).await?;
    }

    Ok(())
}
