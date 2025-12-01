/**
 * Chat Route Handlers
 * 
 * This module defines route handlers for chat-related endpoints,
 * including Braid protocol endpoints and typing indicators.
 * 
 * # Routes
 * 
 * - `GET /chat` - Braid subscription or page rendering
 * - `PUT /chat` - Braid PUT for adding messages
 * - `POST /typing` - Typing indicator events
 * - `GET /realtime` - Generic real-time event subscription
 * 
 * # Braid Protocol
 * 
 * The `/chat` endpoint implements the Braid HTTP protocol:
 * - GET with `Subscribe:` header initiates a subscription
 * - PUT with `Parents:` header adds a new message
 * - Without `Subscribe:` header, GET renders the Leptos page
 */

use axum::Router;
#[cfg(feature = "ssr")]
use crate::backend::server::state::AppState;
#[cfg(feature = "ssr")]
use crate::backend::chat::handlers::{handle_braid_put, handle_typing_event};
#[cfg(feature = "ssr")]
use crate::backend::realtime::subscription::handle_realtime_subscription;

/// Configure chat-related routes
/// 
/// This function adds the following routes to the router:
/// - `GET /chat` - Braid subscription or page rendering
/// - `PUT /chat` - Braid PUT for adding messages
/// - `POST /typing` - Typing indicator events
/// - `GET /realtime` - Generic real-time event subscription
/// 
/// # Arguments
/// 
/// * `router` - The router to add routes to
/// 
/// # Returns
/// 
/// Router with chat routes configured
/// 
/// # Route Details
/// 
/// ## GET /chat
/// 
/// This route handles two different cases:
/// - If `Subscribe:` header is present: Returns a Braid subscription stream
/// - Otherwise: Renders the Leptos chat page
/// 
/// ## PUT /chat
/// 
/// Accepts new messages via the Braid PUT protocol. Requires authentication
/// and includes usage limit checking.
/// 
/// ## POST /typing
/// 
/// Receives typing indicator events and broadcasts them to all subscribers.
/// 
/// ## GET /realtime
/// 
/// Provides a generic real-time event subscription stream supporting
/// multiple event types (messages, notifications, status, typing).
#[cfg(feature = "ssr")]
pub fn configure_chat_routes(router: Router<AppState>) -> Router<AppState> {
    // Note: /chat GET handler is now handled in router.rs to avoid duplication
    // This function is kept for potential future use or other chat-related routes
    
    router
        // PUT /chat: always handles Braid PUT
        .route(
            "/chat",
            axum::routing::put(handle_braid_put),
        )
        // Generic real-time update endpoint
        // Supports multiple event types: messages, notifications, status updates, etc.
        .route(
            "/realtime",
            axum::routing::get(handle_realtime_subscription),
        )
        // Typing indicator endpoint
        .route(
            "/typing",
            axum::routing::post(handle_typing_event),
        )
}

