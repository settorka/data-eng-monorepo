use crate::event::ProcessedEvent;
use anyhow::Result;

pub mod redpanda;
pub mod mock;

pub async fn publish(source: &str, event: &ProcessedEvent) -> Result<()> {
    match source {
        "redpanda" => redpanda::publish(event).await,
        "mock" => mock::publish(event).await,
        other => Err(anyhow::anyhow!("Unknown publish source: {}", other)),
    }
}
