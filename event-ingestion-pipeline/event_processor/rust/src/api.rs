use axum::{extract::State, Json};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::event::Event;
use crate::processor::process_event;
use crate::publisher::publish;

pub async fn ingest_event(
    State(source): State<Arc<Mutex<String>>>,
    Json(event): Json<Event>,
) -> Json<serde_json::Value> {
    let processed = process_event(event);
    let source = source.lock().await.clone();

    match publish(&source, &processed).await {
        Ok(_) => Json(serde_json::json!({ "ok": true, "processed": processed })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}
