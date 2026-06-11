use axum::{routing::get, routing::post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/chat/ingestion", post(ingest))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    "not-implemented"
}

async fn ingest() -> &'static str {
    "not-implemented"
}

