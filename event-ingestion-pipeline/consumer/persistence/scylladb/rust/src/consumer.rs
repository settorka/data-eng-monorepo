use anyhow::Result;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use scylla::Session;
use serde_json;
use std::sync::Arc;

use crate::model::ProcessedEvent;
use crate::database::repository;

pub async fn run(session: Arc<Session>) -> Result<()> {
    // Configure Redpanda/Kafka consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "127.0.0.1:9092")
        .set("group.id", "scylla-persistence-consumer")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(&["chat_events"])?;
    tracing::info!("Subscribed to Redpanda topic: chat_events");

    loop {
        match consumer.recv().await {
            Err(e) => tracing::error!("Kafka error: {:?}", e),
            Ok(msg) => {
                if let Some(Ok(payload)) = msg.payload_view::<str>() {
                    match serde_json::from_str::<ProcessedEvent>(payload) {
                        Ok(event) => {
                            if let Err(e) = repository::insert_event(&session, &event).await {
                                tracing::error!(
                                    "Failed to persist event {:?}: {:?}",
                                    event.event_type,
                                    e
                                );
                            } else {
                                tracing::info!(
                                    "Persisted event {:?} for room {}",
                                    event.event_type,
                                    event.room_id
                                );
                            }
                        }
                        Err(e) => tracing::error!("Deserialization error: {:?}", e),
                    }
                }
                if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                    tracing::error!("Commit failed: {:?}", e);
                }
            }
        }
    }
}
