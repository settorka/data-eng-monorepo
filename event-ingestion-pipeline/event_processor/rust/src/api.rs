use axum::{extract::State, Json};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::event::Event;
use crate::processor::process_event;
use crate::publisher::publish;
use tracing::{info, error, instrument};

#[instrument(skip(source, event))]
pub async fn ingest_event(
    State(source): State<Arc<Mutex<String>>>,
    Json(event): Json<Event>,
) -> Json<serde_json::Value> {

    info!(?event, "Received raw event");

    let processed = process_event(event);

    info!(?processed, "Transformed event");

    let source = source.lock().await.clone();
    info!(publish_source=%source, "Publishing event");

    match publish(&source, &processed).await {
        Ok(_) => {
            info!(event_id=%processed.event_id, "Event delivered");
            Json(serde_json::json!({
                "ok": true,
                "status": "delivered",
                "event_id": processed.event_id,
                "topic": "chat_events"
            }))
        }
        Err(e) => {
            error!(error=?e, "Event publish failed");
            Json(serde_json::json!({
                "ok": false,
                "error": e.to_string()
            }))
        }
    }
}
