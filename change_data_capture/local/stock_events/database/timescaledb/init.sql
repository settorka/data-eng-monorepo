-- time series data
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Table to track stock trades
CREATE TABLE IF NOT EXISTS stock_trades (
    trade_id UUID NOT NULL,
    stock_name TEXT NOT NULL,
    stock_price NUMERIC(12,2) NOT NULL,
    stock_purchase_choice TEXT CHECK (stock_purchase_choice IN ('BUY', 'SELL')) NOT NULL,
    trader_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (trade_id, created_at)  -- composite key ensures Timescale hypertable works
);

-- Convert to hypertable (partitioned by created_at)
SELECT create_hypertable('stock_trades', 'created_at', if_not_exists => TRUE);

-- Indexes on components of composite key
CREATE INDEX IF NOT EXISTS idx_stock_trades_trade_id
    ON stock_trades (trade_id);
    
CREATE INDEX IF NOT EXISTS idx_stock_trades_created_at
    ON stock_trades (created_at DESC);
