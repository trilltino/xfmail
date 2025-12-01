// Increase recursion limit for complex async operations
#![recursion_limit = "256"]

//! XFMail - Main Library
//!
//! XFMail is a high-performance collaborative email application built with Rust,
//! featuring real-time synchronization via the Braid protocol and conflict-free
//! collaborative editing using diamond-types CRDT.
//!
//! # Overview
//!
//! This library provides the core functionality for XFMail, including:
//! - Real-time messaging with persistent memory
//! - Collaborative text editing using diamond-types CRDT
//! - Native desktop application via egui
//! - AI assistant integration with multi-provider support
//! - Braid protocol implementation for seamless synchronization
//!
//! # Module Structure
//!
//! The library is organized into three main modules:
//!
//! - **`shared`** - Types shared between frontend and backend
//!   - Message structures, event types, CRDT operations
//!   - Version conversion utilities
//!   - Error types
//!
//! - **`backend`** - Server-side code (only compiled with `ssr` feature)
//!   - Axum HTTP server with Braid protocol handlers
//!   - Chat and collaborative editing state management
//!   - Authentication, AI bot, Stripe integration
//!   - Database persistence and real-time broadcasting
//!
//! - **`egui_app`** - Native desktop app (egui/eframe)
//!   - Native desktop application
//!   - Authentication UI
//!   - Messaging and editing demos
//!   - Braid HTTP client
//!
//! # Feature Flags
//!
//! The library uses feature flags to control compilation:
//!
//! - **`ssr`** - Server-side rendering (enables backend modules)
//!   - Includes Axum server, database, Stripe, AI bot
//!   - Required for server builds
//!
//! # Usage
//!
//! ## Server-Side
//!
//! For server builds, use the `ssr` feature:
//!
//! ```rust,no_run
//! use xfmail::backend::server::init::create_app;
//!
//! # async fn example() {
//! let app = create_app().await;
//! // Use app with Axum server
//! # }
//! ```
//!
//! ## Native Desktop App
//!
//! For native builds (non-WASM), use the egui_app module:
//!
//! ```rust,no_run
//! use xfmail::egui_app::BraidApp;
//! use eframe::run_native;
//!
//! // Run the native desktop app
//! run_native("XFMail", options, Box::new(|_| Ok(Box::new(BraidApp::default()))));
//! ```
//!
//! # Architecture
//!
//! The application follows a modular architecture:
//!
//! - **Shared Types**: Platform-agnostic types for serialization
//! - **Backend**: Axum server with Braid protocol handlers
//! - **Native App**: egui desktop application
//!
//! For detailed architecture information, see [ARCHITECTURE.md](../ARCHITECTURE.md).
//!
//! # Protocol Support
//!
//! ## Braid Protocol
//!
//! XFCollab implements the Braid HTTP protocol (draft-toomim-httpbis-braid-http-04)
//! for real-time state synchronization:
//!
//! - Version DAG tracking
//! - Long-lived subscriptions (209 Subscription status)
//! - Reconnection with Parents headers
//! - Merge Types for conflict resolution
//!
//! ## Diamond-Types CRDT
//!
//! Collaborative editing uses diamond-types, a high-performance CRDT:
//!
//! - Operation Log (OpLog) for append-only change tracking
//! - Branches for document snapshots
//! - Frontiers for version tracking
//! - Agent IDs for unique session identification
//!
//! # Examples
//!
//! See the following for usage examples:
//!
//! - [README.md](../README.md) - Project overview and quick start
//! - [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture
//! - Module-level documentation for specific features
//!
//! # Thread Safety
//!
//! - **Server**: All state is thread-safe using `Arc<RwLock<>>` and `broadcast::Sender`
//! - **Client**: WASM is single-threaded; Leptos signals are thread-safe
//! - **Native**: egui is single-threaded immediate mode GUI
//!
//! # Error Handling
//!
//! The library uses Rust's standard error handling:
//!
//! - `Result<T, E>` for fallible operations
//! - `Option<T>` for optional values
//! - Custom error types in `shared::error` and `backend::error`
//!
//! # See Also
//!
//! - [README.md](../README.md) - Project overview
//! - [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture
//! - [reference/braid-spec/](../reference/braid-spec/) - Braid protocol specification
//! - [reference/diamond-types/](../reference/diamond-types/) - Diamond-types documentation
/// Shared types and data structures
pub mod shared;

/// Backend server-side code
#[cfg(feature = "ssr")]
pub mod backend;



/// egui native desktop app
/// Only compiled for native targets (not WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod egui_app;

/// Debug utilities (only in debug builds)
#[cfg(debug_assertions)]
pub mod debug;




