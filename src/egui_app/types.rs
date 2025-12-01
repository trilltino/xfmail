/**
 * Shared Types Module
 * 
 * Defines shared types for the egui app including app view states and user info.
 */

use serde::{Deserialize, Serialize};

/// Current app view/mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppView {
    /// Login/signup screen
    Auth,
    /// Landing page with app selection buttons
    Landing,
    /// Messaging - Telegram-style messaging application
    Messaging,
    /// XFCollab - Collaborative editing
    XFCollab,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub subscription_status: Option<String>,
}

/// Authentication response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

// Re-export auth types from backend for compatibility
#[cfg(feature = "ssr")]
pub use crate::backend::auth::handlers::types::{LoginRequest, SignupRequest, UserResponse};

// Define types for non-SSR builds
#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub subscription_status: Option<String>,
}

impl From<UserResponse> for UserInfo {
    fn from(value: UserResponse) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            subscription_status: value.subscription_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_view_variants() {
        let auth = AppView::Auth;
        let landing = AppView::Landing;
        let messaging = AppView::Messaging;
        let xfcollab = AppView::XFCollab;

        assert_eq!(auth, AppView::Auth);
        assert_eq!(landing, AppView::Landing);
        assert_eq!(messaging, AppView::Messaging);
        assert_eq!(xfcollab, AppView::XFCollab);
    }

    #[test]
    fn test_user_info_creation() {
        let user = UserInfo {
            id: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            subscription_status: Some("active".to_string()),
        };

        assert_eq!(user.id, "123");
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.subscription_status, Some("active".to_string()));
    }

    #[test]
    fn test_auth_response_creation() {
        let user = UserInfo {
            id: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            subscription_status: None,
        };

        let response = AuthResponse {
            token: "token123".to_string(),
            user: user.clone(),
        };

        assert_eq!(response.token, "token123");
        assert_eq!(response.user.email, "test@example.com");
    }

    #[test]
    fn test_user_response_to_user_info() {
        let user_response = UserResponse {
            id: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            subscription_status: Some("active".to_string()),
        };

        let user_info: UserInfo = user_response.into();
        assert_eq!(user_info.id, "123");
        assert_eq!(user_info.username, "testuser");
        assert_eq!(user_info.email, "test@example.com");
        assert_eq!(user_info.subscription_status, Some("active".to_string()));
    }

    #[test]
    fn test_user_info_serialization() {
        let user = UserInfo {
            id: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            subscription_status: Some("active".to_string()),
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: UserInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.username, deserialized.username);
        assert_eq!(user.email, deserialized.email);
    }
}

