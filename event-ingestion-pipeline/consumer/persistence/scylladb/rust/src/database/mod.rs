use anyhow::Result;
use scylla::{Session, SessionBuilder};

/// Initialise a ScyllaDB session.
/// TODO: support multiple nodes (might go into a load balancer)
pub async fn init_session() -> Result<Session> {
    let session = SessionBuilder::new()
        .known_node("127.0.0.1:9042") 
        .build()
        .await?;

    tracing::info!("Connected to ScyllaDB cluster");
    Ok(session)
}

pub mod repository;
