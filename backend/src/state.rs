use std::sync::Arc;

use crate::{db::Pool, schema::TinyCmsConfig, storage::StorageClient};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub schema: TinyCmsConfig,
    pub storage: Option<StorageClient>,
    pub http: reqwest::Client,
    pub base_url: String,
}

pub type SharedState = Arc<AppState>;
