use axum::Json;
use crate::event::Event;
use super::redpanda::produce_to_redpanda;

pub async fn ingest_event(Json(event): Json<Event>) -> &'static str{
    match produce_to_redpanda(event).await {
        Ok(_) => "ok",
        Err(_) => "error"
    }
}