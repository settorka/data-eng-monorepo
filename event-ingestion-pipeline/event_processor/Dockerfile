FROM rust:1.82 as builder

RUN apt-get update && \
    apt-get install -y cmake g++ pkg-config libssl-dev zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY rust/ ./rust/

WORKDIR /app/rust
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/rust/target/release/event_processor /app/event_processor

CMD ["/app/event_processor"]
