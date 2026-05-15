use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    error::{Error, Result},
    models::user::{UpdateUser, User, UserRole, UserStatus},
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", patch(update_user))
        .route("/{id}", delete(delete_user))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<UserStatus>,
}

async fn list_users(
    _admin: AdminUser,
    State(state): State<SharedState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<User>>> {
    let users = sqlx::query_as!(
        User,
        r#"SELECT id, name, email,
                  status AS "status: UserStatus",
                  role AS "role: UserRole",
                  created_at, updated_at
           FROM users
           WHERE ($1::text IS NULL OR status = $1::text)
           ORDER BY created_at"#,
        q.status as Option<UserStatus>,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

async fn update_user(
    _admin: AdminUser,
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUser>,
) -> Result<Json<User>> {
    if body.name.is_none() && body.status.is_none() && body.role.is_none() {
        return Err(Error::BadRequest("no fields to update".into()));
    }

    let user = sqlx::query_as!(
        User,
        r#"UPDATE users SET
             name   = COALESCE($1, name),
             status = COALESCE($2::text, status)::text,
             role   = COALESCE($3::text, role)::text
           WHERE id = $4
           RETURNING id, name, email,
                     status AS "status: UserStatus",
                     role AS "role: UserRole",
                     created_at, updated_at"#,
        body.name,
        body.status as Option<UserStatus>,
        body.role as Option<UserRole>,
        id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(Json(user))
}

async fn delete_user(
    _admin: AdminUser,
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let res = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
