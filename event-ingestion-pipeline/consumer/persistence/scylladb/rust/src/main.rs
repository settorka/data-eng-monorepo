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
    // Logger init
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting ScyllaDB persistence consumer...");

    // Connects to Scylla cluster
    let session: Session = database::init_session().await?;
    let session = Arc::new(session);

    // async task for consumer loop
    let consumer_handle = tokio::spawn(consumer::run(session.clone()));

    // Fault tolerant shutdown
    tokio::select! {
        res = consumer_handle => {
            if let Err(e) = res {
                tracing::error!("Consumer task ended with error: {:?}", e);
            }
        }
        _ = signal::ctrl_c() => {
            tracing::warn!("Shutdown signal received. Exiting...");
        }
    }

    Ok(())
}
