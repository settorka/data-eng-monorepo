use std::time::Duration;

use anyhow::{Context, Result};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use tokio::time::timeout;

use crate::{config::Settings, domain::event::EventEnvelope};

#[derive(Clone)]
pub struct Publisher {
    producer: FutureProducer,
    topic: String,
    publish_timeout: Duration,
}

impl Publisher {
    pub fn new(settings: &Settings) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &settings.kafka_brokers)
            .set(
                "message.timeout.ms",
                settings.publish_timeout_ms.to_string(),
            )
            .set(
                "queue.buffering.max.messages",
                settings.queue_capacity.to_string(),
            )
            .create()
            .context("failed to create kafka producer")?;

        Ok(Self {
            producer,
            topic: settings.kafka_topic.clone(),
            publish_timeout: Duration::from_millis(settings.publish_timeout_ms),
        })
    }

    pub async fn publish(&self, event: &EventEnvelope) -> Result<()> {
        self.publish_json(event, &event.event_id).await
    }

    pub async fn publish_json<T: serde::Serialize>(&self, value: &T, key: &str) -> Result<()> {
        let payload =
            serde_json::to_string(value).context("failed to serialize publish payload")?;
        let record = FutureRecord::to(&self.topic).key(key).payload(&payload);

        let delivery = timeout(
            self.publish_timeout,
            self.producer.send(record, self.publish_timeout),
        )
        .await
        .context("timed out waiting for broker ack")?;

        match delivery {
            Ok(_) => Ok(()),
            Err((error, _)) => Err(anyhow::anyhow!(error)).context("broker rejected publish"),
        }
    }
}
