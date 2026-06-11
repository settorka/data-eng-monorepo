use anyhow::Result;
use event_ingestion_pipeline_v1::{
    config::Settings,
    kafka::clickhouse_consumer,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    clickhouse_consumer::run(settings).await
}