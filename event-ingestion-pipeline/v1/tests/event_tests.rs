use event_ingestion_pipeline_v1::event::EventEnvelope;

#[test]
fn event_envelope_has_schema_version() {
    let event = EventEnvelope::new(
        "chat".to_string(),
        "user-1".to_string(),
        "room-1".to_string(),
        serde_json::json!({ "message": "hello" }),
        None,
    );

    assert_eq!(event.schema_version, "v1");
    assert!(!event.event_id.is_empty());
}
