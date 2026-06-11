use std::sync::Arc;

use anyhow::Result;
use event_ingestion_pipeline_v1::{
    config::Settings,
    ingress::routes::{self, AppState},
    kafka::producer::Publisher,
    observability::{
        health::HealthState,
        metrics,
    },
};
use tokio::sync::Semaphore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    // Initialize structured logging early for full visibility
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize metrics (no-op now, future exporter hook)
    metrics::init();

    let publisher = Publisher::new(&settings)?;

    let app = routes::router(AppState {
        settings: settings.clone(),
        health: Arc::new(HealthState::new()), // shared, thread-safe health state
        publisher,
        in_flight: Arc::new(Semaphore::new(settings.max_in_flight_requests)), // bounds concurrency
    });

    axum::Server::bind(&settings.bind_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}