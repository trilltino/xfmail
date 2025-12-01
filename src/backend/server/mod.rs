//! Server Module
//!
//! This module contains all server-side code for initializing and configuring
//! the Axum HTTP server. It provides the foundation for the application's
//! backend infrastructure.
//!
//! # Architecture
//!
//! The server module is organized into focused submodules:
//!
//! - **`state`** - Application state structure and `FromRef` implementations
//! - **`config`** - Configuration loading and validation
//! - **`init`** - Server initialization and app creation
//!
//! # Module Structure
//!
//! ```
//! server/
//! ├── mod.rs          - Module exports and documentation
//! ├── state.rs        - AppState and FromRef implementations
//! ├── config.rs       - Configuration loading (database, Stripe)
//! └── init.rs         - Server initialization and app creation
//! ```
//!
//! # State Management
//!
//! The server uses `AppState` as the central state container, which holds:
//! - Leptos configuration options
//! - Chat state (messages, version history)
//! - Broadcast channels for real-time updates
//! - Optional services (Stripe, database)
//!
//! State is shared across all request handlers using `Arc` and `RwLock` for
//! thread-safe concurrent access.
//!
//! # Initialization Flow
//!
//! 1. **Configuration Loading**: Loads database and Stripe configuration
//! 2. **State Creation**: Creates chat state and broadcast channels
//! 3. **State Restoration**: Restores chat state from database if available
//! 4. **Background Tasks**: Starts AI bot and other background tasks
//! 5. **Router Creation**: Configures all routes and middleware
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::server::create_app;
//! use braid_site::frontend::app::{App, shell};
//! use leptos::get_configuration;
//!
//! # async fn example() {
//! let conf = get_configuration(Some("Cargo.toml")).unwrap();
//! let app = create_app(conf.leptos_options, App, shell).await;
//! # }
//! ```
//!
//! # Dependencies
//!
//! - `backend::chat::state` - Chat state management
//! - `backend::routes` - Route configuration
//! - `backend::realtime` - Real-time event broadcasting
//! - `backend::ai_bot` - AI bot integration (optional)

/// Application state management
pub mod state;

/// Server configuration loading
pub mod config;

/// Server initialization
pub mod init;

// Re-export commonly used types
#[cfg(feature = "ssr")]
pub use state::{AppState, MessageEvent};
#[cfg(feature = "ssr")]
pub use init::create_app;

