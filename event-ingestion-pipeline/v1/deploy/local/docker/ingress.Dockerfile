FROM rust:1.82 as builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
COPY tests ./tests
RUN cargo build --release --bin ingress

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/ingress /app/ingress
CMD ["/app/ingress"]

