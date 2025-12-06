use crate::event as http;
use super::elixir_client::chat;

impl TryFrom<http::Event> for chat::Event {
    type Error = ();

    fn try_from(value: http::Event) -> Result<Self, Self::Error> {
        match value {
            http::Event::Chat(e) => Ok(chat::Event {
                kind: Some(chat::event::Kind::Chat(chat::ChatEvent {
                    user_id: e.user_id,
                    room_id: e.room_id,
                    journey_id: e.journey_id.unwrap_or_default(),
                    timestamp: e.timestamp,
                    message: e.message,
                    message_type: e.message_type,
                    chat_type: e.chat_type,
                })),
            }),

            http::Event::Join(e) => Ok(chat::Event {
                kind: Some(chat::event::Kind::Join(chat::JoinEvent {
                    user_id: e.user_id,
                    room_id: e.room_id,
                    timestamp: e.timestamp,
                })),
            }),

            http::Event::Leave(e) => Ok(chat::Event {
                kind: Some(chat::event::Kind::Leave(chat::LeaveEvent {
                    user_id: e.user_id,
                    room_id: e.room_id,
                    timestamp: e.timestamp,
                })),
            }),

            http::Event::Reaction(e) => Ok(chat::Event {
                kind: Some(chat::event::Kind::Reaction(chat::ReactionEvent {
                    user_id: e.user_id,
                    room_id: e.room_id,
                    message_id: e.message_id,
                    emoji: e.emoji,
                    timestamp: e.timestamp,
                })),
            }),
        }
    }
}
