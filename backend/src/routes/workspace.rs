use axum::{Json, Router, extract::State, routing::get};
use serde_json::json;

use crate::{
    auth::{AdminUser, AuthUser},
    error::Result,
    models::{
        user::UserRole,
        workspace::{UpdateWorkspaceSettings, WorkspaceSettings},
    },
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new().route("/", get(get_settings).patch(update_settings))
}

async fn get_settings(
    _user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<WorkspaceSettings>> {
    let settings = sqlx::query_as!(
        WorkspaceSettings,
        r#"SELECT id, name, require_approval, default_role AS "default_role: UserRole",
                  created_at, updated_at
           FROM workspace_settings LIMIT 1"#,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(settings))
}

async fn update_settings(
    _admin: AdminUser,
    State(state): State<SharedState>,
    Json(body): Json<UpdateWorkspaceSettings>,
) -> Result<Json<WorkspaceSettings>> {
    let settings =
        if body.name.is_none() && body.require_approval.is_none() && body.default_role.is_none() {
            sqlx::query_as!(
                WorkspaceSettings,
                r#"SELECT id, name, require_approval, default_role AS "default_role: UserRole",
                      created_at, updated_at
               FROM workspace_settings LIMIT 1"#,
            )
            .fetch_one(&state.pool)
            .await?
        } else {
            sqlx::query_as!(
                WorkspaceSettings,
                r#"UPDATE workspace_settings SET
                 name             = COALESCE($1, name),
                 require_approval = COALESCE($2, require_approval),
                 default_role     = COALESCE($3::text, default_role)::text
               RETURNING id, name, require_approval, default_role AS "default_role: UserRole",
                         created_at, updated_at"#,
                body.name,
                body.require_approval,
                body.default_role as Option<UserRole>,
            )
            .fetch_one(&state.pool)
            .await?
        };

    Ok(Json(settings))
}

pub async fn setup_status(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(None)
        .unwrap_or(0);
    Json(json!({ "initialized": count > 0 }))
}
