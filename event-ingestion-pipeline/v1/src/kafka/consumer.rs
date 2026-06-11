use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use futures_util::StreamExt;
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::{Message, OwnedMessage},
    topic_partition_list::{Offset, TopicPartitionList},
    ClientConfig,
};
use tracing::{error, warn};

use crate::{
    config::Settings,
    domain::event::EventEnvelope,
    failure::dlq,
    kafka::producer::Publisher,
    observability::metrics,
    persistence::scylla::Persistence,
};

/// Main consumer loop: transforms Kafka input into durable storage or DLQ.
pub async fn run(
    settings: Settings,
    persistence: Persistence,
    dlq_publisher: Publisher,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &settings.kafka_consumer_group)
        .set("bootstrap.servers", &settings.kafka_brokers)
        .set("enable.auto.commit", "false") // prevent unsafe auto-ack
        .set("enable.auto.offset.store", "false")
        .create()
        .context("failed to create kafka consumer")?;

    consumer
        .subscribe(&[&settings.kafka_topic])
        .context("failed to subscribe to kafka topic")?;

    let mut stream = consumer.stream();

    let mut buffered_messages: Vec<OwnedMessage> =
        Vec::with_capacity(settings.batch_size);
    let mut buffered_events: Vec<EventEnvelope> =
        Vec::with_capacity(settings.batch_size);

    let flush_interval = Duration::from_millis(settings.flush_interval_ms);
    let mut last_flush = Instant::now();

    loop {
        tokio::select! {
            maybe_message = stream.next() => {
                match maybe_message {
                    Some(Ok(message)) => {
                        let owned = message.detach();

                        match decode_event(&owned) {
                            Ok(event) => {
                                metrics::inc_consumed(); // message entered processing pipeline

                                buffered_messages.push(owned);
                                buffered_events.push(event);
                            }
                            Err(err) => {
                                warn!(error = %err, "malformed → dlq");

                                // malformed must be preserved, never dropped
                                dlq::write_raw_failure(
                                    &dlq_publisher,
                                    &owned,
                                    &err.to_string(),
                                ).await?;

                                commit_single(&consumer, &message)?;
                            }
                        }

                        if buffered_events.len() >= settings.batch_size {
                            flush_batch(
                                &consumer,
                                &persistence,
                                &dlq_publisher,
                                &mut buffered_messages,
                                &mut buffered_events,
                                settings.max_retry_attempts,
                            ).await?;

                            last_flush = Instant::now();
                        }
                    }
                    Some(Err(err)) => {
                        warn!(error = %err, "consumer stream error");
                    }
                    None => break,
                }
            }

            _ = tokio::time::sleep(flush_interval),
            if !buffered_events.is_empty() && last_flush.elapsed() >= flush_interval => {
                flush_batch(
                    &consumer,
                    &persistence,
                    &dlq_publisher,
                    &mut buffered_messages,
                    &mut buffered_events,
                    settings.max_retry_attempts,
                ).await?;

                last_flush = Instant::now();
            }
        }
    }

    Ok(())
}

/// Flush batch enforces durability before offset commit.
async fn flush_batch(
    consumer: &StreamConsumer,
    persistence: &Persistence,
    dlq_publisher: &Publisher,
    buffered_messages: &mut Vec<OwnedMessage>,
    buffered_events: &mut Vec<EventEnvelope>,
    max_retry_attempts: usize,
) -> Result<()> {
    if buffered_events.is_empty() {
        return Ok(());
    }

    // move buffers to avoid cloning large batches
    let events = std::mem::take(buffered_events);
    let messages = std::mem::take(buffered_messages);

    let persist_result =
        persist_with_retry(persistence, &events, max_retry_attempts).await;

    match persist_result {
        Ok(()) => {
            metrics::inc_persisted(events.len() as u64); // durability achieved

            commit_batch(consumer, &messages)?;
        }
        Err(err) => {
            error!(error = %err, "persist failed → dlq");

            metrics::inc_persist_failure(); // indicates storage instability

            for (event, msg) in events.iter().zip(messages.iter()) {
                dlq::write_failure_with_provenance(
                    dlq_publisher,
                    event,
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    &err.to_string(),
                ).await?;
            }

            // commit after DLQ to avoid infinite reprocessing
            commit_batch(consumer, &messages)?;
        }
    }

    Ok(())
}

/// Retry persistence with bounded exponential backoff.
async fn persist_with_retry(
    persistence: &Persistence,
    events: &[EventEnvelope],
    max_retry_attempts: usize,
) -> Result<()> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match persistence.persist_batch(events).await {
            Ok(()) => return Ok(()),

            Err(err) if attempts < max_retry_attempts => {
                warn!(attempts, error = %err, "persist retry");

                metrics::inc_persist_failure();

                // bounded exponential backoff prevents retry storms
                let backoff = (100 * (1 << attempts)).min(1000);

                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }

            Err(err) => return Err(err),
        }
    }
}

/// Decode message into structured event; failure must be handled upstream.
fn decode_event(message: &OwnedMessage) -> Result<EventEnvelope> {
    let payload = message
        .payload_view::<str>()
        .transpose()
        .context("invalid utf-8")?
        .context("empty payload")?;

    serde_json::from_str(payload)
        .context("decode event envelope failed")
}

/// Commit single message after DLQ handling.
fn commit_single<M: Message>(
    consumer: &StreamConsumer,
    message: &M,
) -> Result<()> {
    let mut offsets = TopicPartitionList::new();

    offsets.add_partition_offset(
        message.topic(),
        message.partition(),
        Offset::Offset(message.offset() + 1),
    )?;

    consumer
        .commit(&offsets, CommitMode::Sync)
        .context("commit single failed")
}

/// Commit highest offset per partition to avoid duplication or gaps.
fn commit_batch(
    consumer: &StreamConsumer,
    messages: &[OwnedMessage],
) -> Result<()> {
    let mut offsets = TopicPartitionList::new();
    let mut max_offsets: HashMap<(String, i32), i64> = HashMap::new();

    for m in messages {
        let key = (m.topic().to_string(), m.partition());

        let entry = max_offsets.entry(key).or_insert(m.offset());
        if m.offset() > *entry {
            *entry = m.offset();
        }
    }

    for ((topic, partition), offset) in max_offsets {
        offsets.add_partition_offset(
            &topic,
            partition,
            Offset::Offset(offset + 1),
        )?;
    }

    consumer
        .commit(&offsets, CommitMode::Sync)
        .context("commit batch failed")
}