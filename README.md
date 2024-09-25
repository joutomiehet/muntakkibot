# MunTakkiBot

## quickstart

### Generate `.env`

```sh
cat << EOF > .env
TELOXIDE_TOKEN=<token>
UID=$(id -u)
GID=$(id -g)
IMAGES=./images
IMAGE_DIR=<full path to the image folder>
EOF
```

### Run container

```sh
docker compose up -d
```
