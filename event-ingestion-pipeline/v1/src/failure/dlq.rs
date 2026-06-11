use anyhow::Result;
use serde::Serialize;

use crate::{
    domain::event::EventEnvelope,
    kafka::producer::Publisher,
    observability::metrics,
};

/// DLQ record preserves everything required for replay and debugging.
#[derive(Debug, Serialize)]
pub struct DlqRecord {
    pub event: Option<EventEnvelope>, // present for valid events
    pub raw_payload: Option<Vec<u8>>, // present for malformed messages

    pub topic: String,
    pub partition: i32,
    pub offset: i64,

    pub reason: String,
    pub attempts: u8,

    pub failed_at_ms: i64,
}

/// Write structured failure after persistence exhaustion.
pub async fn write_failure_with_provenance(
    publisher: &Publisher,
    event: &EventEnvelope,
    topic: &str,
    partition: i32,
    offset: i64,
    reason: &str,
) -> Result<()> {
    let record = DlqRecord {
        event: Some(event.clone()),
        raw_payload: None,
        topic: topic.to_string(),
        partition,
        offset,
        reason: reason.to_string(),
        attempts: 1,
        failed_at_ms: chrono::Utc::now().timestamp_millis(),
    };

    publisher.publish_json(&record, &event.event_id).await?;

    metrics::inc_dlq(); // every DLQ write must be counted to prove no silent loss

    Ok(())
}

/// Write raw failure for malformed or undecodable messages.
pub async fn write_raw_failure(
    publisher: &Publisher,
    message: &rdkafka::message::OwnedMessage,
    reason: &str,
) -> Result<()> {
    let payload = message.payload().map(|p| p.to_vec());

    let record = DlqRecord {
        event: None,
        raw_payload: payload,
        topic: message.topic().to_string(),
        partition: message.partition(),
        offset: message.offset(),
        reason: reason.to_string(),
        attempts: 1,
        failed_at_ms: chrono::Utc::now().timestamp_millis(),
    };

    // synthetic key ensures uniqueness for replay
    let key = format!(
        "{}:{}:{}",
        message.topic(),
        message.partition(),
        message.offset()
    );

    publisher.publish_json(&record, &key).await?;

    metrics::inc_dlq(); // malformed messages still count toward failure budget

    Ok(())
}