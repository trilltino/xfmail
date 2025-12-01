//! Collaborative Editing Module
//!
//! This module contains all server-side functionality for collaborative text editing
//! using diamond-types CRDT. It includes:
//! - CRDT state management (OpLog storage per document)
//! - Braid protocol handlers for collaborative editing
//! - Version tracking and synchronization
//!
//! # Architecture
//!
//! The collab module is organized into focused submodules:
//!
//! - **`state`** - CRDT state management (OpLogs per document, version mapping)
//! - **`handlers`** - Braid protocol handlers (GET/PUT /collab/:doc_id)
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::collab::state::CollabState;
//!
//! let state = CollabState::new();
//! let doc_id = "doc-123".to_string();
//! state.create_document(doc_id.clone()).await;
//! ```

/// CRDT state management
pub mod state;

/// Braid protocol handlers for collaborative editing
pub mod handlers;

/// Re-export commonly used types
#[cfg(feature = "ssr")]
pub use state::CollabState;
#[cfg(feature = "ssr")]
pub use handlers::{handle_collab_subscription, handle_collab_put};

