# QPosit

QPosit is a prediction market platform.

## Tech stack

- Rust
- Axum for web server
- sqlx for database interaction
- askama for templating
- htmx for reactivity
- picocss for styling
- justfile for development recipes
- docker for deployment


## Run locally

```shell
sqlx migrate add --source db/migrations init
cargo run --release
```

## Deploy

```shell
sqlx migrate add --source db/migrations init

docker run -v ./db:/db basic_site
```

### Example `.env` file

```shell
MIGRATIONS_PATH=db/migrations
DATABASE_PATH=db/db.db
DATABASE_URL=sqlite:${DATABASE_PATH}
```
