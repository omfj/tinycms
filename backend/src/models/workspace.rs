use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user::UserRole;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkspaceSettings {
    pub id: Uuid,
    pub name: String,
    pub require_approval: bool,
    pub default_role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateWorkspaceSettings {
    pub name: Option<String>,
    pub require_approval: Option<bool>,
    pub default_role: Option<UserRole>,
}
