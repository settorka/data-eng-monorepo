use std::sync::Arc;
use anyhow::Result;
use scylla::Session;
use tokio::signal;
use tracing_subscriber;

mod consumer;
mod database;
mod metrics;
mod model;

#[tokio::main]
async fn main() -> Result<()> {

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting ScyllaDB persistence consumer...");

    
    let interval_secs = std::env::var("METRICS_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    metrics::logger::init_with_interval(interval_secs);
    tracing::info!("Metrics reporting every {}s", interval_secs);

    let scylla_host = std::env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".into());
    std::env::set_var("SCYLLA_HOST", &scylla_host); // optional consistency
    tracing::info!("Connecting to ScyllaDB at {}", scylla_host);

    let session: Session = database::init_session().await?;
    let session = Arc::new(session);

   
    let consumer_handle = tokio::spawn(consumer::run(session.clone()));

    tokio::select! {
        res = consumer_handle => {
            if let Err(e) = res {
                tracing::error!("Consumer task ended with error: {:?}", e);
            }
        }
        _ = signal::ctrl_c() => {
            tracing::warn!("Shutdown signal received. Exiting gracefully...");
        }
    }

    tracing::info!("Shutdown complete. Goodbye.");
    Ok(())
}
