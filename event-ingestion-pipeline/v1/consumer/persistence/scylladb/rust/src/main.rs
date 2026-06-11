mod consumer;
mod config;
mod database;
mod metrics;
mod model;

use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let settings = config::Settings::from_env()?;
    metrics::logger::init_with_interval(settings.metrics_interval_secs);

    tracing::info!(
        scylla_host = %settings.scylla_host,
        kafka_brokers = %settings.kafka_brokers,
        kafka_topic = %settings.kafka_topic,
        consumer_group = %settings.consumer_group,
        queue_capacity = settings.queue_capacity,
        batch_size = settings.batch_size,
        flush_interval_ms = settings.flush_interval_ms,
        "consumer configuration loaded"
    );

    let session = Arc::new(database::init_session(&settings).await?);

    consumer::run(session, settings).await?;

    Ok(())
}
