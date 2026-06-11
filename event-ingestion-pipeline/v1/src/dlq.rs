use anyhow::Result;
use serde::Serialize;

use crate::{event::EventEnvelope, publisher::Publisher};

#[derive(Debug, Serialize)]
pub struct DlqRecord {
    pub event: EventEnvelope,
    pub reason: String,
    pub failed_at_ms: i64,
}

pub async fn write_failure(
    publisher: &Publisher,
    event: &EventEnvelope,
    reason: &str,
) -> Result<()> {
    let record = DlqRecord {
        event: event.clone(),
        reason: reason.to_string(),
        failed_at_ms: chrono::Utc::now().timestamp_millis(),
    };
    publisher.publish_json(&record, &event.event_id).await
}
