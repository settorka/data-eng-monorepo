use std::time::{Duration, Instant};

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
    config::Settings, domain::event::EventEnvelope, failure::dlq, kafka::producer::Publisher,
    persistence::scylla::Persistence,
};

pub async fn run(
    settings: Settings,
    persistence: Persistence,
    dlq_publisher: Publisher,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &settings.kafka_consumer_group)
        .set("bootstrap.servers", &settings.kafka_brokers)
        .set("enable.auto.commit", "false")
        .set("enable.auto.offset.store", "false")
        .create()
        .context("failed to create kafka consumer")?;

    consumer
        .subscribe(&[&settings.kafka_topic])
        .context("failed to subscribe to kafka topic")?;

    let mut stream = consumer.stream();
    let mut buffered_messages: Vec<OwnedMessage> = Vec::with_capacity(settings.batch_size);
    let mut buffered_events: Vec<EventEnvelope> = Vec::with_capacity(settings.batch_size);
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
                                buffered_messages.push(owned);
                                buffered_events.push(event);
                            }
                            Err(err) => {
                                warn!(error = %err, "dropping malformed message to dlq");
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
            _ = tokio::time::sleep(flush_interval), if !buffered_events.is_empty() && last_flush.elapsed() >= flush_interval => {
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

    let events = buffered_events.clone();
    let persist_result = persist_with_retry(persistence, &events, max_retry_attempts).await;

    match persist_result {
        Ok(()) => {
            commit_batch(consumer, buffered_messages)?;
        }
        Err(err) => {
            error!(error = %err, "batch persistence failed; sending to dlq");
            for event in events.iter() {
                dlq::write_failure(dlq_publisher, event, &err.to_string()).await?;
            }
            commit_batch(consumer, buffered_messages)?;
        }
    }

    buffered_messages.clear();
    buffered_events.clear();
    Ok(())
}

async fn persist_with_retry(
    persistence: &Persistence,
    events: &[EventEnvelope],
    max_retry_attempts: usize,
) -> Result<()> {
    let mut attempts = 0usize;
    loop {
        attempts += 1;
        match persistence.persist_batch(events).await {
            Ok(()) => return Ok(()),
            Err(err) if attempts < max_retry_attempts => {
                warn!(attempts, error = %err, "persist failed, retrying");
                tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
            }
            Err(err) => return Err(err),
        }
    }
}

fn decode_event(message: &OwnedMessage) -> Result<EventEnvelope> {
    let payload = message
        .payload_view::<str>()
        .transpose()
        .context("payload was not valid utf-8")?
        .context("message payload was empty")?;
    serde_json::from_str(payload).context("failed to decode event envelope")
}

fn commit_single<M: Message>(consumer: &StreamConsumer, message: &M) -> Result<()> {
    let mut offsets = TopicPartitionList::new();
    offsets
        .add_partition_offset(
            message.topic(),
            message.partition(),
            Offset::Offset(message.offset() + 1),
        )
        .context("failed to build single-message offset commit")?;
    consumer
        .commit(&offsets, CommitMode::Sync)
        .context("failed to commit single offset")
}

fn commit_batch(consumer: &StreamConsumer, messages: &[OwnedMessage]) -> Result<()> {
    let mut offsets = TopicPartitionList::new();
    for message in messages {
        offsets
            .add_partition_offset(
                message.topic(),
                message.partition(),
                Offset::Offset(message.offset() + 1),
            )
            .with_context(|| {
                format!(
                    "failed to build offset commit for topic {} partition {}",
                    message.topic(),
                    message.partition()
                )
            })?;
    }
    consumer
        .commit(&offsets, CommitMode::Sync)
        .context("failed to commit batch offsets")
}
