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

pub async fn serve(cfg: config::Config, watch: bool) -> anyhow::Result<()> {
    let initial_schema = schema::load(&cfg.config_path).await?;
    let pool = db::create_pool(&initial_schema.database.url).await?;
    let storage = initial_schema
        .storage
        .clone()
        .map(storage::StorageClient::from);
    let http = reqwest::Client::new();

    tracing::info!(
        "loaded {} type(s) from {}",
        initial_schema.types.len(),
        cfg.config_path
    );
    if storage.is_some() {
        tracing::info!("storage enabled");
    } else {
        tracing::info!("storage not configured — set S3_BUCKET to enable uploads");
    }

    let (schema_tx, schema_rx) = tokio::sync::watch::channel(initial_schema);

    if watch {
        use notify::{EventKind, RecursiveMode, Watcher};

        let config_path = cfg.config_path.clone();
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel(1);
        let mut watcher = notify::RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res
                    && matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                {
                    notify_tx.blocking_send(()).ok();
                }
            },
            notify::Config::default(),
        )?;
        watcher.watch(
            std::path::Path::new(&cfg.config_path),
            RecursiveMode::NonRecursive,
        )?;
        tracing::info!("watching {} for changes", cfg.config_path);

        tokio::spawn(async move {
            let _watcher = watcher;
            while notify_rx.recv().await.is_some() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                while notify_rx.try_recv().is_ok() {}
                match schema::load(&config_path).await {
                    Ok(new_schema) => {
                        tracing::info!(
                            "config reloaded: {} type(s) from {}",
                            new_schema.types.len(),
                            config_path
                        );
                        schema_tx.send(new_schema).ok();
                    }
                    Err(e) => {
                        tracing::warn!("config reload failed, keeping previous config: {e:#}");
                    }
                }
            }
        });
    }

    let state: state::SharedState = std::sync::Arc::new(state::AppState {
        pool,
        schema: schema_rx,
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
