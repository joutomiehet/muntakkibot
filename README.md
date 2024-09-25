# MunTakkiBot

## quickstart

### Generate `.env`

```sh
cat << EOF > .env
TELOXIDE_TOKEN=<token>
UID=$(id -u)
GID=$(id -g)
IMAGES=./images
EOF
```

### Run container

```sh
docker compose up -d
```
