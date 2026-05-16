use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, generate_token},
    db::Pool,
    models::user::{UserRole, UserStatus},
    state::SharedState,
};

mod credentials;
mod github;
mod google;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/providers", get(providers))
        .route("/me", get(me))
        .route("/logout", post(logout))
        .route("/login", post(credentials::login))
        .route("/register", post(credentials::register))
        .route("/github", get(github::redirect))
        .route("/github/callback", get(github::callback))
        .route("/google", get(google::redirect))
        .route("/google/callback", get(google::callback))
}

async fn providers(State(state): State<SharedState>) -> Json<serde_json::Value> {
    use crate::auth::provider::{
        credentials_enabled, github::find_github_config, google::find_google_config,
    };
    let schema = state.schema.borrow();
    Json(json!({
        "credentials": credentials_enabled(&schema),
        "github": find_github_config(&schema).is_some(),
        "google": find_google_config(&schema).is_some(),
    }))
}

async fn me(user: AuthUser) -> Json<AuthUser> {
    Json(user)
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove(Cookie::from("session")), StatusCode::NO_CONTENT)
}

pub(super) enum LoginOutcome {
    Active { token: String },
    Pending,
    Suspended,
}

pub(super) async fn create_session(pool: &Pool, user_id: Uuid) -> anyhow::Result<String> {
    let token = generate_token();
    let expires = Utc::now() + chrono::Duration::days(30);
    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
        user_id,
        token,
        expires,
    )
    .execute(pool)
    .await?;
    Ok(token)
}

pub(super) async fn oauth_login(
    pool: &Pool,
    email: &str,
    name: Option<&str>,
    provider: &str,
    provider_account_id: &str,
) -> anyhow::Result<LoginOutcome> {
    let (new_status, new_role) = resolve_new_user_status(pool).await?;

    struct UpsertedUser {
        id: Uuid,
        status: UserStatus,
    }

    let user = sqlx::query_as!(
        UpsertedUser,
        r#"INSERT INTO users (email, name, status, role)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (email) DO UPDATE SET name = COALESCE(EXCLUDED.name, users.name)
           RETURNING id, status AS "status: UserStatus""#,
        email,
        name,
        new_status as UserStatus,
        new_role as UserRole,
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO accounts (user_id, provider, provider_account_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (provider, provider_account_id) DO NOTHING",
        user.id,
        provider,
        provider_account_id,
    )
    .execute(pool)
    .await?;

    match user.status {
        UserStatus::Active => {
            let token = create_session(pool, user.id).await?;
            Ok(LoginOutcome::Active { token })
        }
        UserStatus::Pending => Ok(LoginOutcome::Pending),
        UserStatus::Suspended => Ok(LoginOutcome::Suspended),
    }
}

pub(super) async fn resolve_new_user_status(pool: &Pool) -> anyhow::Result<(UserStatus, UserRole)> {
    struct WsRow {
        require_approval: bool,
        default_role: UserRole,
    }

    let ws = sqlx::query_as!(
        WsRow,
        r#"SELECT require_approval, default_role AS "default_role: UserRole"
           FROM workspace_settings LIMIT 1"#,
    )
    .fetch_optional(pool)
    .await?;

    let (require_approval, default_role) = ws
        .map(|r| (r.require_approval, r.default_role))
        .unwrap_or((true, UserRole::Editor));

    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    if count == 0 {
        Ok((UserStatus::Active, UserRole::Admin))
    } else if require_approval {
        Ok((UserStatus::Pending, default_role))
    } else {
        Ok((UserStatus::Active, default_role))
    }
}

pub(super) fn session_cookie(token: String) -> Cookie<'static> {
    Cookie::build(("session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .build()
}

pub(super) fn oauth_state_cookie(csrf: String) -> Cookie<'static> {
    Cookie::build(("oauth_state", csrf))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(600))
        .build()
}

#[derive(Deserialize)]
pub(super) struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}
