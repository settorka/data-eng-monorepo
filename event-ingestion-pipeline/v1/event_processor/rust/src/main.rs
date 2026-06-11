mod event;
mod processor;
mod publisher;
mod api;
mod config;

use axum::{
    http::Request,
    middleware::Next,
    response::Response,
    Router, routing::post,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;
use std::time::Instant;
use tower::timeout::TimeoutLayer;
use tower_http::limit::RequestBodyLimitLayer;

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
    let settings = config::Settings::from_env().expect("invalid configuration");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let publish_source = Arc::new(Mutex::new(String::from("redpanda")));

    let api = Router::new()
        .route("/chat/ingestion", post(api::ingest_event))
        .layer(axum::middleware::from_fn(request_logger))
        .layer(RequestBodyLimitLayer::new(settings.max_request_body_bytes))
        .layer(TimeoutLayer::new(Duration::from_millis(settings.request_timeout_ms)))
        .with_state(publish_source);

    let app = Router::new()
        .nest("/api/v1", api);

    tracing::info!(
        bind_addr = %settings.bind_addr,
        kafka_brokers = %settings.kafka_brokers,
        kafka_topic = %settings.topic,
        publish_timeout_ms = settings.publish_timeout_ms,
        request_timeout_ms = settings.request_timeout_ms,
        max_request_body_bytes = settings.max_request_body_bytes,
        "event processor configuration loaded"
    );

    axum::Server::bind(&settings.bind_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
