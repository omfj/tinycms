use serde::Deserialize;

use crate::schema::{ProviderConfig, TinyCmsConfig};

pub fn find_google_config(schema: &TinyCmsConfig) -> Option<(String, String)> {
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

#[derive(Deserialize)]
pub struct GoogleUser {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn exchange(
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

pub async fn user(http: &reqwest::Client, token: &str) -> anyhow::Result<GoogleUser> {
    Ok(http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token)
        .send()
        .await?
        .json::<GoogleUser>()
        .await?)
}
