# Example of a docker-compose.yml file

version: '3.8'

services:
muntakkibot:
build: .
environment:
TELOXIDE_TOKEN: "<token>"
volumes: - .:/usr/src/muntakkibot
command: ["mun_takki_bot"]

# Running

Insert your bots token in the TELOXIDE_TOKEN then
docker compose up -d
