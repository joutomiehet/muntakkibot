FROM rust:bookworm AS builder
WORKDIR /usr/src/myapp

# Build dependencies first so they are cached until Cargo.toml or Cargo.lock change
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src src
RUN touch src/main.rs && cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /usr/src/mun_takki_bot
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/mun_takki_bot /usr/local/bin/mun_takki_bot
CMD ["mun_takki_bot"]
