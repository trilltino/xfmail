/**
 * API Route Handlers
 * 
 * This module defines route handlers for API endpoints, including:
 * - Authentication endpoints (signup, login, get current user)
 * - Usage statistics endpoints
 * 
 * # Routes
 * 
 * ## Authentication
 * - `POST /api/auth/signup` - User registration
 * - `POST /api/auth/login` - User login
 * - `GET /api/auth/me` - Get current user info
 * 
 * ## Usage
 * - `GET /api/usage` - Get usage statistics (requires authentication)
 */

use axum::Router;
#[cfg(feature = "ssr")]
use crate::backend::server::state::AppState;
#[cfg(feature = "ssr")]
use crate::backend::auth::{signup, login, get_me};
#[cfg(feature = "ssr")]
use crate::backend::subscription::api::get_usage_stats;
#[cfg(feature = "ssr")]
use crate::backend::messaging::handlers::{
    send_friend_request, get_friend_requests, respond_to_friend_request, get_contacts,
    get_conversations, get_messages, mark_message_read,
};
#[cfg(feature = "ssr")]
use crate::backend::messaging::message_sync::{
    handle_message_subscription, handle_message_put,
};

/// Configure API routes
/// 
/// This function adds the following routes to the router:
/// 
/// ## Authentication Routes
/// - `POST /api/auth/signup` - User registration
/// - `POST /api/auth/login` - User login
/// - `GET /api/auth/me` - Get current user info (requires authentication)
/// 
/// ## Usage Routes
/// - `GET /api/usage` - Get usage statistics (requires authentication)
/// 
/// # Arguments
/// 
/// * `router` - The router to add routes to
/// 
/// # Returns
/// 
/// Router with API routes configured
/// 
/// # Authentication
/// 
/// Some routes require authentication:
/// - `/api/auth/me` - Requires JWT token in `Authorization` header
/// - `/api/usage` - Requires JWT token in `Authorization` header
/// 
/// Other routes are public:
/// - `/api/auth/signup` - Public (creates new user)
/// - `/api/auth/login` - Public (returns JWT token)
#[cfg(feature = "ssr")]
pub fn configure_api_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // Authentication endpoints
        .route(
            "/api/auth/signup",
            axum::routing::post(signup),
        )
        .route(
            "/api/auth/login",
            axum::routing::post(login),
        )
        .route(
            "/api/auth/me",
            axum::routing::get(get_me),
        )
        // Usage statistics endpoint (requires authentication - checked in handler)
        .route(
            "/api/usage",
            axum::routing::get(get_usage_stats),
        )
        // Friend request endpoints
        .route(
            "/api/friends/request",
            axum::routing::post(send_friend_request),
        )
        .route(
            "/api/friends/requests",
            axum::routing::get(get_friend_requests),
        )
        .route(
            "/api/friends/respond",
            axum::routing::post(respond_to_friend_request),
        )
        // Contacts endpoint
        .route(
            "/api/contacts",
            axum::routing::get(get_contacts),
        )
        // Conversations endpoints
        .route(
            "/api/conversations",
            axum::routing::get(get_conversations),
        )
        // Messages endpoints
        .route(
            "/api/conversations/{conversation_id}/messages",
            axum::routing::get(get_messages),
        )
        .route(
            "/api/messages/{message_id}/read",
            axum::routing::patch(mark_message_read),
        )
        // Message sync endpoints (Braid-HTTP)
        .route(
            "/sync/conversations/{conversation_id}/messages",
            axum::routing::get(handle_message_subscription),
        )
        .route(
            "/sync/conversations/{conversation_id}/messages/{message_id}",
            axum::routing::put(handle_message_put),
        )
}

