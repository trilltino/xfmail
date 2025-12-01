use std::sync::mpsc::{channel, Receiver};

use crate::egui_app::{
    login, signup, AppView, AuthState, Config, DebugLogger, DebugCategory,
};
use crate::egui_app::messaging::MessagingState;

/// Central application state shared across egui views.
pub struct AppState {
    pub config: Config,
    pub auth_state: AuthState,
    pub current_view: AppView,
    pub username_input: String,
    pub email_input: String,
    pub password_input: String,
    pub confirm_password_input: String,
    pub is_signup_mode: bool,
    pub auth_result: Option<Receiver<Result<(String, crate::egui_app::UserInfo), String>>>,
    pub debug_logger: DebugLogger,
    pub debug_view_expanded: bool,
    pub debug_filter_category: Option<DebugCategory>,
    /// Messaging state for Telegram-style messaging UI
    pub messaging_state: MessagingState,

    /// Network connectivity state
    pub is_online: bool,
    pub last_sync_time: Option<String>,
    pub pending_sync_operations: usize,
}

impl AppState {
    pub fn new() -> Self {
        let debug_logger = DebugLogger::new(1000);
        debug_logger.info(DebugCategory::Other, "AppState initialized");

        Self {
            config: Config::new(),
            auth_state: AuthState::new(),
            current_view: AppView::Auth,
            username_input: String::new(),
            email_input: String::new(),
            password_input: String::new(),
            confirm_password_input: String::new(),
            is_signup_mode: false,
            auth_result: None,
            debug_logger,
            debug_view_expanded: false,
            debug_filter_category: None,
            messaging_state: MessagingState::new(),
            is_online: true, // Assume online by default
            last_sync_time: None,
            pending_sync_operations: 0,
        }
    }

    pub fn check_auth_result(&mut self) {
        if let Some(ref rx) = self.auth_result {
            if let Ok(result) = rx.try_recv() {
                self.auth_result = None;
                self.auth_state.loading = false;

                match result {
                    Ok((token, user)) => {
                        self.debug_logger.info(DebugCategory::Auth, format!("✓ Authentication successful: {}", user.email));
                        self.config.set_token(Some(token));
                        self.auth_state.authenticated = true;
                        self.auth_state.user = Some(user);
                        self.auth_state.error = None;
                        self.current_view = AppView::Landing;
                        self.password_input.clear();
                        self.confirm_password_input.clear();
                        self.is_signup_mode = false;
                    }
                    Err(e) => {
                        self.debug_logger.error(DebugCategory::Auth, format!("✗ Authentication failed: {}", e));
                        self.auth_state.set_error(e);
                    }
                }
            }
        }
    }

    pub fn handle_login(&mut self) {
        if self.username_input.is_empty() || self.password_input.is_empty() {
            self.auth_state
                .set_error("Username and password are required".to_string());
            return;
        }

        self.auth_state.loading = true;
        self.auth_state.error = None;

        let username = self.username_input.clone();
        let password = self.password_input.clone();
        let config = self.config.clone();

        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let result = login(&config, username, password).map(|auth| (auth.token, auth.user));
            let _ = tx.send(result);
        });

        self.auth_result = Some(rx);
    }

    pub fn handle_signup(&mut self) {
        if self.username_input.is_empty() {
            self.auth_state
                .set_error("Username is required".to_string());
            return;
        }

        if self.email_input.is_empty() || self.password_input.is_empty() {
            self.auth_state
                .set_error("Email and password are required".to_string());
            return;
        }

        // Simple email validation
        if !self.email_input.contains('@') || !self.email_input.contains('.') {
            self.auth_state
                .set_error("Please enter a valid email address".to_string());
            return;
        }

        if self.password_input != self.confirm_password_input {
            self.auth_state
                .set_error("Passwords do not match".to_string());
            return;
        }

        self.auth_state.loading = true;
        self.auth_state.error = None;

        let username = self.username_input.clone();
        let email = self.email_input.clone();
        let password = self.password_input.clone();
        let config = self.config.clone();

        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let result = signup(&config, username, email, password).map(|auth| (auth.token, auth.user));
            let _ = tx.send(result);
        });

        self.auth_result = Some(rx);
    }

    pub fn logout(&mut self) {
        self.config.clear_token();
        self.auth_state = AuthState::new();
        self.current_view = AppView::Auth;
        self.username_input.clear();
        self.email_input.clear();
        self.password_input.clear();
        self.confirm_password_input.clear();
        self.messaging_state = MessagingState::new();
    }

    pub fn toggle_auth_mode(&mut self) {
        self.is_signup_mode = !self.is_signup_mode;
        self.auth_state.clear_error();
        self.password_input.clear();
        self.confirm_password_input.clear();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

