use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, generate_token, hash_token},
    error::{Error, Result},
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_tokens).post(create_token))
        .route("/{id}", delete(delete_token))
}

#[derive(Serialize, sqlx::FromRow)]
struct ApiToken {
    id: Uuid,
    name: String,
    expires_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    name: String,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    #[serde(flatten)]
    token: ApiToken,
    raw_token: String,
}

async fn list_tokens(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<ApiToken>>> {
    let tokens = sqlx::query_as!(
        ApiToken,
        "SELECT id, name, expires_at, last_used_at, created_at
         FROM api_tokens WHERE user_id = $1 ORDER BY created_at DESC",
        user.id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(tokens))
}

async fn create_token(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(body): Json<CreateTokenRequest>,
) -> Result<Json<CreateTokenResponse>> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(Error::BadRequest("name is required".into()));
    }

    let raw_token = generate_token();
    let token_hash = hash_token(&raw_token);

    let token = sqlx::query_as!(
        ApiToken,
        "INSERT INTO api_tokens (user_id, name, token_hash, expires_at)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, expires_at, last_used_at, created_at",
        user.id,
        name,
        token_hash,
        body.expires_at
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(CreateTokenResponse { token, raw_token }))
}

async fn delete_token(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM api_tokens WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
