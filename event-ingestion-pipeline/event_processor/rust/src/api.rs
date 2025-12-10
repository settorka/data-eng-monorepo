use axum::{extract::State, Json};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::event::Event;
use crate::processor::process_event;
use crate::publisher::publish;
use tracing::error;

pub async fn ingest_event(
    State(source): State<Arc<Mutex<String>>>,
    Json(event): Json<Event>,
) -> Json<serde_json::Value> {
    let processed = process_event(event);
    let source = source.lock().await.clone();

    match publish(&source, &processed).await {
        Ok(_) => Json(serde_json::json!({
            "ok": true,
            "status": "delivered",
            "event_id": processed.event_id,
            "topic": "chat_events"
        })),
        Err(e) => {
            error!("Publish failed: {:?}", e);
            Json(serde_json::json!({
                "ok": false,
                "error": e.to_string()
            }))
        }
    }
}
