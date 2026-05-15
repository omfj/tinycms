use serde::{Deserialize, Serialize};

use super::field::TypeDef;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TinyCmsConfig {
    pub database: DatabaseConfig,
    pub auth: Option<AuthConfig>,
    pub storage: Option<StorageConfig>,
    pub runtime: Option<String>,
    #[serde(default)]
    pub types: Vec<TypeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "provider", rename_all = "camelCase")]
pub enum ProviderConfig {
    #[serde(rename = "github")]
    GitHub {
        #[serde(rename = "clientId")]
        client_id: Option<String>,
        #[serde(rename = "clientSecret")]
        client_secret: Option<String>,
    },
    #[serde(rename = "google")]
    Google {
        #[serde(rename = "clientId")]
        client_id: Option<String>,
        #[serde(rename = "clientSecret")]
        client_secret: Option<String>,
    },
    #[serde(rename = "credentials")]
    Credentials,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub bucket: String,
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub endpoint: Option<String>,
}
