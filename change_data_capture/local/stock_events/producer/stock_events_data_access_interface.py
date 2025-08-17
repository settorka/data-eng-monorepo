import os
import uuid
from datetime import datetime
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker, declarative_base
from sqlalchemy import Column, String, Numeric, TIMESTAMP, CheckConstraint
from dotenv import load_dotenv

# Load .env file
load_dotenv()

DB_USER = os.getenv("STOCK_EVENTS_DB_USER", "postgres")
DB_PASSWORD = os.getenv("STOCK_EVENTS_DB_PASSWORD", "postgres")
DB_HOST = os.getenv("STOCK_EVENTS_DB_HOST", "localhost")
DB_PORT = os.getenv("STOCK_EVENTS_DB_PORT", "5432")
DB_NAME = os.getenv("STOCK_EVENTS_DB_NAME", "trading")

DATABASE_URL = (
    f"postgresql+asyncpg://{DB_USER}:{DB_PASSWORD}@{DB_HOST}:{DB_PORT}/{DB_NAME}"
)

# Async engine & session factory
engine = create_async_engine(DATABASE_URL, echo=True, future=True)
AsyncSessionLocal = sessionmaker(
    bind=engine, class_=AsyncSession, expire_on_commit=False
)

Base = declarative_base()


class StockTrade(Base):
    __tablename__ = "stock_trades"

    trade_id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    stock_name = Column(String, nullable=False)
    stock_price = Column(Numeric(12, 2), nullable=False)
    stock_purchase_choice = Column(
        String,
        CheckConstraint("stock_purchase_choice IN ('BUY','SELL')"),
        nullable=False,
    )
    trader_id = Column(String, nullable=False, default=lambda: str(uuid.uuid4()))
    created_at = Column(TIMESTAMP, nullable=False, default=datetime.utcnow)
    updated_at = Column(TIMESTAMP, nullable=False, default=datetime.utcnow)


# Dependency: create new session
async def get_session():
    async with AsyncSessionLocal() as session:
        yield session
