use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub bind_addr: SocketAddr,
    pub kafka_brokers: String,
    pub topic: String,
    pub publish_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub max_request_body_bytes: usize,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
            .parse()
            .context("invalid BIND_ADDR")?;

        let kafka_brokers =
            env::var("KAFKA_BROKERS").unwrap_or_else(|_| "127.0.0.1:9092".to_string());

        let topic = env::var("KAFKA_TOPIC").unwrap_or_else(|_| "chat_events".to_string());
        let publish_timeout_ms = env_u64("PUBLISH_TIMEOUT_MS", 5_000)?;
        let request_timeout_ms = env_u64("REQUEST_TIMEOUT_MS", 2_000)?;
        let max_request_body_bytes = env_usize("MAX_REQUEST_BODY_BYTES", 64 * 1024)?;

        Ok(Self {
            bind_addr,
            kafka_brokers,
            topic,
            publish_timeout_ms,
            request_timeout_ms,
            max_request_body_bytes,
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
