mod consumer;
mod database;
mod metrics;
mod model;

use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let interval_secs = std::env::var("METRICS_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    metrics::logger::init_with_interval(interval_secs);

    let scylla_host = std::env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".into());
    tracing::info!("Connecting to ScyllaDB at {}", scylla_host);

    let session = Arc::new(database::init_session().await?);

    consumer::run(session).await?;

    Ok(())
}
