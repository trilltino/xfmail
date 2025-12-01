//! Middleware Module
//!
//! This module contains all HTTP middleware for the backend server.
//! Middleware functions are used to process requests before they reach
//! handlers, such as authentication, logging, rate limiting, etc.
//!
//! # Architecture
//!
//! The middleware module currently provides:
//!
//! - **`auth`** - Authentication middleware for protecting routes
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::middleware::auth_middleware;
//!
//! // Apply auth middleware to a route
//! // let protected_route = route.layer(auth_middleware());
//! ```

pub mod auth;

pub use auth::{AuthenticatedUser, AuthUser, auth_middleware, extract_authenticated_user};

