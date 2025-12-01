//! Authentication test helpers
//!
//! Provides utilities for creating test users, generating tokens,
//! and testing authentication flows.

#[cfg(feature = "ssr")]
use bcrypt;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
#[cfg(feature = "ssr")]
use uuid::Uuid;
#[cfg(feature = "ssr")]
use xfcollab::backend::auth::sessions::create_token;
#[cfg(feature = "ssr")]
use xfcollab::backend::auth::users::create_user;

/// Test user credentials
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password: String,
    pub token: String,
}

/// Create a test user in the database
#[cfg(feature = "ssr")]
pub async fn create_test_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<TestUser, Box<dyn std::error::Error>> {
    // Hash password
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

    // Create user
    let user = create_user(pool, email.to_string(), password_hash).await?;

    // Generate token
    let token = create_token(user.id, user.email.clone()).expect("Failed to create test token");

    Ok(TestUser {
        id: user.id.to_string(),
        email: user.email,
        password: password.to_string(),
        token,
    })
}

/// Create a test user with a unique email
#[cfg(feature = "ssr")]
pub async fn create_unique_test_user(
    pool: &PgPool,
) -> Result<TestUser, Box<dyn std::error::Error>> {
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "test_password_123";
    create_test_user(pool, &email, password).await
}

/// Generate a test JWT token
#[cfg(feature = "ssr")]
pub fn generate_test_token(user_id: Uuid, email: &str) -> String {
    create_token(user_id, email.to_string()).expect("Failed to generate test token")
}

/// Create authorization header value
pub fn auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

#[cfg(not(feature = "ssr"))]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password: String,
    pub token: String,
}

#[cfg(not(feature = "ssr"))]
pub fn create_test_user(
    _pool: &(),
    _email: &str,
    _password: &str,
) -> Result<TestUser, Box<dyn std::error::Error>> {
    Ok(TestUser {
        id: "test_id".to_string(),
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        token: "test_token".to_string(),
    })
}

#[cfg(not(feature = "ssr"))]
pub fn generate_test_token(_user_id: Uuid, _email: &str) -> String {
    "test_token".to_string()
}

#[cfg(not(feature = "ssr"))]
pub fn auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}
