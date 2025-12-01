//! Authentication Handlers Module
//!
//! This module contains all HTTP handlers for authentication endpoints.
//! Handlers are organized into focused submodules for maintainability.
//!
//! # Module Structure
//!
//! ```
//! handlers/
//! ├── mod.rs      - Module exports and documentation
//! ├── types.rs    - Request and response types
//! ├── signup.rs   - User registration handler
//! ├── login.rs    - User authentication handler
//! └── me.rs       - Get current user handler
//! ```
//!
//! # Handlers
//!
//! - **`signup`** - POST /api/auth/signup - User registration
//! - **`login`** - POST /api/auth/login - User authentication
//! - **`get_me`** - GET /api/auth/me - Get current user info
//!
//! # Authentication Flow
//!
//! 1. **Signup**: User provides email and password → User created → JWT token returned
//! 2. **Login**: User provides email and password → Credentials verified → JWT token returned
//! 3. **Get Me**: User provides JWT token → Token verified → User info returned
//!
//! # Security
//!
//! - Passwords are hashed using bcrypt before storage
//! - JWT tokens are used for stateless authentication
//! - Tokens expire after 30 days
//! - Invalid credentials return 401 (no information leakage)
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::auth::handlers::{signup, login, get_me};
//! use axum::{routing::post, routing::get, Router};
//!
//! # async fn example() {
//! let router = Router::new()
//!     .route("/api/auth/signup", post(signup))
//!     .route("/api/auth/login", post(login))
//!     .route("/api/auth/me", get(get_me));
//! # }
//! ```

/// Request and response types
pub mod types;

/// Signup handler
pub mod signup;

/// Login handler
pub mod login;

/// Get current user handler
pub mod me;

// Re-export commonly used types
pub use types::{SignupRequest, LoginRequest, AuthResponse, UserResponse};

// Re-export handlers
#[cfg(feature = "ssr")]
pub use signup::signup;
#[cfg(feature = "ssr")]
pub use login::login;
#[cfg(feature = "ssr")]
pub use me::get_me;

