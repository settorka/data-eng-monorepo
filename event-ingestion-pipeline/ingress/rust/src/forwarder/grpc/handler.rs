use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::event::Event as HttpEvent;
use super::elixir_client::{ElixirGrpcClient, chat};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn ingest_event(
    State(grpc_client): State<Arc<Mutex<ElixirGrpcClient>>>,
    Json(event): Json<HttpEvent>,
) -> impl IntoResponse {
    let grpc_event: chat::Event = match event.try_into() {
        Ok(e) => e,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid event").into_response(),
    };

    let mut client = grpc_client.lock().await;

    match client.ingest_event(grpc_event).await {
        Ok(resp) if resp.ok => "ok".into_response(),
        Ok(resp) => (StatusCode::BAD_GATEWAY, resp.error).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, "gRPC error").into_response(),
    }
}
