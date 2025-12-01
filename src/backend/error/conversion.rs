/**
 * Error Conversion
 * 
 * This module provides conversion implementations for backend errors,
 * allowing them to be converted to HTTP responses and other formats.
 * 
 * # HTTP Response Conversion
 * 
 * All backend errors implement `IntoResponse` from Axum, allowing them to be
 * returned directly from handlers. The error is automatically converted to an
 * appropriate HTTP status code and response body.
 * 
 * # Response Format
 * 
 * Error responses are returned as JSON with the following structure:
 * ```json
 * {
 *   "error": "Error message",
 *   "status": 400
 * }
 * ```
 */

use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body,
};
use crate::backend::error::types::BackendError;

impl IntoResponse for BackendError {
    /// Convert a backend error into an HTTP response
    /// 
    /// This implementation creates a JSON error response with the appropriate
    /// status code and error message.
    /// 
    /// # Response Format
    /// 
    /// The response is a JSON object with:
    /// - `error`: The error message
    /// - `status`: The HTTP status code
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use braid_site::backend::error::BackendError;
    /// use axum::response::Response;
    /// 
    /// async fn handler() -> Result<Response, BackendError> {
    ///     let err = BackendError::handler(StatusCode::BAD_REQUEST, "Invalid request");
    ///     Err(err)
    /// }
    /// ```
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.message();
        
        // Create a JSON error response
        let body = serde_json::json!({
            "error": message,
            "status": status.as_u16(),
        });
        
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap_or_else(|_| {
                format!(r#"{{"error":"{}","status":{}}}"#, message, status.as_u16())
            })))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap()
            })
    }
}

