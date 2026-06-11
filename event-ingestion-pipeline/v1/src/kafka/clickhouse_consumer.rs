use anyhow::Result;
use futures_util::StreamExt;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::Message,
    ClientConfig,
};
use tracing::error;

use clickhouse::Client;
use serde::Serialize;

use crate::{config::Settings, domain::event::EventEnvelope};

/// Flat row for ClickHouse (must match table schema)
#[derive(Debug, Serialize, clickhouse::Row)]
struct ClickhouseEvent {
    event_id: String,
    event_type: String,
    user_id: String,
    room_id: String,
    payload: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Analytics consumer (non-critical path)
pub async fn run(settings: Settings) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "clickhouse-consumer")
        .set("bootstrap.servers", &settings.kafka_brokers)
        .create()?;

    consumer.subscribe(&[&settings.kafka_topic])?;

    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("events");

    let mut stream = consumer.stream();

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    match serde_json::from_slice::<EventEnvelope>(payload) {
                        Ok(event) => {
                            let row = map_event(event);

                            if let Err(err) = insert_event(&client, &row).await {
                                error!(error = %err, "clickhouse insert failed");
                            }
                        }
                        Err(err) => {
                            error!(error = %err, "decode failed (analytics path)");
                        }
                    }
                }
            }
            Err(err) => {
                error!(error = %err, "consumer error");
            }
        }
    }

    Ok(())
}

/// Converts the ingestion model to analytics model
fn map_event(event: EventEnvelope) -> ClickhouseEvent {
    ClickhouseEvent {
        event_id: event.event_id,
        event_type: event.event_type,
        user_id: event.user_id,
        room_id: event.room_id,
        payload: event.payload.to_string(),
        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
            event.server_timestamp
        ).unwrap_or_else(|| chrono::Utc::now()),
    }
}

/// Insert row into ClickHouse
async fn insert_event(client: &Client, row: &ClickhouseEvent) -> Result<()> {
    let mut insert = client
        .insert::<ClickhouseEvent>("events")
        .await?;

    insert.write(row).await?;
    insert.end().await?;

    Ok(())
}