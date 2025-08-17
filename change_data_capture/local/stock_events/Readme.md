# Real-Time CDC Pipeline with TimescaleDB, Debezium, Kafka, and MinIO

This project simulates a Change Data Capture (CDC) pipeline that ingests synthetic stock trade events from a PostgreSQL-compatible TimescaleDB into Kafka using Debezium, for downstream consumption into data lakes (e.g., Apache Hudi on MinIO via Spark).

## Architecture Overview

```
Postgres (TimescaleDB w/ WAL)
  │
  ▼
Debezium (Kafka Connect)
  │
  ▼
Kafka → Topic: `cdc.public.stock_trades`
  │
  ├── Realtime Consumer 1: Spark Structured Streaming → Hudi MOR Table → MinIO
  └── Realtime Consumer 2: (optional) Streamlit Dashboard / Logging
```

## Goals

- Simulate time-series stock trade data (inserts, updates, deletes)
- Capture database changes in real-time using Debezium + Kafka Connect
- Push CDC events into Kafka topic for downstream processing
- Persist and analyze via lakehouse engines like Apache Hudi (not included yet)

## Services via Docker Compose

| Service         | Purpose                            | Port |
|----------------|------------------------------------|------|
| TimescaleDB     | PostgreSQL w/ time-series support | 5432 |
| Zookeeper       | Kafka dependency                   | 2181 |
| Kafka           | Messaging system                   | 9092, 29092 |
| Kafka Connect   | Debezium runtime                   | 8083 |
| Kafka UI        | Kafka topic viewer                 | 8080 |
| MinIO           | S3-compatible storage for Hudi     | 9000, 9001 |

Ensure Docker Compose is installed and run:

```bash
docker compose up --build -d
```

## Debezium Configuration (Connector JSON)

POST to Kafka Connect (http://localhost:8083/connectors) with:

```json
{
  "name": "stock_trades_connector",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.hostname": "timescaledb",
    "database.port": "5432",
    "database.user": "postgres",
    "database.password": "postgres",
    "database.dbname": "trading",
    "database.server.name": "cdc",
    "plugin.name": "pgoutput",
    "slot.name": "cdc_slot",
    "topic.prefix": "cdc",
    "publication.name": "cdc_publication",
    "table.include.list": "public.stock_trades",
    "tombstones.on.delete": "false",
    "include.schema.changes": "false",
    "key.converter": "org.apache.kafka.connect.json.JsonConverter",
    "value.converter": "org.apache.kafka.connect.json.JsonConverter"
  }
}
```

Run with Postman or:

```bash
curl -X POST http://localhost:8083/connectors \
  -H "Content-Type: application/json" \
  -d @stock_trades_connector.json
```

Check registration:

```bash
curl http://localhost:8083/connectors
```

## TimescaleDB Table

This is automatically created via the `init.sql`:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS stock_trades (
    trade_id UUID NOT NULL,
    stock_name TEXT NOT NULL,
    stock_price NUMERIC(12,2) NOT NULL,
    stock_purchase_choice TEXT CHECK (stock_purchase_choice IN ('BUY', 'SELL')) NOT NULL,
    trader_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (trade_id, created_at)
);

SELECT create_hypertable('stock_trades', 'created_at', if_not_exists => TRUE);
```

## Python Producer CLI Tools

Create a virtual environment:

```bash
python3 -m venv cdc-stock-events
source cdc-stock-events/bin/activate
pip install -r requirements.txt
```

### `load_stock_events.py`

Bulk inserts trades to seed the DB.

```bash
python load_stock_events.py --record-count 50
```

### `generate_stock_events.py`

Runs forever, generating random insert, update, and delete events at a configurable rate.

```bash
python generate_stock_events.py --rate 5
```

- Internally throttles with `asyncio.sleep(1 / rate)`
- Uses `ORDER BY RANDOM()` to fetch rows for update/delete
- Logs every action (inserted/updated/deleted)

## Monitoring

- Kafka UI: http://localhost:8080
- Kafka Connect API: http://localhost:8083
- MinIO: http://localhost:9001

## Deep Dive: How Debezium Works

- PostgreSQL’s WAL logs every change (insert/update/delete)
- Debezium runs as a Kafka Connect plugin
- It reads WAL changes via logical decoding and pushes them to a Kafka topic
- Consumers (like Spark) can then subscribe to these events for ingestion

## Next Steps

- Create Spark Structured Streaming job to consume `cdc.public.stock_trades`
- Write to Hudi Merge-On-Read table in MinIO
- Add compaction and querying layer (Trino, DuckDB, etc.)
- Optional: stream to Streamlit dashboard

## Folder Structure (WIP)

```
stock_events/
├── docker-compose.yml
├── database/
│   └── timescaledb/
│       └── init.sql
├── broker/
│   └── connect_config/
│       └── stock_trades_connector.json
├── producer/
│   ├── load_stock_events.py
│   ├── generate_stock_events.py
│   ├── stock_events_db_access_interface.py
│   └── .env
```

## Status

- [x] Docker Compose up and running
- [x] Debezium connector active
- [x] Table seeded and changes streaming to Kafka
- [x] Async Python producer generating real-time events
- [ ] Spark CDC consumer + Hudi ingestion (next step)