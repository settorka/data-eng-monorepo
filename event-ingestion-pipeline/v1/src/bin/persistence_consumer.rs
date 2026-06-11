use anyhow::Result;
use event_ingestion_pipeline_v1::{
    config::Settings, consumer, metrics, persistence::Persistence, publisher::Publisher,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    metrics::init();
    let persistence = Persistence::connect(&settings).await?;
    let dlq_publisher = Publisher::new(&Settings {
        kafka_topic: settings.dlq_topic.clone(),
        ..settings.clone()
    })?;
    consumer::run(settings, persistence, dlq_publisher).await
}
