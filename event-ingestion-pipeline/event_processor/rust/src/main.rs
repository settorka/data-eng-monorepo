mod event;
mod processor;
mod publisher;
mod api;

use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let publish_source = Arc::new(Mutex::new(String::from("redpanda")));

    let api = Router::new()
        .route("/chat/ingestion", post(api::ingest_event))
        .with_state(publish_source);

    let app = Router::new().nest("/api/v1", api);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
