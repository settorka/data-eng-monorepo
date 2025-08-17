"""
Continuously generate random stock trade events (insert, update, delete) into Postgres.
"""

import asyncio
import argparse
import logging
import random
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import text
from stock_events_db_access_interface import StockTrade, get_session

# -- Logger Setup --
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s | %(levelname)s | %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)


# -- Insert Operation --
async def insert_trade(session: AsyncSession) -> None:
    trade = StockTrade(
        stock_name=random.choice(["AAPL", "GOOG", "MSFT", "AMZN", "TSLA"]),
        stock_price=round(random.uniform(50, 500), 2),
        stock_purchase_choice=random.choice(["BUY", "SELL"]),
    )
    session.add(trade)
    await session.commit()
    logging.info(f"Inserted: {trade.trade_id}")


# -- Update Operation --
async def update_random_trade(session: AsyncSession) -> None:
    query = text(
        """
        WITH random_trade AS (
            SELECT trade_id, created_at FROM stock_trades ORDER BY RANDOM() LIMIT 1
        )
        UPDATE stock_trades
        SET stock_price = ROUND(random() * 500 + 50, 2),
            stock_purchase_choice = CASE stock_purchase_choice WHEN 'BUY' THEN 'SELL' ELSE 'BUY' END,
            updated_at = NOW()
        FROM random_trade
        WHERE stock_trades.trade_id = random_trade.trade_id
          AND stock_trades.created_at = random_trade.created_at
        RETURNING stock_trades.trade_id
    """
    )
    result = await session.execute(query)
    updated = result.fetchone()
    await session.commit()
    if updated:
        logging.info(f"Updated: {updated[0]}")
    else:
        logging.info("Update skipped: no record found")


# -- Delete Operation --
async def delete_random_trade(session: AsyncSession) -> None:
    query = text(
        """
        WITH to_delete AS (
            SELECT trade_id, created_at FROM stock_trades ORDER BY RANDOM() LIMIT 1
        )
        DELETE FROM stock_trades
        USING to_delete
        WHERE stock_trades.trade_id = to_delete.trade_id
          AND stock_trades.created_at = to_delete.created_at
        RETURNING stock_trades.trade_id
    """
    )
    result = await session.execute(query)
    deleted = result.fetchone()
    await session.commit()
    if deleted:
        logging.info(f"Deleted: {deleted[0]}")
    else:
        logging.info("Delete skipped: no record found")


# -- Event Loop --
async def generate_events(rate: float) -> None:
    async for session in get_session():
        while True:
            op = random.choice(["INSERT", "UPDATE", "DELETE"])
            try:
                if op == "INSERT":
                    await insert_trade(session)
                elif op == "UPDATE":
                    await update_random_trade(session)
                else:
                    await delete_random_trade(session)
            except Exception as e:
                logging.error(f"{op} failed: {e}")
            await asyncio.sleep(1 / rate)


# -- CLI Entry Point --
def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate CDC stock events in real-time."
    )
    parser.add_argument("--rate", type=float, required=True, help="Ops per second")
    args = parser.parse_args()

    try:
        asyncio.run(generate_events(args.rate))
    except KeyboardInterrupt:
        logging.info("Generator stopped by user.")


if __name__ == "__main__":
    main()
