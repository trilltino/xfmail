/**
 * Session Management and JWT Tokens
 * 
 * This module handles JWT token generation and validation for user sessions.
 */

#[cfg(feature = "ssr")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims structure
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Email
    pub email: String,
    /// Username (optional for backwards compatibility)
    #[serde(default)]
    pub username: Option<String>,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issued at time (Unix timestamp)
    pub iat: u64,
}



/// Get JWT secret from environment
#[cfg(feature = "ssr")]
fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
    .unwrap_or_else(|err| {
        eprintln!("Missing JWT_SECRET. Error: {}", err);
        "your-secret-key-change-in-production".to_string()
    })
}

/// Create a JWT token for a user
/// 
/// # Arguments
/// * `user_id` - User ID (UUID)
/// * `email` - User email
/// 
/// # Returns
/// JWT token string
#[cfg(feature = "ssr")]
pub fn create_token(user_id: uuid::Uuid, email: String) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Token expires in 30 days
    let exp = now + (30 * 24 * 60 * 60);
    
    let claims = Claims {
        sub: user_id.to_string(),
        email,
        username: None,
        exp,
        iat: now,
    };
    
    let secret = get_jwt_secret();
    let key = EncodingKey::from_secret(secret.as_ref());
    
    encode(&Header::default(), &claims, &key)
}

/// Verify and decode a JWT token
/// 
/// # Arguments
/// * `token` - JWT token string
/// 
/// # Returns
/// Decoded claims or error
#[cfg(feature = "ssr")]
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

/// Extract user ID from token
/// 
/// # Arguments
/// * `token` - JWT token string
/// 
/// # Returns
/// User ID (UUID) or error
#[cfg(feature = "ssr")]
pub fn get_user_id_from_token(token: &str) -> Result<uuid::Uuid, String> {
    let claims = verify_token(token)
        .map_err(|e| format!("Token verification failed: {}", e))?;
    uuid::Uuid::parse_str(&claims.sub)
        .map_err(|e| format!("Invalid user ID in token: {}", e))
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;

    #[test]
    fn test_create_token() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let result = create_token(user_id, email.clone());
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_verify_token() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = create_token(user_id, email.clone()).unwrap();
        
        let result = verify_token(&token);
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.email, email);
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[test]
    fn test_get_user_id_from_token() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = create_token(user_id, email).unwrap();
        
        let result = get_user_id_from_token(&token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_verify_invalid_token() {
        let invalid_token = "invalid.token.here";
        let result = verify_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_contains_user_info() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = create_token(user_id, email.clone()).unwrap();
        
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.email, email);
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > claims.iat);
    }
}

