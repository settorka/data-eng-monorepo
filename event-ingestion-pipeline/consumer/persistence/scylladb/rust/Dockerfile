FROM rust:1.82 AS builder

RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:stable-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/scylladb_consumer /app/scylladb_consumer

CMD ["/app/scylladb_consumer"]
