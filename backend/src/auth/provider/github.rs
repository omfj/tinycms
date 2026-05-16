use serde::Deserialize;

use crate::schema::{ProviderConfig, TinyCmsConfig};

pub fn find_github_config(schema: &TinyCmsConfig) -> Option<(String, String)> {
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

#[derive(Deserialize)]
pub struct GitHubUser {
    pub id: i64,
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

pub async fn user(http: &reqwest::Client, token: &str) -> anyhow::Result<GitHubUser> {
    Ok(http
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("User-Agent", "tinycms")
        .send()
        .await?
        .json::<GitHubUser>()
        .await?)
}

pub async fn primary_email(http: &reqwest::Client, token: &str) -> anyhow::Result<Option<String>> {
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
