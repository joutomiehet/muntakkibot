FROM rust:bookworm as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR  /usr/src/mun_takki_bot
RUN apt-get update && apt install libssl3 ca-certificates -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/mun_takki_bot /usr/local/bin/mun_takki_bot
CMD ["mun_takki_bot"]
