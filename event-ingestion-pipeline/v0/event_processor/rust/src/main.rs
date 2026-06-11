mod event;
mod processor;
mod publisher;
mod api;

use axum::{
    http::Request,
    middleware::Next,
    response::Response,
    Router, routing::post,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;
use std::time::Instant;

/// Logging middleware: logs method, path, status and latency
async fn request_logger<B>(req: Request<B>, next: Next<B>) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let resp = next.run(req).await;

    let status = resp.status();

    let elapsed = start.elapsed();
    info!(
        method = %method,
        path = %path,
        status = %status.as_u16(),
        latency_ms = %elapsed.as_millis(),
        "request completed"
    );

    resp
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let publish_source = Arc::new(Mutex::new(String::from("redpanda")));

    let api = Router::new()
        .route("/chat/ingestion", post(api::ingest_event))
        .layer(axum::middleware::from_fn(request_logger))
        .with_state(publish_source);

    let app = Router::new()
        .nest("/api/v1", api);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("API listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
