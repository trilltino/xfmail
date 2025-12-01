//! Chat API integration tests
//!
//! Tests for chat endpoints including message handling and Braid protocol

#[cfg(feature = "ssr")]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use leptos::get_configuration;
    use tests::common::auth_helpers::create_test_user;
    use tests::common::database::TestDatabase;
    use xfcollab::backend::server::create_app;
    use xfcollab::frontend::app::{shell, App};

    async fn create_test_server() -> TestServer {
        let conf =
            get_configuration(Some("Cargo.toml")).expect("Failed to get Leptos configuration");
        let app = create_app(conf.leptos_options, App, shell).await;
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_put_message() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        let response = server
            .put("/chat")
            .add_header("Authorization", &format!("Bearer {}", user.token))
            .json(&serde_json::json!({
                "text": "Hello, world!",
                "author": "test@example.com"
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_messages() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        let response = server
            .get("/chat")
            .add_header("Authorization", &format!("Bearer {}", user.token))
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
