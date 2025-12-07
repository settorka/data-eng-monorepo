use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::to_string;
use crate::event::ProcessedEvent;
use anyhow::Result;

pub async fn publish(event: &ProcessedEvent) -> Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092") // Kakfa endpoint, but it's Redpanda-compatible
        .create()?;
    let payload = to_string(event)?;
    producer
        .send(
            FutureRecord::to("chat_events").payload(&payload).key(&event.event_type),
            std::time::Duration::from_secs(0),
        )
        .await?;
    Ok(())
}
