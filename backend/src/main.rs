use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod cli;
mod config;
mod db;
mod error;
mod models;
mod query;
mod routes;
mod schema;
mod state;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tinycms=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    cli::run().await
}

pub async fn serve(cfg: config::Config) -> anyhow::Result<()> {
    let schema = schema::load(&cfg.config_path).await?;
    let pool = db::create_pool(&schema.database.url).await?;
    let storage = schema.storage.clone().map(storage::StorageClient::from);
    let http = reqwest::Client::new();

    tracing::info!(
        "loaded {} type(s) from {}",
        schema.types.len(),
        cfg.config_path
    );
    if storage.is_some() {
        tracing::info!("storage enabled");
    } else {
        tracing::info!("storage not configured — set S3_BUCKET to enable uploads");
    }

    let state: state::SharedState = std::sync::Arc::new(state::AppState {
        pool,
        schema,
        storage,
        http,
        base_url: cfg.base_url.clone(),
    });

    let app = axum::Router::new()
        .nest("/api", routes::api_router(state))
        .fallback(routes::assets::handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.port)).await?;
    tracing::info!("listening on http://0.0.0.0:{}", cfg.port);
    axum::serve(listener, app).await?;

    Ok(())
}
