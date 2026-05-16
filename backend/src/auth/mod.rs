use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::CookieJar;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::Error,
    models::user::{UserRole, UserStatus},
    state::SharedState,
};

pub mod provider;
mod token;

pub use token::{generate_token, hash_token};

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
}

#[derive(Debug, Clone)]
pub struct AdminUser(#[allow(dead_code)] pub AuthUser);

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    email: String,
    name: Option<String>,
    role: UserRole,
    status: UserStatus,
}

impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        if let Some(token) = jar.get("session").map(|c| c.value().to_owned())
            && let Some(row) = sqlx::query_as!(
                SessionRow,
                r#"SELECT u.id, u.email, u.name,
                          u.role AS "role: UserRole",
                          u.status AS "status: UserStatus"
                   FROM sessions s
                   JOIN users u ON u.id = s.user_id
                   WHERE s.token = $1 AND s.expires_at > now()"#,
                token
            )
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| Error::Internal(e.into()))?
            && row.status == UserStatus::Active
        {
            return Ok(AuthUser {
                id: row.id,
                email: row.email,
                name: row.name,
                role: row.role,
                status: row.status,
            });
        }

        // Try Bearer token
        if let Some(auth_header) = parts.headers.get("authorization")
            && let Ok(auth_str) = auth_header.to_str()
            && let Some(raw_token) = auth_str.strip_prefix("Bearer ")
        {
            let raw_token = raw_token.trim();
            let token_hash = hash_token(raw_token);

            #[derive(sqlx::FromRow)]
            struct ApiTokenRow {
                token_id: Uuid,
                id: Uuid,
                email: String,
                name: Option<String>,
                role: UserRole,
                status: UserStatus,
            }

            let row = sqlx::query_as!(
                ApiTokenRow,
                r#"SELECT t.id AS token_id, u.id, u.email, u.name,
                                  u.role AS "role: UserRole",
                                  u.status AS "status: UserStatus"
                           FROM api_tokens t
                           JOIN users u ON u.id = t.user_id
                           WHERE t.token_hash = $1
                             AND (t.expires_at IS NULL OR t.expires_at > now())"#,
                token_hash
            )
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| Error::Internal(e.into()))?;

            match row {
                None => {}
                Some(row) if row.status != UserStatus::Active => {}
                Some(row) => {
                    let _ = sqlx::query!(
                        "UPDATE api_tokens SET last_used_at = now() WHERE id = $1",
                        row.token_id
                    )
                    .execute(&state.pool)
                    .await;

                    return Ok(AuthUser {
                        id: row.id,
                        email: row.email,
                        name: row.name,
                        role: row.role,
                        status: row.status,
                    });
                }
            }
        }

        Err(Error::Unauthorized)
    }
}

impl FromRequestParts<SharedState> for AdminUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Admin {
            return Err(Error::Forbidden);
        }
        Ok(AdminUser(user))
    }
}
