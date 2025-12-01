/**
 * Authentication Handler Types
 * 
 * This module defines the request and response types used by authentication handlers.
 * These types are shared across signup, login, and get_me handlers.
 */

use serde::{Deserialize, Serialize};

/// Sign up request
///
/// Contains the username, email and password for user registration.
#[derive(Deserialize, Serialize, Debug)]
pub struct SignupRequest {
    /// User's chosen username (3-30 chars, alphanumeric + underscore)
    pub username: String,
    /// User's email address
    pub email: String,
    /// User's password (will be hashed before storage)
    pub password: String,
}

/// Login request
///
/// Contains the username (or email) and password for user authentication.
#[derive(Deserialize, Serialize, Debug)]
pub struct LoginRequest {
    /// User's username (can also be email for backwards compatibility)
    pub username: String,
    /// User's password (will be verified against stored hash)
    pub password: String,
}

/// Auth response
///
/// Returned by signup and login handlers. Contains the JWT token
/// and user information for immediate authentication.
#[derive(Serialize, Debug)]
pub struct AuthResponse {
    /// JWT token for authentication (30-day expiration)
    pub token: String,
    /// User information (without sensitive data)
    pub user: UserResponse,
}

/// User response (without sensitive data)
///
/// Contains user information that is safe to return to clients.
/// Does not include password hash or other sensitive information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    /// User's unique ID (UUID)
    pub id: String,
    /// User's username
    pub username: String,
    /// User's email address
    pub email: String,
    /// Subscription status (active, cancelled, past_due, etc.)
    pub subscription_status: Option<String>,
}

