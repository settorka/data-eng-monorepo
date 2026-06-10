use serde::{Deserialize, Serialize};

/// Unified event format as emitted by the ingress service.
/// Each message should be from Redpanda, 
/// it's intercepted as ProcessedEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub event_type: String,
    pub user_id: String,
    pub room_id: String,
    pub journey_id: Option<String>,
    pub timestamp: i64,
    pub payload: String,           // JSON string of event body
    pub metadata_json: String,     // metadata as JSON
    pub event_id: String,          // unique id for idempotency
    pub server_timestamp: i64,     // when processed upstream
}
