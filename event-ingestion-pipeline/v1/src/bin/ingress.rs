use anyhow::Result;
use event_ingestion_pipeline_v1::{config::Settings, http, metrics};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    metrics::init();

    let app = http::router();

    axum::Server::bind(&settings.bind_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

