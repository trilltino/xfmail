/**
 * Get Current User Handler
 * 
 * This module implements the handler for GET /api/auth/me, which returns
 * information about the currently authenticated user.
 * 
 * # Authentication
 * 
 * This endpoint requires a valid JWT token in the `Authorization` header.
 * The token is verified and the user ID is extracted to fetch user information.
 * 
 * # Response
 * 
 * Returns user information without sensitive data (no password hash).
 * Includes subscription status if available.
 */

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
#[cfg(feature = "ssr")]
use sqlx::PgPool;

use crate::backend::auth::sessions::verify_token;
use crate::backend::auth::handlers::types::UserResponse;

/// Get current user handler
/// 
/// This handler returns information about the currently authenticated user.
/// It extracts the JWT token from the Authorization header, verifies it, and
/// returns the user's information.
/// 
/// # Arguments
/// 
/// * `State(pool)` - Database connection pool
/// * `headers` - Request headers (to extract Authorization header)
/// 
/// # Returns
/// 
/// JSON response with user info, or an error status code
/// 
/// # Errors
/// 
/// * `401 Unauthorized` - If Authorization header is missing or token is invalid
/// * `404 Not Found` - If user is not found in database
/// * `503 Service Unavailable` - If database is not configured
/// * `500 Internal Server Error` - If token verification or database query fails
/// 
/// # Example Request
/// 
/// ```http
/// GET /api/auth/me HTTP/1.1
/// Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
/// ```
/// 
/// # Example Response
/// 
/// ```json
/// {
///   "id": "123e4567-e89b-12d3-a456-426614174000",
///   "email": "user@example.com",
///   "subscription_status": "active"
/// }
/// ```
#[cfg(feature = "ssr")]
pub async fn get_me(
    State(pool): State<Option<PgPool>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    let pool = pool.ok_or_else(|| {
        tracing::error!("Database not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    
    // Get token from Authorization header
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Missing authorization header");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Extract token (format: "Bearer <token>")
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| {
            tracing::warn!("Invalid authorization header format");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Verify token
    let claims = verify_token(token)
        .map_err(|e| {
            tracing::warn!("Invalid token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Get user ID from claims
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|e| {
            tracing::error!("Invalid user ID in token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get user from database
    let user = crate::backend::auth::users::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User not found: {}", user_id);
            StatusCode::NOT_FOUND
        })?;
    
    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        subscription_status: user.subscription_status,
    }))
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::HeaderMap;
    use crate::backend::auth::sessions::create_token;
    use crate::backend::auth::users::create_user;
    use tests::common::database::TestDatabase;
    use bcrypt;

    #[tokio::test]
    async fn test_get_me_success() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        // Create user and token
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        let token = create_token(user.id, user.email.clone()).unwrap();
        
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", token).parse().unwrap());
        
        let result = get_me(State(Some(pool.clone())), headers).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_me_no_auth_header() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let headers = HeaderMap::new();
        let result = get_me(State(Some(pool.clone())), headers).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_me_invalid_token() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid_token".parse().unwrap());
        
        let result = get_me(State(Some(pool.clone())), headers).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_me_no_database() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer token".parse().unwrap());
        
        let result = get_me(State(None), headers).await;
        assert_eq!(result.unwrap_err(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

