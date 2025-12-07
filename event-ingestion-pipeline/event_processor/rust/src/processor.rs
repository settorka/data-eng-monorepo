use crate::event::{Event, ChatEvent, JoinEvent, LeaveEvent, ReactionEvent, ProcessedEvent};
use serde_json::json;

/// Transforms incoming Event and converts it into a ProcessedEvent
pub fn process_event(event: Event) -> ProcessedEvent {
    match event {
        Event::Chat(e) => {
            let metadata = json!({
                "message_type": e.message_type,
                "chat_type": e.chat_type
            }).to_string();
            ProcessedEvent::new(
                "chat",
                e.user_id,
                e.room_id,
                e.journey_id,
                e.timestamp,
                e.message,
                metadata,
            )
        }

        Event::Join(e) => {
            ProcessedEvent::new(
                "join",
                e.user_id,
                e.room_id,
                None,
                e.timestamp,
                "".into(),
                "{}".into(),
            )
        }

        Event::Leave(e) => {
            ProcessedEvent::new(
                "leave",
                e.user_id,
                e.room_id,
                None,
                e.timestamp,
                "".into(),
                "{}".into(),
            )
        }

        Event::Reaction(e) => {
            let metadata = json!({
                "message_id": e.message_id
            }).to_string();
            ProcessedEvent::new(
                "reaction",
                e.user_id,
                e.room_id,
                None,
                e.timestamp,
                e.emoji,
                metadata,
            )
        }
    }
}
