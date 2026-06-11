FROM rust:1.91.1 as builder

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
RUN cargo build --release --bin persistence_consumer

FROM debian:trixie-slim
WORKDIR /app
COPY --from=builder /app/target/release/persistence_consumer /app/persistence_consumer
CMD ["/app/persistence_consumer"]

