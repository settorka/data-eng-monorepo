FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/event_processor /usr/local/bin/event_processor
EXPOSE 3000
CMD ["event_processor"]
