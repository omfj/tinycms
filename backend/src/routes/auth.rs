use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, clear_session_cookie, extract_cookie, generate_token, session_cookie},
    db::Pool,
    error::{Error, Result},
    models::user::{UserRole, UserStatus},
    schema::{ProviderConfig, TinyCmsConfig},
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/providers", get(providers))
        .route("/me", get(me))
        .route("/logout", post(logout))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/github", get(github_redirect))
        .route("/github/callback", get(github_callback))
        .route("/google", get(google_redirect))
        .route("/google/callback", get(google_callback))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn providers(State(state): State<SharedState>) -> Json<serde_json::Value> {
    Json(json!({
        "credentials": credentials_enabled(&state.schema),
        "github": find_github_config(&state.schema).is_some(),
        "google": find_google_config(&state.schema).is_some(),
    }))
}

async fn me(user: AuthUser) -> Json<AuthUser> {
    Json(user)
}

async fn logout() -> impl IntoResponse {
    (
        [(SET_COOKIE, clear_session_cookie())],
        StatusCode::NO_CONTENT,
    )
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login(
    State(state): State<SharedState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response> {
    if !credentials_enabled(&state.schema) {
        return Err(Error::BadRequest(
            "credentials provider not configured".into(),
        ));
    }

    struct Row {
        id: Uuid,
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

    let (token, expires) = create_session(&state.pool, row.id).await?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie(&token, expires).parse().unwrap());
    Ok((StatusCode::OK, headers, Json(json!({ "ok": true }))).into_response())
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: Option<String>,
}

async fn register(
    State(state): State<SharedState>,
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

    let user_id: Uuid = sqlx::query_scalar!(
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

    // Re-read the actual status in case the user already existed with a different status.
    let actual_status: UserStatus = sqlx::query_scalar!(
        r#"SELECT status AS "status: UserStatus" FROM users WHERE id = $1"#,
        user_id,
    )
    .fetch_one(&state.pool)
    .await?;

    if actual_status == UserStatus::Pending {
        return Ok((StatusCode::ACCEPTED, Json(json!({ "pending": true }))).into_response());
    }

    let (token, expires) = create_session(&state.pool, user_id).await?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie(&token, expires).parse().unwrap());
    Ok((StatusCode::CREATED, headers, Json(json!({ "ok": true }))).into_response())
}

// ---------------------------------------------------------------------------
// GitHub OAuth
// ---------------------------------------------------------------------------

async fn github_redirect(State(state): State<SharedState>) -> Result<impl IntoResponse> {
    let (client_id, _) = find_github_config(&state.schema)
        .ok_or_else(|| Error::BadRequest("GitHub provider not configured".into()))?;

    let csrf = generate_token();
    let redirect_uri = format!("{}/api/auth/github/callback", state.base_url);
    let url = format!(
        "https://github.com/login/oauth/authorize\
         ?client_id={client_id}\
         &redirect_uri={redirect_uri}\
         &state={csrf}\
         &scope=read:user,user:email"
    );

    let state_cookie = format!("oauth_state={csrf}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600");
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, state_cookie.parse().unwrap());

    Ok((headers, Redirect::to(&url)))
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn github_callback(
    State(state): State<SharedState>,
    Query(params): Query<CallbackQuery>,
    req_headers: HeaderMap,
) -> Response {
    let clear_state = "oauth_state=; Path=/; HttpOnly; Max-Age=0";

    let expected = extract_cookie(&req_headers, "oauth_state");
    if expected.as_deref() != params.state.as_deref() || expected.is_none() {
        return redirect_with_cookie("/?error=invalid_state", clear_state);
    }
    if params.error.is_some() {
        return redirect_with_cookie("/?error=access_denied", clear_state);
    }
    let code = match &params.code {
        Some(c) => c.clone(),
        None => return redirect_with_cookie("/?error=no_code", clear_state),
    };

    let Some((client_id, client_secret)) = find_github_config(&state.schema) else {
        return redirect_with_cookie("/?error=not_configured", clear_state);
    };

    let redirect_uri = format!("{}/api/auth/github/callback", state.base_url);

    let access_token = match github_exchange(
        &state.http,
        &client_id,
        &client_secret,
        &code,
        &redirect_uri,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("github token exchange: {e}");
            return redirect_with_cookie("/?error=auth_failed", clear_state);
        }
    };

    let gh_user = match github_user(&state.http, &access_token).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("github user fetch: {e}");
            return redirect_with_cookie("/?error=auth_failed", clear_state);
        }
    };

    let email = if let Some(e) = gh_user.email {
        e
    } else {
        match github_primary_email(&state.http, &access_token).await {
            Ok(Some(e)) => e,
            _ => return redirect_with_cookie("/?error=no_email", clear_state),
        }
    };

    let provider_id = gh_user.id.to_string();
    match oauth_login(
        &state.pool,
        &email,
        gh_user.name.as_deref(),
        "github",
        &provider_id,
    )
    .await
    {
        Ok(LoginOutcome::Active { token, expires }) => {
            let mut resp = Redirect::to("/").into_response();
            resp.headers_mut()
                .insert(SET_COOKIE, session_cookie(&token, expires).parse().unwrap());
            resp.headers_mut()
                .append(SET_COOKIE, clear_state.parse().unwrap());
            resp
        }
        Ok(LoginOutcome::Pending) => redirect_with_cookie("/?status=pending", clear_state),
        Ok(LoginOutcome::Suspended) => redirect_with_cookie("/?error=suspended", clear_state),
        Err(e) => {
            tracing::error!("oauth login: {e}");
            redirect_with_cookie("/?error=server_error", clear_state)
        }
    }
}

// ---------------------------------------------------------------------------
// Google OAuth
// ---------------------------------------------------------------------------

async fn google_redirect(State(state): State<SharedState>) -> Result<impl IntoResponse> {
    let (client_id, _) = find_google_config(&state.schema)
        .ok_or_else(|| Error::BadRequest("Google provider not configured".into()))?;

    let csrf = generate_token();
    let redirect_uri = format!("{}/api/auth/google/callback", state.base_url);
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
         ?client_id={client_id}\
         &redirect_uri={redirect_uri}\
         &response_type=code\
         &scope=openid%20email%20profile\
         &state={csrf}"
    );

    let state_cookie = format!("oauth_state={csrf}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600");
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, state_cookie.parse().unwrap());

    Ok((headers, Redirect::to(&url)))
}

async fn google_callback(
    State(state): State<SharedState>,
    Query(params): Query<CallbackQuery>,
    req_headers: HeaderMap,
) -> Response {
    let clear_state = "oauth_state=; Path=/; HttpOnly; Max-Age=0";

    let expected = extract_cookie(&req_headers, "oauth_state");
    if expected.as_deref() != params.state.as_deref() || expected.is_none() {
        return redirect_with_cookie("/?error=invalid_state", clear_state);
    }
    if params.error.is_some() {
        return redirect_with_cookie("/?error=access_denied", clear_state);
    }
    let code = match &params.code {
        Some(c) => c.clone(),
        None => return redirect_with_cookie("/?error=no_code", clear_state),
    };

    let Some((client_id, client_secret)) = find_google_config(&state.schema) else {
        return redirect_with_cookie("/?error=not_configured", clear_state);
    };

    let redirect_uri = format!("{}/api/auth/google/callback", state.base_url);

    let access_token = match google_exchange(
        &state.http,
        &client_id,
        &client_secret,
        &code,
        &redirect_uri,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("google token exchange: {e}");
            return redirect_with_cookie("/?error=auth_failed", clear_state);
        }
    };

    let g_user = match google_user(&state.http, &access_token).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("google user fetch: {e}");
            return redirect_with_cookie("/?error=auth_failed", clear_state);
        }
    };

    let email = match g_user.email {
        Some(e) => e,
        None => return redirect_with_cookie("/?error=no_email", clear_state),
    };

    match oauth_login(
        &state.pool,
        &email,
        g_user.name.as_deref(),
        "google",
        &g_user.sub,
    )
    .await
    {
        Ok(LoginOutcome::Active { token, expires }) => {
            let mut resp = Redirect::to("/").into_response();
            resp.headers_mut()
                .insert(SET_COOKIE, session_cookie(&token, expires).parse().unwrap());
            resp.headers_mut()
                .append(SET_COOKIE, clear_state.parse().unwrap());
            resp
        }
        Ok(LoginOutcome::Pending) => redirect_with_cookie("/?status=pending", clear_state),
        Ok(LoginOutcome::Suspended) => redirect_with_cookie("/?error=suspended", clear_state),
        Err(e) => {
            tracing::error!("oauth login: {e}");
            redirect_with_cookie("/?error=server_error", clear_state)
        }
    }
}

// ---------------------------------------------------------------------------
// Shared OAuth helpers
// ---------------------------------------------------------------------------

enum LoginOutcome {
    Active {
        token: String,
        expires: chrono::DateTime<Utc>,
    },
    Pending,
    Suspended,
}

async fn oauth_login(
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
            let (token, expires) = create_session(pool, user.id).await?;
            Ok(LoginOutcome::Active { token, expires })
        }
        UserStatus::Pending => Ok(LoginOutcome::Pending),
        UserStatus::Suspended => Ok(LoginOutcome::Suspended),
    }
}

/// Determines status and role for a brand-new user based on workspace settings.
/// First user in the system always becomes admin + active.
async fn resolve_new_user_status(pool: &Pool) -> anyhow::Result<(UserStatus, UserRole)> {
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

async fn create_session(
    pool: &Pool,
    user_id: Uuid,
) -> anyhow::Result<(String, chrono::DateTime<Utc>)> {
    let token = generate_token();
    let expires = Utc::now() + Duration::days(30);
    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
        user_id,
        token,
        expires,
    )
    .execute(pool)
    .await?;
    Ok((token, expires))
}

fn redirect_with_cookie(location: &str, cookie: &str) -> Response {
    let mut resp = Redirect::to(location).into_response();
    resp.headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());
    resp
}

fn verify_password(password: &str, hash: &str) -> Result<()> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed = PasswordHash::new(hash).map_err(|_| Error::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| Error::Unauthorized)
}

// ---------------------------------------------------------------------------
// Provider config lookup
// ---------------------------------------------------------------------------

fn credentials_enabled(schema: &TinyCmsConfig) -> bool {
    schema
        .auth
        .as_ref()
        .map(|a| {
            a.providers
                .iter()
                .any(|p| matches!(p, ProviderConfig::Credentials))
        })
        .unwrap_or(false)
}

fn find_github_config(schema: &TinyCmsConfig) -> Option<(String, String)> {
    schema.auth.as_ref()?.providers.iter().find_map(|p| {
        if let ProviderConfig::GitHub {
            client_id,
            client_secret,
        } = p
        {
            Some((client_id.clone()?, client_secret.clone()?))
        } else {
            None
        }
    })
}

fn find_google_config(schema: &TinyCmsConfig) -> Option<(String, String)> {
    schema.auth.as_ref()?.providers.iter().find_map(|p| {
        if let ProviderConfig::Google {
            client_id,
            client_secret,
        } = p
        {
            Some((client_id.clone()?, client_secret.clone()?))
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------------------
// GitHub API
// ---------------------------------------------------------------------------

async fn github_exchange(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }
    let resp = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;
    Ok(resp.access_token)
}

#[derive(Deserialize)]
struct GitHubUser {
    id: i64,
    name: Option<String>,
    email: Option<String>,
}

async fn github_user(http: &reqwest::Client, token: &str) -> anyhow::Result<GitHubUser> {
    let user = http
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("User-Agent", "tinycms")
        .send()
        .await?
        .json::<GitHubUser>()
        .await?;
    Ok(user)
}

async fn github_primary_email(
    http: &reqwest::Client,
    token: &str,
) -> anyhow::Result<Option<String>> {
    #[derive(Deserialize)]
    struct EmailEntry {
        email: String,
        primary: bool,
        verified: bool,
    }
    let emails: Vec<EmailEntry> = http
        .get("https://api.github.com/user/emails")
        .bearer_auth(token)
        .header("User-Agent", "tinycms")
        .send()
        .await?
        .json()
        .await?;
    Ok(emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email))
}

// ---------------------------------------------------------------------------
// Google API
// ---------------------------------------------------------------------------

async fn google_exchange(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }
    let resp = http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;
    Ok(resp.access_token)
}

#[derive(Deserialize)]
struct GoogleUser {
    sub: String,
    name: Option<String>,
    email: Option<String>,
}

async fn google_user(http: &reqwest::Client, token: &str) -> anyhow::Result<GoogleUser> {
    let user = http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token)
        .send()
        .await?
        .json::<GoogleUser>()
        .await?;
    Ok(user)
}
