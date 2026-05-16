use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub doc_type: String,
    pub status: String,
    pub data: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub status: Option<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocument {
    pub status: Option<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentRevision {
    pub id: Uuid,
    pub document_id: Uuid,
    pub data: Value,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}
