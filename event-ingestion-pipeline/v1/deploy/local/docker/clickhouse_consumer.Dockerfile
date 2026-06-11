FROM rust:1.91 as builder

RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
COPY tests ./tests

RUN cargo build --release --bin clickhouse_consumer

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/clickhouse_consumer /app/clickhouse_consumer

CMD ["/app/clickhouse_consumer"]