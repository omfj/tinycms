pub mod github;
pub mod google;

use crate::schema::{ProviderConfig, TinyCmsConfig};

pub fn credentials_enabled(schema: &TinyCmsConfig) -> bool {
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
