use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::to_string;
use crate::event::ProcessedEvent;
use anyhow::{Result, anyhow};
use tracing::info;

pub async fn publish(event: &ProcessedEvent) -> Result<()> {
    // Create a producer
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "5000")
        .create()
        .map_err(|e| anyhow!("Failed to create producer: {e}"))?;

    let payload = to_string(event)?;
    let topic = "chat_events";

    // Awaiting broker confirmation
    match producer
        .send(
            FutureRecord::to(topic)
                .payload(&payload)
                .key(&event.event_type),
            std::time::Duration::from_secs(5),
        )
        .await
    {
        Ok((partition, offset)) => {
            info!(
                "Delivered to topic={} partition={} offset={}",
                topic, partition, offset
            );
            Ok(())
        }
        Err((e, _msg)) => Err(anyhow!("Broker delivery error: {e:?}")),
    }
}
