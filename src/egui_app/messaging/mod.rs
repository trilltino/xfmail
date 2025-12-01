//! Messaging Module
//!
//! This module contains all the components for the Telegram-style messaging UI.

pub mod state;
pub mod main_layout;
pub mod sidebar;
pub mod chat_area;
pub mod components;
pub mod braid_sync;
pub mod friend_api;

pub use state::MessagingState;
pub use main_layout::render_messaging_view;
pub use braid_sync::MessageSyncClient;

