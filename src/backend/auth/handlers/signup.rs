/**
 * Signup Handler
 * 
 * This module implements the user registration handler for POST /api/auth/signup.
 * 
 * # Registration Process
 * 
 * 1. Validate email format and password length
 * 2. Check if user already exists
 * 3. Hash password using bcrypt
 * 4. Create user in database
 * 5. Generate JWT token
 * 6. Return token and user info
 * 
 * # Validation
 * 
 * - Email must contain '@' character (basic validation)
 * - Password must be at least 8 characters long
 * - Email must be unique (no existing user with same email)
 * 
 * # Security
 * 
 * - Passwords are hashed using bcrypt with DEFAULT_COST
 * - Passwords are never returned in responses
 * - JWT tokens are generated with 30-day expiration
 */

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
#[cfg(feature = "ssr")]
use bcrypt::{hash, DEFAULT_COST};
#[cfg(feature = "ssr")]
use sqlx::PgPool;

use crate::backend::auth::users::{create_user, get_user_by_email, get_user_by_username};
use crate::backend::auth::sessions::create_token;
use crate::backend::auth::handlers::types::{SignupRequest, AuthResponse, UserResponse};

/// Validate username format
///
/// Usernames must be:
/// - 3-30 characters long
/// - Contain only alphanumeric characters and underscores
/// - Start with a letter
fn is_valid_username(username: &str) -> bool {
    if username.len() < 3 || username.len() > 30 {
        return false;
    }

    let mut chars = username.chars();

    // First character must be a letter
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }

    // Rest can be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Sign up handler
/// 
/// This handler processes user registration requests. It validates the input,
/// creates a new user account, and returns a JWT token for immediate authentication.
/// 
/// # Arguments
/// 
/// * `State(pool)` - Database connection pool
/// * `Json(request)` - Signup request containing email and password
/// 
/// # Returns
/// 
/// JSON response with JWT token and user info, or an error status code
/// 
/// # Errors
/// 
/// * `400 Bad Request` - If email format is invalid or password is too short
/// * `409 Conflict` - If user with this email already exists
/// * `503 Service Unavailable` - If database is not configured
/// * `500 Internal Server Error` - If password hashing, user creation, or token generation fails
/// 
/// # Example Request
/// 
/// ```http
/// POST /api/auth/signup HTTP/1.1
/// Content-Type: application/json
/// 
/// {
///   "email": "user@example.com",
///   "password": "securepassword123"
/// }
/// ```
/// 
/// # Example Response
/// 
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "user": {
///     "id": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "subscription_status": null
///   }
/// }
/// ```
#[cfg(feature = "ssr")]
pub async fn signup(
    State(pool): State<Option<PgPool>>,
    Json(request): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let pool = pool.ok_or_else(|| {
        tracing::error!("Database not configured");
        (StatusCode::SERVICE_UNAVAILABLE, "Database not configured".to_string())
    })?;
    tracing::info!("Signup request for username: {}, email: {}", request.username, request.email);

    // Validate username format
    if !is_valid_username(&request.username) {
        tracing::warn!("Invalid username format: {}", request.username);
        return Err((StatusCode::BAD_REQUEST, "Username must be 3-30 chars, start with a letter, and contain only letters, numbers, and underscores".to_string()));
    }

    // Validate email format (basic check)
    if !request.email.contains('@') {
        tracing::warn!("Invalid email format: {}", request.email);
        return Err((StatusCode::BAD_REQUEST, "Invalid email format".to_string()));
    }

    // Validate password length
    if request.password.len() < 8 {
        tracing::warn!("Password too short");
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 8 characters".to_string()));
    }

    // Check if username already exists
    if let Ok(Some(_)) = get_user_by_username(&pool, &request.username).await {
        tracing::warn!("Username already exists: {}", request.username);
        return Err((StatusCode::CONFLICT, "Username already taken".to_string()));
    }

    // Check if email already exists
    if let Ok(Some(_)) = get_user_by_email(&pool, &request.email).await {
        tracing::warn!("Email already exists: {}", request.email);
        return Err((StatusCode::CONFLICT, "Email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Server error".to_string())
        })?;

    // Create user
    let user = create_user(&pool, request.username.clone(), request.email.clone(), password_hash)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string())
        })?;

    // Create token
    let token = create_token(user.id, user.email.clone())
        .map_err(|e| {
            tracing::error!("Failed to create token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Server error".to_string())
        })?;

    tracing::info!("User created successfully: {} ({})", user.username, user.email);

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            subscription_status: user.subscription_status,
        },
    }))
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;
    use axum::extract::State;
    use tests::common::database::TestDatabase;

    #[tokio::test]
    async fn test_signup_success() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let request = SignupRequest {
            email: "newuser@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = signup(State(Some(pool.clone())), Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, "newuser@example.com");
    }

    #[tokio::test]
    async fn test_signup_invalid_email() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let request = SignupRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        
        let result = signup(State(Some(pool.clone())), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_signup_short_password() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let request = SignupRequest {
            email: "user@example.com".to_string(),
            password: "short".to_string(),
        };
        
        let result = signup(State(Some(pool.clone())), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_signup_duplicate_email() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        // Create first user
        let request1 = SignupRequest {
            email: "duplicate@example.com".to_string(),
            password: "password123".to_string(),
        };
        let _ = signup(State(Some(pool.clone())), Json(request1)).await;
        
        // Try to create duplicate
        let request2 = SignupRequest {
            email: "duplicate@example.com".to_string(),
            password: "password123".to_string(),
        };
        let result = signup(State(Some(pool.clone())), Json(request2)).await;
        assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_signup_no_database() {
        let request = SignupRequest {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = signup(State(None), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

