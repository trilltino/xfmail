//! Authentication Module
//!
//! This module handles user authentication, registration, and session management.
//! It provides HTTP handlers for authentication endpoints and manages user data
//! and JWT tokens.
//!
//! # Architecture
//!
//! The auth module is organized into focused submodules:
//!
//! - **`users`** - User data model and database operations
//! - **`sessions`** - JWT token generation and validation
//! - **`handlers`** - HTTP handlers for authentication endpoints
//!
//! # Module Structure
//!
//! ```
//! auth/
//! ├── mod.rs          - Module exports and documentation
//! ├── users.rs        - User model and database operations
//! ├── sessions.rs     - JWT token management
//! └── handlers/       - HTTP handlers
//!     ├── mod.rs      - Handler exports
//!     ├── types.rs    - Request/response types
//!     ├── signup.rs   - User registration handler
//!     ├── login.rs    - User authentication handler
//!     └── me.rs       - Get current user handler
//! ```
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
//! use braid_site::backend::auth::{signup, login, get_me};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Handlers are used in route definitions
//! // let app = Router::new()
//! //     .route("/api/auth/signup", post(signup))
//! //     .route("/api/auth/login", post(login))
//! //     .route("/api/auth/me", get(get_me));
//! # Ok(())
//! # }
//! ```

/// User data model and database operations
pub mod users;

/// JWT token generation and validation
pub mod sessions;

/// HTTP handlers for authentication endpoints
pub mod handlers;

// Re-export commonly used types and handlers
pub use handlers::types::{SignupRequest, LoginRequest, AuthResponse, UserResponse};
#[cfg(feature = "ssr")]
pub use handlers::{signup, login, get_me};

