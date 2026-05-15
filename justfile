set dotenv-load

db:
    docker compose up -d postgres

db-stop:
    docker compose down

backend:
    cd backend && cargo run

frontend:
    pnpm dev

lint:
    pnpm lint

fmt:
    pnpm fmt

# Build frontend only — cargo build runs this automatically via build.rs
build-frontend:
    pnpm install && pnpm build

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
