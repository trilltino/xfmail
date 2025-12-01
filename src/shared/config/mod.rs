//! Application configuration module
//!
//! Provides configuration types for the application.

use thiserror::Error;

/// Application configuration
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    /// Server URL
    pub server_url: Option<String>,
}

impl AppConfig {
    /// Create a new AppConfigBuilder
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

/// Builder for AppConfig
#[derive(Debug, Default)]
pub struct AppConfigBuilder {
    server_url: Option<String>,
}

impl AppConfigBuilder {
    /// Set the server URL
    pub fn server_url(mut self, url: String) -> Self {
        self.server_url = Some(url);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<AppConfig, ConfigError> {
        Ok(AppConfig {
            server_url: self.server_url,
        })
    }
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("missing value: {0}")]
    MissingValue(&'static str),
}

