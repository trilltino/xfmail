//! Shared Error Types
//!
//! This module defines error types that are shared between the frontend and backend.
//! These errors represent common failure cases that can occur in both contexts.
//!
//! # Error Categories
//!
//! - `SerializationError` - JSON serialization/deserialization failures
//! - `ValidationError` - Data validation failures
//! - `MessageError` - Message-related errors
//!
//! # Usage
//!
//! ```rust
//! use xfmail::shared::error::SharedError;
//!
//! // Create a validation error
//! let error = SharedError::validation("text", "Message text cannot be empty");
//! ```
//!
//! # Thread Safety
//!
//! All error types are `Send + Sync` and can be safely shared across thread boundaries.
use thiserror::Error;

/// Shared error types that can occur in both frontend and backend
#[derive(Debug, Error, Clone)]
pub enum SharedError {
    /// JSON serialization or deserialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Human-readable error message
        message: String,
    },
    
    /// Data validation error
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        /// The field that failed validation
        field: String,
        /// Human-readable error message
        message: String,
    },
    
    /// Message-related error
    #[error("Message error: {message}")]
    MessageError {
        /// Human-readable error message
        message: String,
    },
}

impl SharedError {
    /// Create a new serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }
    
    /// Create a new validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// Create a new message error
    pub fn message(message: impl Into<String>) -> Self {
        Self::MessageError {
            message: message.into(),
        }
    }
}

/// Helper trait for converting serialization errors
impl From<serde_json::Error> for SharedError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(format!("JSON error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_error() {
        let error = SharedError::serialization("Invalid JSON");
        match error {
            SharedError::SerializationError { message } => {
                assert_eq!(message, "Invalid JSON");
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_validation_error() {
        let error = SharedError::validation("email", "Invalid email format");
        match error {
            SharedError::ValidationError { field, message } => {
                assert_eq!(field, "email");
                assert_eq!(message, "Invalid email format");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_message_error() {
        let error = SharedError::message("Message too long");
        match error {
            SharedError::MessageError { message } => {
                assert_eq!(message, "Message too long");
            }
            _ => panic!("Expected MessageError"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = SharedError::serialization("Test error");
        let display = format!("{}", error);
        assert!(display.contains("Serialization error"));
        assert!(display.contains("Test error"));
    }

    #[test]
    fn test_from_serde_error() {
        let invalid_json = "{ invalid json }";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        let serde_error = result.unwrap_err();
        let shared_error: SharedError = serde_error.into();
        
        match shared_error {
            SharedError::SerializationError { .. } => {}
            _ => panic!("Expected SerializationError from serde error"),
        }
    }

    #[test]
    fn test_error_clone() {
        let error = SharedError::validation("field", "message");
        let cloned = error.clone();
        match (error, cloned) {
            (
                SharedError::ValidationError { field: f1, message: m1 },
                SharedError::ValidationError { field: f2, message: m2 },
            ) => {
                assert_eq!(f1, f2);
                assert_eq!(m1, m2);
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}