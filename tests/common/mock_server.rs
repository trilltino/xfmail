//! Mock server helpers for integration tests
//!
//! Provides utilities for creating mock HTTP servers for testing
//! external API integrations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock server configuration
pub struct MockServerConfig {
    pub base_url: String,
    pub responses: Arc<RwLock<HashMap<String, MockResponse>>>,
}

/// Mock HTTP response
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl MockResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            body,
            headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

impl MockServerConfig {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_response(&self, path: String, response: MockResponse) {
        let mut responses = self.responses.write().await;
        responses.insert(path, response);
    }

    pub async fn get_response(&self, path: &str) -> Option<MockResponse> {
        let responses = self.responses.read().await;
        responses.get(path).cloned()
    }
}

/// Helper to create a mock Stripe server
pub fn create_mock_stripe_server() -> MockServerConfig {
    MockServerConfig::new("https://api.stripe.com".to_string())
}

/// Helper to create a mock AI provider server
pub fn create_mock_ai_server() -> MockServerConfig {
    MockServerConfig::new("https://api.openai.com".to_string())
}
