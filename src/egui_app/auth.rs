/**
 * Authentication Module
 * 
 * Handles authentication UI and HTTP client functions for login/signup.
 */

use crate::egui_app::config::Config;
use crate::egui_app::types::{AuthResponse, UserInfo, LoginRequest, SignupRequest, UserResponse};
use reqwest::Client;
use tokio::runtime::Runtime;

/// Authentication state
#[derive(Debug, Clone)]
pub struct AuthState {
    pub authenticated: bool,
    pub user: Option<UserInfo>,
    pub error: Option<String>,
    pub loading: bool,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            authenticated: false,
            user: None,
            error: None,
            loading: false,
        }
    }
}

impl AuthState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn clear_error(&mut self) {
        self.error = None;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }
}

/// Login user with username and password
pub fn login(
    config: &Config,
    username: String,
    password: String,
) -> Result<AuthResponse, String> {
    let client = Client::new();
    let url = config.api_url("/api/auth/login");

    let request = LoginRequest { username, password };

    // Create a runtime for async execution
    let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async {
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
            return Err(format!("Login failed: {} - {}", status, error_text));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(auth_response)
    })
}

/// Signup new user with username, email, and password
pub fn signup(
    config: &Config,
    username: String,
    email: String,
    password: String,
) -> Result<AuthResponse, String> {
    let client = Client::new();
    let url = config.api_url("/api/auth/signup");

    let request = SignupRequest { username, email, password };

    // Create a runtime for async execution
    let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async {
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
            return Err(format!("Signup failed: {} - {}", status, error_text));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(auth_response)
    })
}

/// Get current user info with token
pub fn get_me(config: &Config, token: &str) -> Result<UserInfo, String> {
    let client = Client::new();
    let url = config.api_url("/api/auth/me");
    
    // Create a runtime for async execution
    let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    rt.block_on(async {
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
            return Err(format!("Get user failed: {} - {}", status, error_text));
        }
        
        let user_response: UserResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        Ok(UserInfo::from(user_response))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state_new() {
        let state = AuthState::new();
        assert!(!state.authenticated);
        assert!(state.user.is_none());
        assert!(state.error.is_none());
        assert!(!state.loading);
    }

    #[test]
    fn test_auth_state_default() {
        let state = AuthState::default();
        assert!(!state.authenticated);
        assert!(state.user.is_none());
    }

    #[test]
    fn test_auth_state_clear_error() {
        let mut state = AuthState::new();
        state.set_error("Test error".to_string());
        assert!(state.error.is_some());
        
        state.clear_error();
        assert!(state.error.is_none());
    }

    #[test]
    fn test_auth_state_set_error() {
        let mut state = AuthState::new();
        state.set_error("Test error".to_string());
        assert_eq!(state.error, Some("Test error".to_string()));
    }
}

