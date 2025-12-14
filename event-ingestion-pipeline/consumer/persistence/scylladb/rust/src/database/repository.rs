use anyhow::Result;
use scylla::{Session, Batch, BatchType};
use crate::model::ProcessedEvent;

/// Inserts a single event 
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

    session
        .execute(
            query,
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

/// Inserts a batch of events efficiently.
/// Automatically chunks the input vector for stability.
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

    const BATCH_SIZE: usize = 100;
    for chunk in events.chunks(BATCH_SIZE) {
        let mut batch = Batch::new(BatchType::Unlogged);

        for _ in chunk {
            batch.append_statement(query);
        }

        // flattening based on append-order
        let values: Vec<_> = chunk
            .iter()
            .flat_map(|evt| {
                vec![
                    evt.room_id.clone().into(),
                    evt.timestamp.into(),
                    evt.event_id.clone().into(),
                    evt.event_type.clone().into(),
                    evt.user_id.clone().into(),
                    evt.payload.clone().into(),
                    evt.metadata_json.clone().into(),
                    evt.server_timestamp.into(),
                ]
            })
            .collect();

        session.batch(&batch, values).await?;
    }

    Ok(())
}
