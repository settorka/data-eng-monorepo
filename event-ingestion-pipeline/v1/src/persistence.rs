use anyhow::Result;

use crate::event::EventEnvelope;

pub async fn persist_batch(_events: &[EventEnvelope]) -> Result<()> {
    anyhow::bail!("not implemented")
}

