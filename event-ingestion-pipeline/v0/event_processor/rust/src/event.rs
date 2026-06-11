use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

/// Incoming event types
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
    pub message: Option<String>,           
    pub message_type: Option<String>,      // "standard", "reply"
    pub chat_type: Option<String>,         // "single", "group", "global"
    pub message_id: Option<String>,        // if reply
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct JoinEvent {
    pub user_id: String,
    pub room_id: String,
    pub room_action: Option<String>,  // "open" or "join"
    pub timestamp: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveEvent {
    pub user_id: String,
    pub room_id: String,
    pub room_action: Option<String>,       // always "leave"
    pub timestamp: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionEvent {
    pub user_id: String,                   // actor
    pub target_user_id: Option<String>,    // message author
    pub room_id: String,
    pub message_id: String,                // target message
    pub emoji: String,
    pub timestamp: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


// Sent for consumption
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub event_type: String,
    pub user_id: String,
    pub room_id: String,
    pub journey_id: Option<String>,
    pub timestamp: i64,
    pub payload: String,           
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
        payload: String,
        metadata_json: String,
    ) -> Self {
        Self {
            event_type: event_type.to_string(),
            user_id,
            room_id,
            journey_id,
            timestamp,
            payload,
            metadata_json,
            event_id: Uuid::new_v4().to_string(),
            server_timestamp: Utc::now().timestamp_millis(),
        }
    }
}
