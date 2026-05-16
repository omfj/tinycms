use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};

use crate::{
    auth::{generate_token, provider::github as gh},
    state::SharedState,
};

use super::{CallbackQuery, LoginOutcome, oauth_login, oauth_state_cookie, session_cookie};

pub async fn redirect(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, crate::error::Error> {
    let (client_id, _) = gh::find_github_config(&state.schema.borrow())
        .ok_or_else(|| crate::error::Error::BadRequest("GitHub provider not configured".into()))?;

    let csrf = generate_token();
    let redirect_uri = format!("{}/api/auth/github/callback", state.base_url);
    let url = format!(
        "https://github.com/login/oauth/authorize\
         ?client_id={client_id}\
         &redirect_uri={redirect_uri}\
         &state={csrf}\
         &scope=read:user,user:email"
    );

    Ok((jar.add(oauth_state_cookie(csrf)), Redirect::to(&url)))
}

pub async fn callback(
    State(state): State<SharedState>,
    Query(params): Query<CallbackQuery>,
    jar: CookieJar,
) -> Response {
    let expected = jar.get("oauth_state").map(|c| c.value().to_owned());
    let jar = jar.remove(Cookie::from("oauth_state"));

    if expected.as_deref() != params.state.as_deref() || expected.is_none() {
        return (jar, Redirect::to("/?error=invalid_state")).into_response();
    }
    if params.error.is_some() {
        return (jar, Redirect::to("/?error=access_denied")).into_response();
    }
    let Some(code) = params.code else {
        return (jar, Redirect::to("/?error=no_code")).into_response();
    };
    let Some((client_id, client_secret)) = gh::find_github_config(&state.schema.borrow()) else {
        return (jar, Redirect::to("/?error=not_configured")).into_response();
    };

    let redirect_uri = format!("{}/api/auth/github/callback", state.base_url);

    let access_token = match gh::exchange(
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
            return (jar, Redirect::to("/?error=auth_failed")).into_response();
        }
    };

    let gh_user = match gh::user(&state.http, &access_token).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("github user fetch: {e}");
            return (jar, Redirect::to("/?error=auth_failed")).into_response();
        }
    };

    let email = if let Some(e) = gh_user.email {
        e
    } else {
        match gh::primary_email(&state.http, &access_token).await {
            Ok(Some(e)) => e,
            _ => return (jar, Redirect::to("/?error=no_email")).into_response(),
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
        Ok(LoginOutcome::Active { token }) => {
            (jar.add(session_cookie(token)), Redirect::to("/")).into_response()
        }
        Ok(LoginOutcome::Pending) => (jar, Redirect::to("/?status=pending")).into_response(),
        Ok(LoginOutcome::Suspended) => (jar, Redirect::to("/?error=suspended")).into_response(),
        Err(e) => {
            tracing::error!("oauth login: {e}");
            (jar, Redirect::to("/?error=server_error")).into_response()
        }
    }
}
