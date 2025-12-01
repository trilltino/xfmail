/**
 * Application State Management
 * 
 * This module defines the application state structure and implements
 * the necessary `FromRef` traits for Axum state extraction.
 * 
 * # Architecture
 * 
 * The `AppState` struct serves as the central state container for the
 * application, holding:
 * - Leptos configuration options
 * - Chat state (messages, version history)
 * - Broadcast channels for real-time updates
 * - Optional services (database)
 * 
 * # Thread Safety
 * 
 * All state is designed to be thread-safe:
 * - `Arc<RwLock<ChatState>>` for concurrent chat state access
 * - `broadcast::Sender` for thread-safe message broadcasting
 * - `Option<T>` for optional services that may not be configured
 * 
 * # State Extraction
 * 
 * The `FromRef` implementations allow Axum handlers to extract specific
 * parts of the state without needing the entire `AppState`. This follows
 * Axum's recommended pattern for state management.
 * 
 * # Example
 * 
 * ```rust
 * use braid_site::backend::server::state::AppState;
 * use axum::extract::State;
 * 
 * async fn handler(State(state): State<AppState>) {
 *     // Access state fields
 *     let chat_state = state.chat_state.read().await;
 *     // ...
 * }
 * ```
 */

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
// use leptos::prelude::*;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::{RwLock, broadcast};
#[cfg(feature = "ssr")]
use crate::shared::Message;
#[cfg(feature = "ssr")]
use crate::backend::chat::state::ChatState;
#[cfg(feature = "ssr")]
use crate::backend::collab::state::CollabState;
#[cfg(feature = "ssr")]
use crate::backend::realtime::broadcast::RealtimeEventBroadcast;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use uuid::Uuid;
#[cfg(feature = "ssr")]
use crate::shared::messaging::ChatMessage;

/// Message broadcast event
///
/// This type represents a single broadcast event containing:
/// - A vector of all current messages
/// - The version ID associated with this update
///
/// When a new message is added, all subscribers receive this event
/// with the complete message list and the new version ID.
#[cfg(feature = "ssr")]
pub type MessageEvent = (Vec<Message>, String);

/// Broadcast state for messaging conversations
///
/// Manages per-conversation broadcast channels for real-time message delivery.
/// Each conversation gets its own broadcast channel to prevent cross-talk.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct MessagingBroadcastState {
    channels: Arc<std::sync::Mutex<HashMap<Uuid, broadcast::Sender<ChatMessage>>>>,
}

/// CRDT state for messaging conversations
///
/// Manages per-conversation CRDT state for conflict-free message synchronization.
/// Each conversation maintains its own MessageCrdt instance.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct MessagingCrdtState {
    // Placeholder for future CRDT implementation
    // Currently using basic broadcasting
}

#[cfg(feature = "ssr")]
impl MessagingBroadcastState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast sender for a conversation
    pub fn get_sender(&self, conversation_id: Uuid) -> broadcast::Sender<ChatMessage> {
        let mut channels = self.channels.lock().unwrap();
        channels.entry(conversation_id)
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    }

    /// Broadcast a message to all subscribers of a conversation
    pub fn broadcast(&self, conversation_id: Uuid, message: ChatMessage) {
        if let Some(sender) = self.channels.lock().unwrap().get(&conversation_id) {
            let _ = sender.send(message); // Ignore if no receivers
        }
    }

    /// Clean up inactive channels (no subscribers)
    pub fn cleanup_inactive_channels(&self) {
        self.channels.lock().unwrap().retain(|_, sender| {
            sender.receiver_count() > 0
        });
    }

    /// Get subscriber count for a conversation (for debugging)
    pub fn get_subscriber_count(&self, conversation_id: Uuid) -> usize {
        if let Some(sender) = self.channels.lock().unwrap().get(&conversation_id) {
            sender.receiver_count()
        } else {
            0
        }
    }
}

#[cfg(feature = "ssr")]
impl MessagingCrdtState {
    pub fn new() -> Self {
        Self {}
    }
}

/// Application state that holds both Leptos options and chat state
/// 
/// This struct serves as the central state container for the Axum application.
/// It implements `FromRef` for various types to allow Axum handlers to extract
/// specific parts of the state without needing the entire `AppState`.
/// 
/// # Fields
/// 
/// * `leptos_options` - Leptos configuration options for SSR
/// * `chat_state` - Shared chat state (messages, version history)
/// * `message_broadcast` - Broadcast channel for chat message updates
/// * `realtime_broadcast` - Broadcast channel for generic real-time events
/// * `db_pool` - Optional PostgreSQL database connection pool
/// 
/// # Thread Safety
/// 
/// All fields are designed for concurrent access:
/// - `Arc<RwLock<ChatState>>` allows multiple readers or a single writer
/// - `broadcast::Sender` is thread-safe and can be cloned
/// - Optional services are `Option<T>` which is thread-safe when `T` is `Send + Sync`
/// 
/// # Usage
/// 
/// ```rust
/// use braid_site::backend::server::state::AppState;
/// use axum::extract::State;
/// 
/// async fn handler(State(app_state): State<AppState>) {
///     // Access chat state
///     let chat_state = app_state.chat_state.read().await;
///     let messages = &chat_state.messages;
///     // ...
/// }
/// ```
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    /// Shared chat state containing messages and version history
    /// 
    /// This is wrapped in `Arc<RwLock<>>` to allow concurrent read access
    /// from multiple handlers while ensuring exclusive write access.
    pub chat_state: Arc<RwLock<ChatState>>,
    
    /// Shared collaborative editing state containing documents and CRDT state
    /// 
    /// This is wrapped in `Arc<RwLock<>>` to allow concurrent read access
    /// from multiple handlers while ensuring exclusive write access.
    pub collab_state: Arc<RwLock<CollabState>>,
    
    /// Broadcast channel for notifying all SSE subscribers of new messages
    /// 
    /// This is a chat-specific broadcast channel. When a new message is added,
    /// it's broadcast to all subscribers via this channel.
    pub message_broadcast: broadcast::Sender<MessageEvent>,
    
    /// Generic real-time event broadcast channel
    /// 
    /// This can handle any type of real-time event: messages, notifications,
    /// status updates, typing indicators, etc. It's more generic than
    /// `message_broadcast` and supports filtering by event type.
    pub realtime_broadcast: RealtimeEventBroadcast,
    
    /// Database connection pool
    ///
    /// This is `None` if the database is not configured (e.g., if
    /// `DATABASE_URL` environment variable is not set). Handlers should
    /// check for `None` before using the database.
    pub db_pool: Option<PgPool>,

    /// Messaging broadcast state for real-time message delivery
    ///
    /// Manages per-conversation broadcast channels for SSE subscriptions.
    /// When a new message is sent, it's broadcast to all active subscribers
    /// of that conversation.
    pub messaging_broadcast: MessagingBroadcastState,

    /// CRDT state for messaging conversations
    ///
    /// Manages per-conversation CRDT state for conflict-free message synchronization.
    /// Each conversation maintains its own MessageCrdt instance.
    pub messaging_crdt: MessagingCrdtState,
}



#[cfg(feature = "ssr")]
/// Implement FromRef for ChatState
/// 
/// This allows Axum handlers to extract `Arc<RwLock<ChatState>>` directly
/// from `AppState` using `State(Arc<RwLock<ChatState>>)`.
/// 
/// # Example
/// 
/// ```rust
/// use axum::extract::State;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use braid_site::backend::chat::state::ChatState;
/// 
/// async fn handler(State(chat_state): State<Arc<RwLock<ChatState>>>) {
///     let state = chat_state.read().await;
///     // Use state
/// }
/// ```
impl FromRef<AppState> for Arc<RwLock<ChatState>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.chat_state.clone()
    }
}

#[cfg(feature = "ssr")]
/// Implement FromRef for CollabState
/// 
/// This allows Axum handlers to extract `Arc<RwLock<CollabState>>` directly
/// from `AppState` using `State(Arc<RwLock<CollabState>>)`.
impl FromRef<AppState> for Arc<RwLock<CollabState>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.collab_state.clone()
    }
}

#[cfg(feature = "ssr")]
/// Implement FromRef for MessageEvent broadcast sender
/// 
/// This allows Axum handlers to extract the message broadcast sender
/// directly from `AppState`.
/// 
/// # Example
/// 
/// ```rust
/// use axum::extract::State;
/// use tokio::sync::broadcast;
/// use braid_site::backend::server::state::MessageEvent;
/// 
/// async fn handler(State(tx): State<broadcast::Sender<MessageEvent>>) {
///     tx.send((messages, version)).ok();
/// }
/// ```
impl FromRef<AppState> for broadcast::Sender<MessageEvent> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.message_broadcast.clone()
    }
}

#[cfg(feature = "ssr")]
/// Implement FromRef for RealtimeEventBroadcast
/// 
/// This allows Axum handlers to extract the real-time event broadcast
/// sender directly from `AppState`.
impl FromRef<AppState> for RealtimeEventBroadcast {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.realtime_broadcast.clone()
    }
}

#[cfg(feature = "ssr")]
/// Implement FromRef for Option<PgPool>
///
/// This allows Axum handlers to extract the optional database pool
/// directly from `AppState`.
impl FromRef<AppState> for Option<PgPool> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}

#[cfg(feature = "ssr")]
/// Implement FromRef for MessagingBroadcastState
///
/// This allows Axum handlers to extract the messaging broadcast state
/// directly from `AppState`.
impl FromRef<AppState> for MessagingBroadcastState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.messaging_broadcast.clone()
    }
}

