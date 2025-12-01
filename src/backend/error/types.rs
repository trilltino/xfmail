/**
 * Backend Error Types
 * 
 * This module defines error types specific to the backend server.
 * These errors are used in HTTP handlers and can be converted to HTTP responses.
 * 
 * # Error Types
 * 
 * - `HandlerError` - Errors that occur in HTTP handlers
 * - `StateError` - Errors related to application state management
 * - `ProtocolError` - Braid protocol-specific errors
 * 
 * # Error Categories
 * 
 * ## Handler Errors
 * 
 * Handler errors occur when processing HTTP requests:
 * - Missing required headers
 * - Invalid request format
 * - Authentication failures
 * 
 * ## State Errors
 * 
 * State errors occur when managing application state:
 * - Lock acquisition failures
 * - State corruption
 * - Concurrent access issues
 * 
 * ## Protocol Errors
 * 
 * Protocol errors occur when processing Braid protocol requests:
 * - Invalid version headers
 * - Malformed Parents headers
 * - Protocol violations
 */

use thiserror::Error;
use axum::http::StatusCode;
use crate::shared::SharedError;

/// Backend-specific error types
/// 
/// This enum represents all possible errors that can occur in the backend.
/// Each variant includes relevant context and can be converted to an HTTP response.
/// 
/// # Usage
/// 
/// ```rust
/// use braid_site::backend::error::BackendError;
/// 
/// // Create a handler error
/// let err = BackendError::handler(StatusCode::BAD_REQUEST, "Invalid request");
/// 
/// // Create a state error
/// let err = BackendError::state("Failed to acquire lock");
/// 
/// // Create a protocol error
/// let err = BackendError::protocol("Invalid version header");
/// ```
#[derive(Debug, Error)]
pub enum BackendError {
    /// Handler error (e.g., missing headers, invalid request)
    /// 
    /// This error occurs when processing HTTP requests fails due to
    /// invalid input, missing headers, or other request-related issues.
    #[error("Handler error: {message}")]
    HandlerError {
        /// HTTP status code for this error
        status: StatusCode,
        /// Human-readable error message
        message: String,
    },
    
    /// State management error (e.g., lock acquisition failure)
    /// 
    /// This error occurs when managing application state fails, such as
    /// when acquiring locks or updating shared state.
    #[error("State error: {message}")]
    StateError {
        /// Human-readable error message
        message: String,
    },
    
    /// Braid protocol error
    /// 
    /// This error occurs when processing Braid protocol requests fails,
    /// such as when version headers are invalid or protocol rules are violated.
    #[error("Protocol error: {message}")]
    ProtocolError {
        /// Human-readable error message
        message: String,
    },
    
    /// Shared error (from shared module)
    /// 
    /// This error wraps errors from the shared module, such as
    /// serialization errors or validation errors.
    #[error(transparent)]
    SharedError(#[from] SharedError),
    
    /// Serialization error
    /// 
    /// This error occurs when serializing or deserializing data fails.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl BackendError {
    /// Create a new handler error with a status code
    /// 
    /// # Arguments
    /// 
    /// * `status` - HTTP status code
    /// * `message` - Error message
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use braid_site::backend::error::BackendError;
    /// use axum::http::StatusCode;
    /// 
    /// let err = BackendError::handler(StatusCode::BAD_REQUEST, "Invalid request");
    /// ```
    pub fn handler(status: StatusCode, message: impl Into<String>) -> Self {
        Self::HandlerError {
            status,
            message: message.into(),
        }
    }
    
    /// Create a new state error
    /// 
    /// # Arguments
    /// 
    /// * `message` - Error message
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use braid_site::backend::error::BackendError;
    /// 
    /// let err = BackendError::state("Failed to acquire lock");
    /// ```
    pub fn state(message: impl Into<String>) -> Self {
        Self::StateError {
            message: message.into(),
        }
    }
    
    /// Create a new protocol error
    /// 
    /// # Arguments
    /// 
    /// * `message` - Error message
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use braid_site::backend::error::BackendError;
    /// 
    /// let err = BackendError::protocol("Invalid version header");
    /// ```
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: message.into(),
        }
    }
    
    /// Get the HTTP status code for this error
    /// 
    /// # Returns
    /// 
    /// The appropriate HTTP status code for this error type
    /// 
    /// # Status Code Mapping
    /// 
    /// - `HandlerError` - Uses the status code from the error
    /// - `StateError` - 500 Internal Server Error
    /// - `ProtocolError` - 400 Bad Request
    /// - `SharedError` - Depends on the shared error type
    /// - `SerializationError` - 500 Internal Server Error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::HandlerError { status, .. } => *status,
            Self::StateError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ProtocolError { .. } => StatusCode::BAD_REQUEST,
            Self::SharedError(err) => match err {
                SharedError::SerializationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                SharedError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                SharedError::MessageError { .. } => StatusCode::BAD_REQUEST,
            },
            Self::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Get the error message
    /// 
    /// # Returns
    /// 
    /// A human-readable error message
    pub fn message(&self) -> String {
        match self {
            Self::HandlerError { message, .. } => message.clone(),
            Self::StateError { message, .. } => message.clone(),
            Self::ProtocolError { message, .. } => message.clone(),
            Self::SharedError(err) => err.to_string(),
            Self::SerializationError(err) => err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_error() {
        let error = BackendError::handler(StatusCode::BAD_REQUEST, "Invalid request");
        match error {
            BackendError::HandlerError { status, message } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert_eq!(message, "Invalid request");
            }
            _ => panic!("Expected HandlerError"),
        }
    }

    #[test]
    fn test_state_error() {
        let error = BackendError::state("Lock failed");
        match error {
            BackendError::StateError { message } => {
                assert_eq!(message, "Lock failed");
            }
            _ => panic!("Expected StateError"),
        }
    }

    #[test]
    fn test_protocol_error() {
        let error = BackendError::protocol("Invalid version");
        match error {
            BackendError::ProtocolError { message } => {
                assert_eq!(message, "Invalid version");
            }
            _ => panic!("Expected ProtocolError"),
        }
    }

    #[test]
    fn test_status_code_mapping() {
        let handler_error = BackendError::handler(StatusCode::UNAUTHORIZED, "Unauthorized");
        assert_eq!(handler_error.status_code(), StatusCode::UNAUTHORIZED);

        let state_error = BackendError::state("State error");
        assert_eq!(state_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        let protocol_error = BackendError::protocol("Protocol error");
        assert_eq!(protocol_error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_shared_error() {
        let shared_error = SharedError::validation("field", "message");
        let backend_error: BackendError = shared_error.into();
        
        match backend_error {
            BackendError::SharedError(_) => {}
            _ => panic!("Expected SharedError variant"),
        }
    }

    #[test]
    fn test_error_message() {
        let error = BackendError::handler(StatusCode::BAD_REQUEST, "Test message");
        assert!(error.message().contains("Test message"));
    }
}

