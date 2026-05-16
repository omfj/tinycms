set dotenv-load

db:
    docker compose up -d postgres

db-stop:
    docker compose down

lint:
    cargo clippy --fix --allow-dirty --tests && pnpm lint:fix

fmt:
    cargo fmt && pnpm fmt

check: lint fmt

# cargo build triggers build.rs which runs pnpm build automatically
build:
    cargo build --release

install: build
    cp target/release/tinycms ~/.local/bin/tinycms

# requires: cargo install sqlx-cli --no-default-features --features rustls,postgres

migrate:
    sqlx migrate run --source migrations --database-url "$DATABASE_URL"

prepare:
    cargo sqlx prepare --workspace
