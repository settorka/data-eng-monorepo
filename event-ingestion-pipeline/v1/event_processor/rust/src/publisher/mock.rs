use crate::event::ProcessedEvent;
use anyhow::Result;

pub async fn publish(event: &ProcessedEvent) -> Result<()> {
    println!("[MOCK PUBLISH] {:?}", event);
    Ok(())
}
