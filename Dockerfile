FROM rust:latest

WORKDIR /usr/src/muntakkibot
COPY . .

ENV TELOXIDE_TOKEN="YOUR_TELEGRAM_BOT_TOKEN"

RUN cargo install --path .

ENV PATH="/root/.cargo/bin:${PATH}"

CMD ["mun_takki_bot"]
