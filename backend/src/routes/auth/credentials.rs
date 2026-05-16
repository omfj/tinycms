use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::provider::credentials_enabled,
    error::{Error, Result},
    models::user::{UserRole, UserStatus},
    state::SharedState,
};

use super::{create_session, resolve_new_user_status, session_cookie};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<Response> {
    if !credentials_enabled(&state.schema) {
        return Err(Error::BadRequest(
            "credentials provider not configured".into(),
        ));
    }

    struct Row {
        id: uuid::Uuid,
        status: UserStatus,
        access_token: Option<String>,
    }

    let row = sqlx::query_as!(
        Row,
        r#"SELECT u.id, u.status AS "status: UserStatus", a.access_token
           FROM accounts a
           JOIN users u ON u.id = a.user_id
           WHERE a.provider = 'credentials' AND a.provider_account_id = $1"#,
        body.email,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::Unauthorized)?;

    if row.status == UserStatus::Suspended {
        return Err(Error::Unauthorized);
    }

    let hash = row.access_token.ok_or(Error::Unauthorized)?;
    verify_password(&body.password, &hash)?;

    if row.status == UserStatus::Pending {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account pending approval" })),
        )
            .into_response());
    }

    let token = create_session(&state.pool, row.id).await?;
    Ok((
        StatusCode::OK,
        jar.add(session_cookie(token)),
        Json(json!({ "ok": true })),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    name: Option<String>,
}

pub async fn register(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(body): Json<RegisterRequest>,
) -> Result<Response> {
    if !credentials_enabled(&state.schema) {
        return Err(Error::BadRequest(
            "credentials provider not configured".into(),
        ));
    }

    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| Error::Internal(anyhow::anyhow!("password hashing failed: {e}")))?
        .to_string();

    let (status, role) = resolve_new_user_status(&state.pool).await?;

    let user_id: uuid::Uuid = sqlx::query_scalar!(
        "INSERT INTO users (email, name, status, role)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (email) DO UPDATE SET name = COALESCE(EXCLUDED.name, users.name)
         RETURNING id",
        body.email,
        body.name,
        status as UserStatus,
        role as UserRole,
    )
    .fetch_one(&state.pool)
    .await?;

    sqlx::query!(
        "INSERT INTO accounts (user_id, provider, provider_account_id, access_token)
         VALUES ($1, 'credentials', $2, $3)
         ON CONFLICT (provider, provider_account_id) DO UPDATE SET access_token = EXCLUDED.access_token",
        user_id,
        body.email,
        hash,
    )
    .execute(&state.pool)
    .await?;

    let actual_status: UserStatus = sqlx::query_scalar!(
        r#"SELECT status AS "status: UserStatus" FROM users WHERE id = $1"#,
        user_id,
    )
    .fetch_one(&state.pool)
    .await?;

    if actual_status == UserStatus::Pending {
        return Ok((StatusCode::ACCEPTED, Json(json!({ "pending": true }))).into_response());
    }

    let token = create_session(&state.pool, user_id).await?;
    Ok((
        StatusCode::CREATED,
        jar.add(session_cookie(token)),
        Json(json!({ "ok": true })),
    )
        .into_response())
}

fn verify_password(password: &str, hash: &str) -> Result<()> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed = PasswordHash::new(hash).map_err(|_| Error::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| Error::Unauthorized)
}
