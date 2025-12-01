//! Authentication API integration tests
//!
//! Tests for the authentication endpoints including login, signup, and user info.

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
    async fn test_signup_success() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let response = server
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "email": "test@example.com",
                "password": "password123"
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert!(body.get("token").is_some());
        assert!(body.get("user").is_some());
    }

    #[tokio::test]
    async fn test_signup_duplicate_email() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        // Create first user
        let _user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        // Try to create duplicate
        let response = server
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "email": "test@example.com",
                "password": "password123"
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_success() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        let response = server
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": "test@example.com",
                "password": "password123"
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert!(body.get("token").is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let response = server
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": "nonexistent@example.com",
                "password": "wrongpassword"
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_me_with_valid_token() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        let response = server
            .get("/api/auth/me")
            .add_header("Authorization", &format!("Bearer {}", user.token))
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert_eq!(body["email"], "test@example.com");
    }

    #[tokio::test]
    async fn test_get_me_without_token() {
        let server = create_test_server().await;

        let response = server.get("/api/auth/me").await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}
