//! Messaging Module
//!
//! This module handles friend requests, contacts, and messaging functionality.

pub mod handlers;
pub mod db;
#[cfg(feature = "ssr")]
pub mod message_sync;

pub use handlers::*;
#[cfg(feature = "ssr")]
pub use message_sync::*;

