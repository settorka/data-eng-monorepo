use anyhow::Result;

use crate::event::EventEnvelope;

pub async fn publish(_event: &EventEnvelope) -> Result<()> {
    anyhow::bail!("not implemented")
}

