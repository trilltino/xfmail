/**
 * Typing Indicator Handler
 * 
 * This module implements the typing indicator event handler for POST /typing requests.
 * 
 * # Typing Indicators
 * 
 * Typing indicators allow users to see when others are typing in real-time.
 * This improves the user experience by providing immediate feedback.
 * 
 * # Event Flow
 * 
 * 1. Client sends typing event (user started/stopped typing)
 * 2. Server receives event and broadcasts it via real-time event system
 * 3. All subscribers receive the typing event
 * 4. Clients update their UI to show/hide typing indicators
 * 
 * # Real-time Event System
 * 
 * Typing events are broadcast using the generic real-time event system,
 * which supports filtering by event type. Clients can subscribe to typing
 * events specifically using the `/realtime?types=typing` endpoint.
 */

use crate::backend::server::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::Response,
};

/// Handle typing indicator event (POST /typing)
/// 
/// This handler receives typing events from clients and broadcasts them
/// to all subscribers via the realtime event system.
/// 
/// # Request Body
/// 
/// JSON object with:
/// - `user`: String - The username of the person typing
/// - `is_typing`: bool - Whether the user is typing (true) or stopped (false)
/// 
/// # Returns
/// 
/// HTTP 200 OK on success, or an error status code
/// 
/// # Errors
/// 
/// * `400 Bad Request` - If the request body cannot be parsed
/// 
/// # Example Request
/// 
/// ```http
/// POST /typing HTTP/1.1
/// Content-Type: application/json
/// 
/// {"user":"Alice","is_typing":true}
/// ```
/// 
/// # Example Response
/// 
/// ```http
/// HTTP/1.1 200 OK
/// ```
/// 
/// # Broadcasting
/// 
/// The typing event is broadcast to all subscribers of the real-time event
/// system. Clients can subscribe to typing events using:
/// 
/// ```http
/// GET /realtime?types=typing HTTP/1.1
/// Subscribe:
/// ```
#[cfg(feature = "ssr")]
pub async fn handle_typing_event(
    State(app_state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    use crate::backend::realtime::broadcast::broadcast_event;
    use crate::shared::RealtimeEvent;
    
    // Parse request body
    #[derive(serde::Deserialize)]
    struct TypingRequest {
        /// Username of the person typing
        user: String,
        /// Whether the user is typing (true) or stopped (false)
        is_typing: bool,
    }
    
    let typing_request: TypingRequest = serde_json::from_slice(&body)
        .map_err(|e| {
            tracing::error!("[Server] Failed to parse typing request: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    tracing::debug!(
        "[Server] Received typing event: user={}, is_typing={}",
        typing_request.user,
        typing_request.is_typing
    );
    
    // Create typing event and broadcast it
    let event = RealtimeEvent::typing(typing_request.user, typing_request.is_typing);
    broadcast_event(&app_state.realtime_broadcast, event).await;
    
    // Return success response
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .map_err(|e| {
            tracing::error!("[Server] Failed to build response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
}

