use serde::{Deserialize, Serialize};

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
