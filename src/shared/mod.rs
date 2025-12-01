//! Shared Module
//!
//! This module contains types and data structures that are shared between
//! the frontend and backend. These types are used for serialization and
//! communication over the Braid HTTP protocol and other APIs.
//!
//! # Overview
//!
//! The shared module provides platform-agnostic types that can be used
//! in both server and client code. All types are designed for serialization
//! and transmission over HTTP.

/// Message data structure
pub mod message;

/// Real-time event system
pub mod event;

/// Shared error types
pub mod error;

/// CRDT types for collaborative editing
pub mod crdt;

/// Version conversion bridge between diamond-types and Braid
pub mod version_bridge;

/// Application configuration
pub mod config;

/// Messaging types for Telegram-style chat
pub mod messaging;

/// Re-export commonly used types for convenience
pub use message::Message;
pub use event::{RealtimeEvent, EventType};
pub use error::SharedError;
pub use crdt::{CRDTOperation, DocumentState, CRDTPatch, ApplyOperationsRequest, ApplyOperationsResponse, DocumentMetadata};
pub use config::{AppConfig, AppConfigBuilder, ConfigError};

