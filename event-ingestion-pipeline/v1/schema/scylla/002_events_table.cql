CREATE TABLE IF NOT EXISTS events.events (
    event_id text,
    event_type text,
    user_id text,
    room_id text,
    payload text,
    producer_timestamp bigint,
    server_timestamp bigint,
    schema_version int,

    PRIMARY KEY ((room_id), server_timestamp, event_id)
)
WITH CLUSTERING ORDER BY (server_timestamp DESC);