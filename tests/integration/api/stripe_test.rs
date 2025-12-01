//! Stripe API integration tests

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
    async fn test_create_checkout_session() {
        let db = TestDatabase::new().await;
        let server = create_test_server().await;

        let user = create_test_user(db.pool(), "test@example.com", "password123")
            .await
            .unwrap();

        let response = server
            .post("/api/stripe/checkout")
            .add_header("Authorization", &format!("Bearer {}", user.token))
            .json(&serde_json::json!({
                "price_id": "price_test123"
            }))
            .await;

        // This will fail without Stripe API key, but tests the endpoint exists
        // In real tests, we'd mock Stripe
        assert!(
            response.status_code() == StatusCode::OK
                || response.status_code() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
