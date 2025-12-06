use crate::event::Event;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use serde_json;


static TOPIC: &str = "chat-events";

pub async fn produce_to_redpanda(event: Event) -> Result<(),()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers","localhost:9092")
        .set("message.timeout.ms","5000")
        .create()
        .map_err(|_| ())?;
    
    
    let payload = serde_json::to_string(&event).map_err(|_| ())?;

    let key = match &event {
        Event::Chat(e) => &e.room_id,
        Event::Join(e) => &e.room_id,
        Event::Leave(e) => &e.room_id,
        Event::Reaction(e) => &e.room_id,
    };

    producer
        .send(
            FutureRecord::to(TOPIC)
                .payload(&payload)
                .key(key),
            Duration::from_secs(0),
        )
        .await
        .map(|delivery| {
            if let Err((e, _)) = delivery {
                eprintln!("Delivery error: {e}");
            }
        })
        .map_err(|_| ())?;

    Ok(())

}