# TinyCMS

Self-hosted headless CMS. Single binary - Rust backend with the React admin UI embedded inside. Postgres for storage, S3-compatible object store for media.

## How it works

You define your content types in a `tinycms.config.ts` file alongside your project. TinyCMS reads that config at startup and generates a REST API and admin UI for those types. No code generation, no build step on your end.

## Requirements

- Rust (for building from source)
- The SQLx CLI
- Node.js, Deno, or Bun
- PostgreSQL
- An S3-compatible bucket for media

## Quickstart

```sh
# 1. Start Postgres (and MinIO for local media uploads)
docker compose up -d

# 2. Copy and fill in env
cp .env.example .env

# 3. Create a `tinycms.config.ts`
# - Add you fields
# - Configure storage

# 4. Start the server (migrations run automatically on first boot)
tinycms serve
```

Open `http://localhost:3000`. The first user to sign up becomes admin.

## Examples

- [`examples/blog`](examples/blog) — blog with posts, authors, categories; GitHub OAuth
- [`examples/ecommerce`](examples/ecommerce) — products, categories, collections; credentials auth
