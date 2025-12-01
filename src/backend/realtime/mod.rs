//! Real-time Update Module
//!
//! This module provides a generic real-time update system that can handle
//! multiple types of events: messages, notifications, status updates, typing
//! indicators, etc.
//!
//! # Architecture
//!
//! The realtime module is organized into focused submodules:
//!
//! - **`broadcast`** - Event broadcasting utilities and type definitions
//! - **`subscription`** - Server-Sent Events subscription handler
//!
//! # Module Structure
//!
//! ```
//! realtime/
//! ├── mod.rs          - Module exports and documentation
//! ├── broadcast.rs    - Event broadcasting utilities
//! └── subscription.rs - SSE subscription handler
//! ```
//!
//! # Real-time System
//!
//! The real-time system uses Server-Sent Events (SSE) to provide one-way
//! communication from server to client. This is simpler than WebSockets
//! for one-way communication and works well with HTTP/2.
//!
//! # Event Types
//!
//! The system supports multiple event types:
//! - `Message` - Chat messages
//! - `Notification` - User notifications
//! - `Status` - Status updates
//! - `Typing` - Typing indicators
//! - `Custom` - Custom event types
//!
//! # Event Filtering
//!
//! Clients can filter events by type using the `types` query parameter:
//! - `?types=message,notification` - Subscribe to messages and notifications
//! - `?types=typing` - Subscribe only to typing events
//! - No parameter - Subscribe to all event types
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::realtime::{handle_realtime_subscription, broadcast_event, RealtimeEventBroadcast};
//! use braid_site::shared::RealtimeEvent;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Subscribe to events
//! // let sse = handle_realtime_subscription(State(broadcast_tx), headers, query).await?;
//!
//! // Broadcast an event
//! let event = RealtimeEvent::notification("Title".to_string(), "Body".to_string());
//! // broadcast_event(&broadcast_tx, event).await;
//! # Ok(())
//! # }
//! ```
//!
//! # Dependencies
//!
//! - `shared::RealtimeEvent` - Event data structure
//! - `shared::EventType` - Event type enumeration

/// Event broadcasting utilities
pub mod broadcast;

/// Server-Sent Events subscription handler
pub mod subscription;

// Re-export commonly used types and functions
pub use broadcast::{RealtimeEventBroadcast, broadcast_event};
#[cfg(feature = "ssr")]
pub use subscription::handle_realtime_subscription;

