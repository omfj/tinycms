use anyhow::Context;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_BASE_URL: &str = "http://localhost";
const DEFAULT_CONFIG_PATH: &str = "./tinycms.config.ts";

pub struct Config {
    pub port: u16,
    pub base_url: String,
    pub config_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = Self::port_from_env()?;
        let base_url = Self::base_url_from_env(port);
        let config_path = Self::config_path_from_env();

        Ok(Self {
            port,
            base_url,
            config_path,
        })
    }

    fn port_from_env() -> anyhow::Result<u16> {
        let port_str = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
        port_str.parse().context("PORT must be a number")
    }

    fn base_url_from_env(port: u16) -> String {
        std::env::var("BASE_URL").unwrap_or_else(|_| format!("{DEFAULT_BASE_URL}:{port}"))
    }

    fn config_path_from_env() -> String {
        std::env::var("TINYCMS_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.into())
    }
}
