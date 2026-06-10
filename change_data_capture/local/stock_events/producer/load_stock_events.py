"""
Script to bulk insert randomly generated stock trades into a PostgreSQL database.
"""

import argparse
import asyncio
import random
import uuid
from datetime import datetime

from faker import Faker
from sqlalchemy.ext.asyncio import AsyncSession
from stock_events_db_access_interface import StockTrade, get_session

fake = Faker()


async def insert_trade(session: AsyncSession) -> StockTrade:
    """
    Creates and adds a new StockTrade record to the given session.

    Args:
        session (AsyncSession): SQLAlchemy async session.

    Returns:
        StockTrade: The created trade object.
    """
    trade = StockTrade(
        trade_id=uuid.uuid4(),
        stock_name=random.choice(["AAPL", "GOOG", "MSFT", "AMZN", "TSLA"]),
        stock_price=round(random.uniform(50, 500), 2),
        stock_purchase_choice=random.choice(["BUY", "SELL"]),
        trader_id=uuid.uuid4(),
        created_at=datetime.utcnow(),
        updated_at=datetime.utcnow(),
    )
    session.add(trade)
    return trade


async def load_data(record_count: int) -> None:
    """
    Inserts the specified number of trade records into the database.

    Args:
        record_count (int): Number of trades to insert.
    """
    async for session in get_session():
        for _ in range(record_count):
            trade = await insert_trade(session)
            print(f"Inserting trade {trade.trade_id} {trade.stock_name}")
        await session.commit()
        print(f"Inserted {record_count} trades.")


def main() -> None:
    """
    CLI entry point for bulk inserting stock trades.
    """
    parser = argparse.ArgumentParser(description="Bulk load stock trades into Postgres")
    parser.add_argument(
        "--record-count", type=int, required=True, help="Number of trades to insert"
    )
    args = parser.parse_args()
    asyncio.run(load_data(args.record_count))


if __name__ == "__main__":
    main()
