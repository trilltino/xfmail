/**
 * Login Handler
 * 
 * This module implements the user authentication handler for POST /api/auth/login.
 * 
 * # Authentication Process
 * 
 * 1. Look up user by email
 * 2. Verify password using bcrypt
 * 3. Generate JWT token
 * 4. Return token and user info
 * 
 * # Security
 * 
 * - Passwords are verified using bcrypt
 * - Invalid credentials return 401 Unauthorized (no information leakage)
 * - JWT tokens are generated with 30-day expiration
 * - User passwords are never returned in responses
 */
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
#[cfg(feature = "ssr")]
use bcrypt::verify;
#[cfg(feature = "ssr")]
use sqlx::PgPool;

use crate::backend::auth::users::{get_user_by_email, get_user_by_username};
use crate::backend::auth::sessions::create_token;
use crate::backend::auth::handlers::types::{LoginRequest, AuthResponse, UserResponse};

/// Login handler
/// 
/// This handler processes user authentication requests. It verifies the
/// email and password, and returns a JWT token if authentication succeeds.
/// 
/// # Arguments
/// 
/// * `State(pool)` - Database connection pool
/// * `Json(request)` - Login request containing email and password
/// 
/// # Returns
/// 
/// JSON response with JWT token and user info, or an error status code
/// 
/// # Errors
/// 
/// * `401 Unauthorized` - If user is not found or password is incorrect
/// * `503 Service Unavailable` - If database is not configured
/// * `500 Internal Server Error` - If database query or token generation fails
/// 
/// # Security Notes
/// 
/// - Invalid credentials return the same error code to prevent user enumeration
/// - Password verification uses constant-time comparison (via bcrypt)
/// - Passwords are never logged or returned in responses
/// 
/// # Example Request
/// 
/// ```http
/// POST /api/auth/login HTTP/1.1
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
///     "subscription_status": "active"
///   }
/// }
/// ```
#[cfg(feature = "ssr")]
pub async fn login(
    State(pool): State<Option<PgPool>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let pool = pool.ok_or_else(|| {
        tracing::error!("Database not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    tracing::info!("Login request for: {}", request.username);

    // Try to get user by username first, then by email (for backwards compatibility)
    let user = if request.username.contains('@') {
        // Looks like an email, try email lookup
        get_user_by_email(&pool, &request.username).await
    } else {
        // Try username lookup first
        get_user_by_username(&pool, &request.username).await
    };

    let user = user
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User not found: {}", request.username);
            StatusCode::UNAUTHORIZED
        })?;

    // Verify password
    let valid = verify(&request.password, &user.password_hash)
        .map_err(|e| {
            tracing::error!("Password verification error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !valid {
        tracing::warn!("Invalid password for user: {}", request.username);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create token
    let token = create_token(user.id, user.email.clone())
        .map_err(|e| {
            tracing::error!("Failed to create token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("User logged in successfully: {} ({})", user.username, user.email);

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
    use crate::backend::auth::users::create_user;
    use tests::common::database::TestDatabase;
    use bcrypt;

    #[tokio::test]
    async fn test_login_success() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        // Create test user
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = login(State(Some(pool.clone())), Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        // Create test user
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let _user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };
        
        let result = login(State(Some(pool.clone())), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = login(State(Some(pool.clone())), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_no_database() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = login(State(None), Json(request)).await;
        assert_eq!(result.unwrap_err(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
