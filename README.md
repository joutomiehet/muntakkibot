# MunTakkiBot

## quickstart

### Generate `.env`

```sh
cat << EOF > .env
TELOXIDE_TOKEN=<token>
UID=$(id -u)
GID=$(id -g)
IMAGES=./images
ALLOWED_CHATS=-1001234567890,-1009876543210
EOF
```

### Allowed chats

`ALLOWED_CHATS` is a comma-separated list of group chat ids. A user can save a takki only if they
are a member of at least one of these groups. If it is left empty, everyone can save a takki.

The bot can only check members of groups it has been added to (it does not need to be an admin).
Listed groups the bot is not in are skipped, so their members can't save a takki unless they are
also in another listed group.

To find a group's id, open the group in [Telegram Web](https://web.telegram.org/a/). The URL ends with
the id, e.g. `web.telegram.org/a/#-1001234567890`. In the `/k/` version of Telegram Web the `-100`
prefix is missing from the URL, so add it yourself.

### Run container

```sh
docker compose up -d
```
