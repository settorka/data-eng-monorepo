use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::to_string;
use crate::event::ProcessedEvent;
use anyhow::{Result, anyhow};
use tracing::info;

pub async fn publish(event: &ProcessedEvent) -> Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "5000")
        .create()
        .map_err(|e| anyhow!("Failed to create producer: {e}"))?;

    let payload = to_string(event)?;
    let topic = "chat_events";

    // Sends with future-based delivery confirmation
    match producer
        .send(
            FutureRecord::to(topic)
                .payload(&payload)
                .key(&event.event_type),
            std::time::Duration::from_secs(5),
        )
        .await
    {
        Ok(Ok(delivery)) => {
            info!(
                "Delivered to topic={} partition={} offset={}",
                delivery.topic(),
                delivery.partition(),
                delivery.offset()
            );
            Ok(())
        }
        Ok(Err((e, _))) => Err(anyhow!("Broker delivery error: {e:?}")),
        Err(e) => Err(anyhow!("Send timeout or cancellation: {e:?}")),
    }
}
