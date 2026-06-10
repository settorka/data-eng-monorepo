use anyhow::Result;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use scylla::Session;
use serde_json;
use std::sync::Arc;
use tokio::{sync::mpsc, time::{self, Duration}};
use crate::{model::ProcessedEvent, database::repository};

pub async fn run(session: Arc<Session>) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "redpanda:9092")
        .set("group.id", "scylla-persistence-async")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(&["chat_events"])?;
    tracing::info!("Subscribed to Redpanda topic: chat_events");

    // bounded channel to prevent unbounded memory growth (adapted from go channels)
    let (tx, mut rx) = mpsc::channel::<ProcessedEvent>(10_000);

    // Spawns background writer
    let writer_session = session.clone();
    tokio::spawn(async move {
        let mut buffer: Vec<ProcessedEvent> = Vec::new();
        let mut flush_timer = time::interval(Duration::from_millis(2000));
        const BATCH_SIZE: usize = 500;

        loop {
            tokio::select! {
                _ = flush_timer.tick() => {
                    if !buffer.is_empty() {
                        if let Err(e) = repository::insert_batch(&writer_session, &buffer).await {
                            tracing::error!("Batch insert failed: {:?}", e);
                        } else {
                            tracing::info!("Flushed {} events to Scylla", buffer.len());
                        }
                        buffer.clear();
                    }
                }
                Some(event) = rx.recv() => {
                    buffer.push(event);
                    if buffer.len() >= BATCH_SIZE {
                        if let Err(e) = repository::insert_batch(&writer_session, &buffer).await {
                            tracing::error!("Batch insert failed: {:?}", e);
                        } else {
                            tracing::info!("Flushed {} events to Scylla", buffer.len());
                        }
                        buffer.clear();
                    }
                }
            }
        }
    });

    // parse and send to channel
    loop {
        match consumer.recv().await {
            Err(e) => tracing::error!("Kafka error: {:?}", e),
            Ok(msg) => {
                if let Some(Ok(payload)) = msg.payload_view::<str>() {
                    match serde_json::from_str::<ProcessedEvent>(payload) {
                        Ok(event) => {
                            if let Err(_) = tx.try_send(event) {
                                tracing::warn!("Channel full, dropping event temporarily");
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
