FROM rust:1.82 as builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
COPY tests ./tests
RUN cargo build --release --bin persistence_consumer

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/persistence_consumer /app/persistence_consumer
CMD ["/app/persistence_consumer"]

