//! Backend Module
//!
//! This module contains all server-side code for the XFCollab application.
//! It provides a complete Axum HTTP server with Braid protocol support,
//! real-time synchronization, and various integrations.
//!
//! # Overview
//!
//! The backend module includes:
//! - Axum HTTP server setup and configuration
//! - Braid protocol handlers (GET/PUT with version DAG)
//! - Chat and collaborative editing state management
//! - Route configuration and middleware
//! - Authentication and user management
//! - Real-time event broadcasting
//! - Database persistence (PostgreSQL)
//!
//! This module is only compiled when the `ssr` feature is enabled.
//! All code in this module runs on the server and handles HTTP requests.
//!
//! # Architecture
//!
//! The backend is organized into focused submodules:
//!
//! - **`server`** - Server initialization, application state, configuration
//! - **`routes`** - HTTP route configuration and router assembly
//! - **`chat`** - Chat message handling and Braid protocol for messaging
//! - **`collab`** - Collaborative editing with diamond-types CRDT
//! - **`auth`** - Authentication, JWT tokens, user management
//! - **`realtime`** - Generic real-time event broadcasting system
//! - **`subscription`** - Usage limit checking and tracking
//! - **`middleware`** - Request processing middleware
//! - **`error`** - Backend-specific error types
//!
//! # Module Structure
//!
//! ```
//! backend/
//! ├── mod.rs          - Module exports and documentation
//! ├── server/         - Server initialization and state
//! ├── routes/         - Route configuration
//! ├── chat/           - Chat handlers and state
//! ├── collab/         - Collaborative editing
//! ├── auth/           - Authentication
//! ├── realtime/       - Event broadcasting
//! ├── subscription/   - Usage limits
//! ├── middleware/     - Request middleware
//! └── error/          - Error types
//! ```
//!
//! # State Management
//!
//! The backend uses shared state (`AppState`) that contains:
//! - Leptos configuration options
//! - Chat state (messages, version history)
//! - Collaborative editing state (CRDT OpLogs per document)
//! - Broadcast channels for real-time updates
//! - Optional services (database)
//!
//! State is shared across all request handlers using `Arc` and `RwLock` for
//! thread-safe concurrent access. Broadcast channels use `tokio::sync::broadcast`
//! for efficient multi-subscriber messaging.
//!
//! # Protocol Support
//!
//! ## Braid Protocol
//!
//! The backend implements the Braid HTTP protocol (draft-toomim-httpbis-braid-http-04):
//!
//! - **GET with Subscribe**: Long-lived subscriptions (209 Subscription status)
//! - **PUT with Parents**: Version-aware updates with DAG tracking
//! - **Version Headers**: Structured Headers format (RFC 8941)
//! - **Reconnection**: Automatic catch-up using Parents headers
//!
//! Endpoints:
//! - `GET /chat` (with Subscribe header) - Message subscription
//! - `PUT /chat` (with Parents header) - Send message
//! - `GET /collab/:doc_id` (with Subscribe header) - Document subscription
//! - `PUT /collab/:doc_id` (with Parents header) - Send CRDT operations
//!
//! ## Diamond-Types CRDT
//!
//! Collaborative editing uses diamond-types for conflict-free merging:
//!
//! - One `ListOpLog` per document
//! - Version mapping between diamond-types Frontiers and Braid version IDs
//! - Thread-safe access via `Arc<RwLock<>>`
//!
//! # Thread Safety
//!
//! All backend code is designed for concurrent access:
//! - `Arc<RwLock<>>` for shared mutable state
//! - `broadcast::Sender` for thread-safe message broadcasting
//! - Axum handlers are `Send + Sync`
//! - Database pool is thread-safe
//!
//! # Error Handling
//!
//! The backend uses standard HTTP status codes and custom error types:
//! - `BackendError` for internal errors
//! - `StatusCode` for HTTP responses
//! - Proper error propagation with `?` operator
//!
//! # Example
//!
//! ```rust,no_run
//! use xfcollab::backend::server::create_app;
//! use xfcollab::frontend::app::{App, shell};
//! use leptos::get_configuration;
//!
//! # async fn example() {
//! let conf = get_configuration(Some("Cargo.toml")).unwrap();
//! let app = create_app(conf.leptos_options, App, shell).await;
//! // Use app with Axum server
//! # }
//! ```
//!
//! # See Also
//!
//! - [ARCHITECTURE.md](../../ARCHITECTURE.md) - System architecture
//! - [README.md](../../README.md) - Project overview
//! - Module-level documentation for specific features

/// Server setup and configuration
#[cfg(feature = "ssr")]
pub mod server;

/// Route configuration
#[cfg(feature = "ssr")]
pub mod routes;

/// Chat-related backend functionality
#[cfg(feature = "ssr")]
pub mod chat;

/// Real-time update system
#[cfg(feature = "ssr")]
pub mod realtime;

/// Backend error types
#[cfg(feature = "ssr")]
pub mod error;

/// Authentication and user management
#[cfg(feature = "ssr")]
pub mod auth;

/// Middleware for request processing
#[cfg(feature = "ssr")]
pub mod middleware;

/// Subscription management and limits
#[cfg(feature = "ssr")]
pub mod subscription;

/// Collaborative editing with diamond-types CRDT
#[cfg(feature = "ssr")]
pub mod collab;

/// Messaging and friend system
#[cfg(feature = "ssr")]
pub mod messaging;

/// Re-export commonly used types
#[cfg(feature = "ssr")]
pub use server::create_app;
#[cfg(feature = "ssr")]
pub use chat::state::ChatState;
#[cfg(feature = "ssr")]
pub use chat::handlers::{handle_braid_subscription, handle_braid_put};
#[cfg(feature = "ssr")]
pub use realtime::{handle_realtime_subscription, broadcast_event, RealtimeEventBroadcast};
#[cfg(feature = "ssr")]
pub use error::BackendError;
#[cfg(feature = "ssr")]
pub use collab::state::CollabState;

