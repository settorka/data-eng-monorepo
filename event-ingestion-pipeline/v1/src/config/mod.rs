use std::{env, net::SocketAddr};

use anyhow::{ensure, Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub bind_addr: SocketAddr,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub dlq_topic: String,
    pub kafka_consumer_group: String,
    pub scylla_host: String,
    pub scylla_keyspace: String,
    pub scylla_table: String,
    pub max_request_body_bytes: usize,
    pub request_timeout_ms: u64,
    pub publish_timeout_ms: u64,
    pub queue_capacity: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub max_in_flight_requests: usize,
    pub max_retry_attempts: usize,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let settings = Self {
            bind_addr: env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
                .parse()
                .context("invalid BIND_ADDR")?,
            kafka_brokers: env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "127.0.0.1:9092".to_string()),
            kafka_topic: env::var("KAFKA_TOPIC").unwrap_or_else(|_| "chat_events".to_string()),
            dlq_topic: env::var("DLQ_TOPIC").unwrap_or_else(|_| "chat_events_dlq".to_string()),
            kafka_consumer_group: env::var("KAFKA_CONSUMER_GROUP")
                .unwrap_or_else(|_| "chat-events-v1".to_string()),
            scylla_host: env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".to_string()),
            scylla_keyspace: env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "chat_app".to_string()),
            scylla_table: env::var("SCYLLA_TABLE")
                .unwrap_or_else(|_| "processed_events".to_string()),
            max_request_body_bytes: env_usize("MAX_REQUEST_BODY_BYTES", 64 * 1024)?,
            request_timeout_ms: env_u64("REQUEST_TIMEOUT_MS", 2_000)?,
            publish_timeout_ms: env_u64("PUBLISH_TIMEOUT_MS", 5_000)?,
            queue_capacity: env_usize("QUEUE_CAPACITY", 10_000)?,
            batch_size: env_usize("BATCH_SIZE", 500)?,
            flush_interval_ms: env_u64("FLUSH_INTERVAL_MS", 2_000)?,
            max_in_flight_requests: env_usize("MAX_IN_FLIGHT_REQUESTS", 1_024)?,
            max_retry_attempts: env_usize("MAX_RETRY_ATTEMPTS", 3)?,
        };

        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.max_request_body_bytes > 0 && self.max_request_body_bytes <= 64 * 1024,
            "MAX_REQUEST_BODY_BYTES must be between 1 and 65536"
        );
        ensure!(
            self.request_timeout_ms > 0 && self.request_timeout_ms <= 30_000,
            "REQUEST_TIMEOUT_MS must be between 1 and 30000"
        );
        ensure!(
            self.publish_timeout_ms > 0 && self.publish_timeout_ms <= 30_000,
            "PUBLISH_TIMEOUT_MS must be between 1 and 30000"
        );
        ensure!(
            self.queue_capacity > 0 && self.queue_capacity <= 100_000,
            "QUEUE_CAPACITY must be between 1 and 100000"
        );
        ensure!(
            self.batch_size > 0 && self.batch_size <= self.queue_capacity,
            "BATCH_SIZE must be between 1 and QUEUE_CAPACITY"
        );
        ensure!(
            self.flush_interval_ms > 0 && self.flush_interval_ms <= 60_000,
            "FLUSH_INTERVAL_MS must be between 1 and 60000"
        );
        ensure!(
            self.max_in_flight_requests > 0 && self.max_in_flight_requests <= 10_000,
            "MAX_IN_FLIGHT_REQUESTS must be between 1 and 10000"
        );
        ensure!(
            self.max_retry_attempts > 0 && self.max_retry_attempts <= 10,
            "MAX_RETRY_ATTEMPTS must be between 1 and 10"
        );
        Ok(())
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
