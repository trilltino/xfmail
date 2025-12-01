//! Chat Handlers Module
//!
//! This module contains all Axum handlers for chat-related endpoints,
//! including Braid protocol handlers and typing indicators.
//!
//! # Architecture
//!
//! The handlers module is organized into focused submodules:
//!
//! - **`subscription`** - Braid subscription handler (GET /chat with Subscribe header)
//! - **`put`** - Braid PUT handler (PUT /chat for adding messages)
//! - **`typing`** - Typing indicator handler (POST /typing)
//!
//! # Module Structure
//!
//! ```
//! handlers/
//! ├── mod.rs          - Module exports and documentation
//! ├── subscription.rs - Braid subscription handler
//! ├── put.rs          - Braid PUT handler
//! └── typing.rs       - Typing indicator handler
//! ```
//!
//! # Braid Protocol
//!
//! The handlers implement the Braid HTTP protocol specification
//! (draft-toomim-httpbis-braid-http-04) for real-time state synchronization.
//!
//! Key features:
//! - Subscribe header support
//! - Structured Headers format (RFC 8941) for Version/Parents headers
//! - CRLF line endings (\r\n) throughout
//! - Proper Cache-Control and X-Accel-Buffering headers
//! - 209 Subscription status code for subscriptions
//!
//! # Route Handlers
//!
//! ## GET /chat (with Subscribe header)
//!
//! Returns a Braid subscription stream that sends real-time updates
//! as new messages arrive. Supports reconnection with Parents header.
//!
//! ## PUT /chat (with Parents header)
//!
//! Accepts new messages via the Braid PUT protocol. Requires authentication
//! and includes usage limit checking.
//!
//! ## POST /typing
//!
//! Receives typing indicator events and broadcasts them to all subscribers.
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::chat::handlers::{
//!     handle_braid_subscription,
//!     handle_braid_put,
//!     handle_typing_event,
//! };
//! use axum::{routing::get, routing::put, routing::post, Router};
//!
//! # async fn example() {
//! let router = Router::new()
//!     .route("/chat", get(handle_braid_subscription).put(handle_braid_put))
//!     .route("/typing", post(handle_typing_event));
//! # }
//! ```
//!
//! # Dependencies
//!
//! - `backend::server::state` - Application state
//! - `backend::realtime` - Real-time event broadcasting
//! - `backend::auth` - Authentication verification
//! - `backend::subscription` - Usage limit checking
//! - `backend::chat::db` - Database persistence
//! - `shared::Message` - Message data structure
//! - `shared::RealtimeEvent` - Real-time event types

/// Braid subscription handler
pub mod subscription;

/// Braid PUT handler
pub mod put;

/// Typing indicator handler
pub mod typing;

// Re-export commonly used handlers
#[cfg(feature = "ssr")]
pub use subscription::handle_braid_subscription;
#[cfg(feature = "ssr")]
pub use put::handle_braid_put;
#[cfg(feature = "ssr")]
pub use typing::handle_typing_event;

