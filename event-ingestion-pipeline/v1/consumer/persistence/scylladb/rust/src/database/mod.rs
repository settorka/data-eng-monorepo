use anyhow::Result;
use scylla::{Session, SessionBuilder};

use crate::config::Settings;

/// Initialise a ScyllaDB session.
/// TODO: support multiple nodes (might go into a load balancer)
pub async fn init_session(settings: &Settings) -> Result<Session> {
    let session = SessionBuilder::new()
        .known_node(&settings.scylla_host)
        .build()
        .await?;

    tracing::info!("Connected to ScyllaDB cluster");
    Ok(session)
}

pub mod repository;
