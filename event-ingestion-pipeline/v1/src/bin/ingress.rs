use std::sync::Arc;

use anyhow::Result;
use event_ingestion_pipeline_v1::{
    config::Settings,
    ingress::routes::{self, AppState},
    kafka::producer::Publisher,
    observability::{
        health::{HealthState, Readiness},
        metrics,
    },
};
use tokio::sync::Semaphore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    metrics::init();

    let publisher = Publisher::new(&settings)?;
    let app = routes::router(AppState {
        settings: settings.clone(),
        health: HealthState::new(Readiness::Ready),
        publisher,
        in_flight: Arc::new(Semaphore::new(settings.max_in_flight_requests)),
    });

    axum::Server::bind(&settings.bind_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
