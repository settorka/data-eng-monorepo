use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

/// Incoming event types (mirrors your .proto structure)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum Event {
    Chat(ChatEvent),
    Join(JoinEvent),
    Leave(LeaveEvent),
    Reaction(ReactionEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatEvent {
    pub user_id: String,
    pub room_id: String,
    pub journey_id: Option<String>,
    pub timestamp: i64,
    pub message: String,
    pub message_type: String,
    pub chat_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinEvent {
    pub user_id: String,
    pub room_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveEvent {
    pub user_id: String,
    pub room_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionEvent {
    pub user_id: String,
    pub room_id: String,
    pub message_id: String,
    pub emoji: String,
    pub timestamp: i64,
}

/// Processed event (what you send back to Redpanda)
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub event_type: String,
    pub user_id: String,
    pub room_id: String,
    pub journey_id: Option<String>,
    pub timestamp: i64,
    pub content: String,
    pub metadata_json: String,
    pub event_id: String,
    pub server_timestamp: i64,
}

impl ProcessedEvent {
    pub fn new(
        event_type: &str,
        user_id: String,
        room_id: String,
        journey_id: Option<String>,
        timestamp: i64,
        content: String,
        metadata_json: String,
    ) -> Self {
        let server_timestamp = Utc::now().timestamp_millis();
        Self {
            event_type: event_type.to_string(),
            user_id,
            room_id,
            journey_id,
            timestamp,
            content,
            metadata_json,
            event_id: Uuid::new_v4().to_string(),
            server_timestamp,
        }
    }
}
