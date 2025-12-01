/**
 * Authentication Middleware
 * 
 * This module provides middleware for protecting routes that require
 * user authentication. It extracts and verifies JWT tokens from the
 * Authorization header and provides the user ID to handlers.
 */

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::backend::auth::sessions::verify_token;
use crate::backend::server::state::AppState;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
use uuid::Uuid;

/// Authenticated user data extracted from JWT token
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

/// Authentication middleware
/// 
/// This middleware:
/// 1. Extracts JWT token from Authorization header
/// 2. Verifies the token
/// 3. Extracts user ID from token claims
/// 4. Attaches user data to request extensions for use in handlers
/// 
/// Returns 401 Unauthorized if token is missing or invalid
#[cfg(feature = "ssr")]
pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get Authorization header
    let auth_header = request.headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Extract token (format: "Bearer <token>")
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| {
            tracing::warn!("Invalid Authorization header format");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Verify token
    let claims = verify_token(token)
        .map_err(|e| {
            tracing::warn!("Invalid token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Parse user ID from claims
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| {
            tracing::error!("Invalid user ID in token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Verify user exists in database (optional but recommended for security)
    if let Some(pool) = &app_state.db_pool {
        if let Err(e) = verify_user_exists(pool, user_id).await {
            tracing::warn!("User not found in database: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    
    // Attach authenticated user to request extensions
    request.extensions_mut().insert(AuthenticatedUser {
        user_id,
        email: claims.email,
    });
    
    Ok(next.run(request).await)
}

/// Verify user exists in database
#[cfg(feature = "ssr")]
async fn verify_user_exists(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    use crate::backend::auth::users::get_user_by_id;
    
    get_user_by_id(pool, user_id).await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    
    Ok(())
}

/// Extract authenticated user from request extensions
/// 
/// This is a helper function for handlers to get the authenticated user
/// that was set by the auth middleware.
pub fn extract_authenticated_user(request: &Request) -> Result<AuthenticatedUser, StatusCode> {
    request.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| {
            tracing::warn!("AuthenticatedUser not found in request extensions");
            StatusCode::UNAUTHORIZED
        })
}

/// Axum extractor for authenticated user
/// 
/// This can be used as a parameter in handlers to automatically extract
/// the authenticated user from request extensions.
#[derive(Clone, Debug)]
pub struct AuthUser(pub AuthenticatedUser);

impl axum::extract::FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;
    
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| {
                tracing::warn!("AuthenticatedUser not found in request extensions");
                StatusCode::UNAUTHORIZED
            })?;
        
        Ok(AuthUser(user))
    }
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, Request};
    use crate::backend::auth::sessions::create_token;
    use crate::backend::auth::users::create_user;
    use crate::backend::server::state::AppState;
    use tests::common::database::TestDatabase;
    use bcrypt;

    #[tokio::test]
    async fn test_extract_authenticated_user() {
        let mut request = Request::builder()
            .uri("http://example.com")
            .body(())
            .unwrap();
        
        let user = AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
        };
        request.extensions_mut().insert(user.clone());
        
        let extracted = extract_authenticated_user(&request);
        assert!(extracted.is_ok());
        assert_eq!(extracted.unwrap().user_id, user.user_id);
    }

    #[tokio::test]
    async fn test_extract_authenticated_user_missing() {
        let request = Request::builder()
            .uri("http://example.com")
            .body(())
            .unwrap();
        
        let extracted = extract_authenticated_user(&request);
        assert_eq!(extracted.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_verify_user_exists() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        
        let result = verify_user_exists(pool, user.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_user_not_exists() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let non_existent_id = uuid::Uuid::new_v4();
        let result = verify_user_exists(pool, non_existent_id).await;
        assert!(result.is_err());
    }
}

