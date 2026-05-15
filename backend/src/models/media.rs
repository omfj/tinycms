use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Media {
    pub id: Uuid,
    pub key: String,
    pub url: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub label: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMedia {
    pub label: Option<String>,
}
