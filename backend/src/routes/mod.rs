use axum::Router;
use axum::routing::get;

use crate::state::SharedState;

pub mod assets;
mod auth;
mod documents;
mod health;
mod query;
mod schema;
mod uploads;
mod users;
mod workspace;

pub fn api_router(state: SharedState) -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/documents", documents::router())
        .nest("/users", users::router())
        .nest("/workspace", workspace::router())
        .nest("/uploads", uploads::router())
        .nest("/query", query::router())
        .route("/workspace/setup", get(workspace::setup_status))
        .route("/health", get(health::health))
        .route("/schema", get(schema::get))
        .with_state(state)
}
