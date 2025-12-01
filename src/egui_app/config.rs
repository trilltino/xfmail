use crate::shared::config::{AppConfig, AppConfigBuilder, ConfigError};

/// Default server URL
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000";

/// Application configuration wrapper.
#[derive(Debug, Clone)]
pub struct Config {
    app: AppConfig,
    token: Option<String>,
    dev_auth_bypass: bool,
    dev_user_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let server_url = std::env::var("CLIENT_API_URL")
            .unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());
        let app = AppConfig::builder()
            .server_url(server_url)
            .build()
            .expect("default app config is valid");
        let dev_auth_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
        let dev_user_id = std::env::var("DEV_USER_ID").ok();
        Self { app, token: None, dev_auth_bypass, dev_user_id }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builder(builder: AppConfigBuilder) -> Result<Self, ConfigError> {
        let app = builder.build()?;
        let dev_auth_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
        let dev_user_id = std::env::var("DEV_USER_ID").ok();
        Ok(Self { app, token: None, dev_auth_bypass, dev_user_id })
    }

    /// Set the JWT token
    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    /// Get the JWT token
    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    /// Clear the token (logout)
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Get the full URL for an API endpoint
    pub fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.server_url(), path)
    }

    pub fn server_url(&self) -> &str {
        self.app.server_url.as_deref().unwrap_or(DEFAULT_SERVER_URL)
    }

    /// Whether to use development auth bypass
    pub fn dev_auth_bypass(&self) -> bool {
        self.dev_auth_bypass
    }

    /// Development user id used when bypassing auth
    pub fn dev_user_id(&self) -> Option<&str> {
        self.dev_user_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.server_url(), "http://127.0.0.1:3000");
        assert!(config.get_token().is_none());
    }

    #[test]
    fn test_set_token() {
        let mut config = Config::new();
        config.set_token(Some("test_token".to_string()));
        assert_eq!(config.get_token(), Some(&"test_token".to_string()));
    }

    #[test]
    fn test_clear_token() {
        let mut config = Config::new();
        config.set_token(Some("test_token".to_string()));
        config.clear_token();
        assert!(config.get_token().is_none());
    }

    #[test]
    fn test_api_url() {
        let config = Config::new();
        let url = config.api_url("/api/auth/login");
        assert_eq!(url, "http://127.0.0.1:3000/api/auth/login");
    }
}
