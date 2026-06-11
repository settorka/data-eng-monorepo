use anyhow::Result;
use event_ingestion_pipeline_v1::{config::Settings, consumer, metrics};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let _settings = Settings::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    metrics::init();
    consumer::run().await
}

