CREATE TABLE IF NOT EXISTS stock_trades (
    trade_id UUID PRIMARY KEY NOT NULL,
    stock_name TEXT NOT NULL,
    stock_price NUMERIC(12,2) NOT NULL,
    stock_purchase_choice TEXT CHECK (stock_purchase_choice IN ('BUY', 'SELL')) NOT NULL,
    trader_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Convert table to hypertable
SELECT create_hypertable('stock_trades', 'created_at', if_not_exists => TRUE);