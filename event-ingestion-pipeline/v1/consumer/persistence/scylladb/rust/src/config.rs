use std::env;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub scylla_host: String,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub consumer_group: String,
    pub queue_capacity: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub metrics_interval_secs: u64,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            scylla_host: env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".into()),
            kafka_brokers: env::var("KAFKA_BROKERS").unwrap_or_else(|_| "127.0.0.1:9092".into()),
            kafka_topic: env::var("KAFKA_TOPIC").unwrap_or_else(|_| "chat_events".into()),
            consumer_group: env::var("KAFKA_CONSUMER_GROUP")
                .unwrap_or_else(|_| "scylla-persistence-async".into()),
            queue_capacity: env_usize("QUEUE_CAPACITY", 10_000)?,
            batch_size: env_usize("BATCH_SIZE", 500)?,
            flush_interval_ms: env_u64("FLUSH_INTERVAL_MS", 2_000)?,
            metrics_interval_secs: env_u64("METRICS_INTERVAL_SECS", 5)?,
        })
    }
}

fn env_u64(key: &str, default: u64) -> Result<u64> {
    env::var(key)
        .ok()
        .map(|v| v.parse().with_context(|| format!("invalid {key}")))
        .transpose()?
        .map_or(Ok(default), Ok)
}

fn env_usize(key: &str, default: usize) -> Result<usize> {
    env::var(key)
        .ok()
        .map(|v| v.parse().with_context(|| format!("invalid {key}")))
        .transpose()?
        .map_or(Ok(default), Ok)
}
