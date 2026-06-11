CREATE TABLE IF NOT EXISTS events.events (
    event_id String,
    event_type String,
    user_id String,
    room_id String,
    payload String,
    created_at DateTime
)
ENGINE = MergeTree()
ORDER BY (room_id, created_at);