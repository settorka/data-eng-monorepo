# TimescaleDB (PostgreSQL + time-series)

This folder sets up TimescaleDB with:
- A hypertable called `stock_trades` partitioned on `created_at`
- Schema designed for simulated CDC events

## Table: stock_trades

| Column              | Type        |
|---------------------|-------------|
| trade_id            | UUID (PK)   |
| stock_name          | TEXT        |
| stock_price         | NUMERIC     |
| stock_purchase_choice | BUY/SELL |
| trader_id           | UUID        |
| created_at          | TIMESTAMPTZ |
| updated_at          | TIMESTAMPTZ |

> NOTE: This DB will emit WAL changes, consumed by Debezium → Kafka.
