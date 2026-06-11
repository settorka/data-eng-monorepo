use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: String,
    pub producer_timestamp: i64,
    pub server_timestamp: i64,
    pub user_id: String,
    pub room_id: String,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    pub fn new(
        event_type: String,
        user_id: String,
        room_id: String,
        payload: serde_json::Value,
        producer_timestamp: Option<i64>,
    ) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            schema_version: "v1".to_string(),
            producer_timestamp: producer_timestamp.unwrap_or(now),
            server_timestamp: now,
            user_id,
            room_id,
            payload,
        }
    }
}
