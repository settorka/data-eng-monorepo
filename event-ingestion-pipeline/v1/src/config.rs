use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub bind_addr: SocketAddr,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub kafka_consumer_group: String,
    pub scylla_host: String,
    pub max_request_body_bytes: usize,
    pub request_timeout_ms: u64,
    pub publish_timeout_ms: u64,
    pub queue_capacity: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
                .parse()
                .context("invalid BIND_ADDR")?,
            kafka_brokers: env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "127.0.0.1:9092".to_string()),
            kafka_topic: env::var("KAFKA_TOPIC")
                .unwrap_or_else(|_| "chat_events".to_string()),
            kafka_consumer_group: env::var("KAFKA_CONSUMER_GROUP")
                .unwrap_or_else(|_| "chat-events-v1".to_string()),
            scylla_host: env::var("SCYLLA_HOST")
                .unwrap_or_else(|_| "127.0.0.1:9042".to_string()),
            max_request_body_bytes: env_usize("MAX_REQUEST_BODY_BYTES", 64 * 1024)?,
            request_timeout_ms: env_u64("REQUEST_TIMEOUT_MS", 2_000)?,
            publish_timeout_ms: env_u64("PUBLISH_TIMEOUT_MS", 5_000)?,
            queue_capacity: env_usize("QUEUE_CAPACITY", 10_000)?,
            batch_size: env_usize("BATCH_SIZE", 500)?,
            flush_interval_ms: env_u64("FLUSH_INTERVAL_MS", 2_000)?,
        })
    }
}

fn env_u64(key: &str, default: u64) -> Result<u64> {
    env::var(key)
        .ok()
        .map(|value| value.parse().with_context(|| format!("invalid {key}")))
        .transpose()?
        .map_or(Ok(default), Ok)
}

fn env_usize(key: &str, default: usize) -> Result<usize> {
    env::var(key)
        .ok()
        .map(|value| value.parse().with_context(|| format!("invalid {key}")))
        .transpose()?
        .map_or(Ok(default), Ok)
}

