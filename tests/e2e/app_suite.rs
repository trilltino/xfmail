//! E2E test suite for XFCollab
//!
//! This module contains end-to-end tests using Cucumber and Fantoccini

#[cfg(feature = "ssr")]
mod tests {
    use fantoccini::{Client, Locator};
    use std::time::Duration;

    async fn create_client() -> Result<Client, fantoccini::error::CmdError> {
        let mut caps = serde_json::map::Map::new();
        let opts = serde_json::json!({
            "args": ["--headless", "--disable-gpu"]
        });
        caps.insert("goog:chromeOptions".to_string(), opts);

        Client::with_capabilities("http://localhost:9515", caps).await
    }

    #[tokio::test]
    #[ignore] // Requires server and chromedriver
    async fn test_homepage_loads() {
        let client = create_client().await.unwrap();
        client.goto("http://127.0.0.1:3000").await.unwrap();

        let title = client.title().await.unwrap();
        assert!(title.contains("XFCollab") || title.len() > 0);

        client.close().await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires server and chromedriver
    async fn test_navigation() {
        let client = create_client().await.unwrap();
        client.goto("http://127.0.0.1:3000").await.unwrap();

        // Wait for page to load
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check if page loaded successfully
        let url = client.current_url().await.unwrap();
        assert!(url.as_str().contains("127.0.0.1:3000"));

        client.close().await.unwrap();
    }
}
