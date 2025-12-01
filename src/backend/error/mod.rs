//! Backend Error Module
//!
//! This module defines error types specific to the backend server.
//! These errors are used in HTTP handlers and can be converted to HTTP responses.
//!
//! # Architecture
//!
//! The error module is organized into focused submodules:
//!
//! - **`types`** - Error type definitions and constructors
//! - **`conversion`** - Error conversion implementations (IntoResponse, etc.)
//!
//! # Module Structure
//!
//! ```
//! error/
//! ├── mod.rs        - Module exports and documentation
//! ├── types.rs      - Error type definitions
//! └── conversion.rs - Error conversion implementations
//! ```
//!
//! # Error Types
//!
//! - `HandlerError` - Errors that occur in HTTP handlers
//! - `StateError` - Errors related to application state management
//! - `ProtocolError` - Braid protocol-specific errors
//! - `SharedError` - Errors from the shared module
//! - `SerializationError` - JSON serialization errors
//!
//! # HTTP Response Conversion
//!
//! All backend errors implement `IntoResponse` from Axum, allowing them to be
//! returned directly from handlers. The error is automatically converted to an
//! appropriate HTTP status code and JSON response body.
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::error::BackendError;
//! use axum::response::Response;
//!
//! # async fn example() -> Result<Response, BackendError> {
//! // Handler can return BackendError directly
//! # Ok(Response::new("OK".into()))
//! # }
//! ```

/// Error type definitions
pub mod types;

/// Error conversion implementations
pub mod conversion;

// Re-export commonly used types
pub use types::BackendError;

