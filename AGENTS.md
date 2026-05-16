# TinyCMS — Agent Guide

Self-hosted headless CMS. Single binary: Rust/axum backend with the React admin UI embedded at compile time via `rust-embed`. Postgres for storage, S3-compatible bucket for media.

## Repository layout

```
backend/          Rust (axum) HTTP server
  src/
    auth/         Session auth + OAuth providers (GitHub, Google)
    models/       DB model types (User, Document, Workspace, Media)
    routes/       Axum route handlers, one file per resource
    schema/       Config-file parsing and field type definitions
    query/        Custom query language: parser → validator → translator → executor
    config.rs     Runtime config (PORT, BASE_URL, TINYCMS_CONFIG env vars)
    state.rs      Shared app state (pool, schema watch channel, storage client)
    error.rs      Unified error type with IntoResponse impl
    storage.rs    S3-compatible storage client

frontend/         React 19 + Vite + Tailwind 4 admin UI
  src/
    routes/       Page-level components (admin, login, settings, media, …)
    components/   Reusable UI + field editors
    lib/          API client, auth context, utilities

migrations/       SQL migration files (run via sqlx-cli on startup)
packages/config/  Shared TypeScript config schema for tinycms.config.ts
examples/         blog and ecommerce reference configs
justfile          All dev tasks (see below)
```

## Tech stack

| Layer           | Tech                                                              |
| --------------- | ----------------------------------------------------------------- |
| Backend         | Rust 2024 edition, axum 0.8, sqlx 0.8, tokio                      |
| Auth            | Session cookies (axum-extra), argon2 passwords, OAuth via reqwest |
| DB              | PostgreSQL; sqlx compile-time query checking via `.sqlx/` cache   |
| Storage         | AWS SDK for S3 (also works with MinIO)                            |
| Config watching | `notify` crate; schema reloads without restart                    |
| Frontend        | React 19, React Router 7, Tailwind 4, Vite 8                      |
| Linting         | `cargo clippy` (backend), `oxlint` (frontend)                     |
| Formatting      | `cargo fmt` (backend), `oxfmt` (frontend)                         |
| Task runner     | `just`                                                            |

## Key conventions

- **Error handling**: return `crate::error::Result<T>` (alias for `Result<T, Error>`). Add variants to `Error` rather than using `anyhow` directly in handlers. `sqlx::Error::RowNotFound` maps to 404 automatically.
- **Auth**: first user to register becomes admin. Session token stored in an HTTP-only cookie. API tokens (Bearer) are also supported for programmatic access.
- **Schema**: content types are defined in `tinycms.config.ts` (TypeScript, evaluated at runtime via Deno/Node/Bun). The schema is loaded at startup and can be hot-reloaded when the file changes.
- **Database URL**: `DATABASE_URL` env var is required at compile time for `cargo sqlx prepare`. At runtime the app reads the URL from `tinycms.config.ts`.
- **Frontend embedding**: `cargo build` triggers `build.rs` which runs `pnpm build`, then `rust-embed` bakes the dist into the binary. The `assets` route serves embedded files with a SPA fallback.
- **No mock databases in tests**: always test against a real Postgres instance.

## Development workflow

```sh
# Start Postgres (+ MinIO for media)
just db

# Run the server (reads .env automatically via dotenv-load in justfile)
cargo run -- serve --watch

# Frontend dev server (proxies API to :3000)
cd frontend && pnpm dev
```

## After implementing anything — always run:

```sh
just check
```

This runs `cargo clippy --fix` + `pnpm lint:fix` (lint) and `cargo fmt` + `oxfmt` (format). Fix any remaining errors before considering the task done.

## Other useful just recipes

```sh
just migrate      # run pending SQL migrations
just prepare      # regenerate .sqlx query cache after changing sqlx queries
just build        # release build (also rebuilds frontend)
just install      # install binary to ~/.local/bin
```
