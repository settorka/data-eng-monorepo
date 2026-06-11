use event_ingestion_pipeline_v1::config::Settings;

#[test]
fn settings_have_defaults() {
    let settings = Settings::from_env().expect("settings should parse");
    assert_eq!(settings.kafka_topic, "chat_events");
    assert_eq!(settings.max_request_body_bytes, 64 * 1024);
}

