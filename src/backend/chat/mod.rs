//! Chat Backend Module
//!
//! This module contains all server-side chat functionality, including:
//! - Chat state management (message storage, version tracking)
//! - Braid protocol handlers (subscription and PUT endpoints)
//! - Database operations for message persistence
//!
//! The chat state is stored in memory and persisted to database when available.
//!
//! # Architecture
//!
//! The chat module is organized into focused submodules:
//!
//! - **`state`** - Chat state management (messages, version DAG)
//! - **`handlers`** - Braid protocol handlers (GET/PUT /chat)
//! - **`db`** - Database operations for persistence
//!
//! # Example
//! //!
//! ```rust,no_run
//! use braid_site::backend::chat::state::ChatState;
//! use braid_site::shared::Message;
//!
//! # async fn example() {
//! let mut state = ChatState::default();
//! let message = Message::new("Hello!".to_string(), "User123".to_string());
//! let version = state.add_message(message, None);
//! # }
//! ```


/// Chat state management
pub mod state;


/// Braid protocol handlers
pub mod handlers;


/// Database operations for chat messages and version history
#[cfg(feature = "ssr")]
pub mod db;


/// Re-export commonly used types
pub use state::ChatState;
pub use handlers::{handle_braid_subscription, handle_braid_put};