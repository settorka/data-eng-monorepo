use anyhow::Result;

use crate::event::EventEnvelope;

pub async fn write_failure(_event: &EventEnvelope, _reason: &str) -> Result<()> {
    anyhow::bail!("not implemented")
}

