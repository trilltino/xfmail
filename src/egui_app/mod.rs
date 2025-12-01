//! egui Native Desktop App Module
//!
//! This module provides a native desktop application using egui/eframe
//! that connects to the Axum backend for authentication and demos.
//!
//! # Architecture
//!
//! The egui_app module is organized into focused submodules:
//!
//! - **`config`** - Configuration management (server URL, token storage)
//! - **`auth`** - Authentication UI and API client functions
//! - **`types`** - Shared types and app state enums
//! - **`braid_client`** - Braid HTTP protocol client
//! - **`local_db`** - Local SQLite database for offline functionality
//! - **`messaging_demo`** - Messaging demo placeholder
//! - **`editing_demo`** - Editing demo placeholder
//! - **`main`** - Main application entry point (binary)
//!
//! # Module Structure
//!
//! ```
//! egui_app/
//! ├── mod.rs          - Module exports and documentation
//! ├── main.rs         - Main application entry point
//! ├── config.rs       - Configuration management
//! ├── auth.rs         - Authentication UI and functions
//! ├── types.rs        - Shared types
//! ├── braid_client.rs - Braid HTTP client
//! ├── messaging_demo.rs - Messaging demo placeholder
//! └── editing_demo.rs   - Editing demo placeholder
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! // Run the egui app:
//! // cargo run --bin egui_app
//! ```

pub mod config;
pub mod auth;
pub mod types;
pub mod braid_client;
pub mod local_db;
pub mod messaging_demo;
pub mod editing_demo;
pub mod state;
pub mod views;
pub mod debug;
pub mod theme;
pub mod messaging;
pub mod local_auth;
mod crdt;

// Re-export commonly used types
pub use config::Config;
pub use auth::{AuthState, login, signup, get_me};
pub use types::{AppView, UserInfo};
pub use state::AppState;
pub use debug::{DebugLogger, DebugLevel, DebugCategory};

