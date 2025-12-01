/**
 * Server Initialization
 * 
 * This module handles the initialization and setup of the Axum HTTP server,
 * including state creation, database loading, and route configuration.
 * 
 * # Initialization Process
 * 
 * The server initialization follows these steps:
 * 1. Create chat state and broadcast channels
 * 2. Load optional services (database)
 * 3. Restore chat state from database if available
 * 4. Create and configure the router
 * 
 * # State Restoration
 * 
 * If a database is available, the server attempts to restore chat state
 * from persisted messages and version history. This allows the server
 * to maintain state across restarts.
 */

#[cfg(feature = "ssr")]
use axum::Router;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::{RwLock, broadcast};
#[cfg(feature = "ssr")]
#[cfg(feature = "ssr")]
use crate::backend::chat::state::ChatState;
#[cfg(feature = "ssr")]
use crate::backend::collab::state::CollabState;
#[cfg(feature = "ssr")]
use crate::backend::routes::router::create_router;
#[cfg(feature = "ssr")]
use crate::backend::server::state::{AppState, MessageEvent};
#[cfg(feature = "ssr")]
use crate::backend::server::config::load_database;

/// Create and configure the Axum application
///
/// This function sets up the Axum HTTP server with:
/// - Chat state initialization
/// - Database connection pool (if configured)
/// - Route configuration
///
/// # Returns
///
/// Configured Axum Router ready to serve requests
///
/// # Initialization Steps
///
/// 1. **Create Chat State**: Initializes an empty `ChatState` wrapped in `Arc<RwLock<>>`
/// 2. **Create Broadcast Channels**: Sets up channels for message and real-time event broadcasting
/// 3. **Load Services**: Attempts to load database from configuration
/// 4. **Restore State**: If database is available, loads persisted messages and version history
/// 5. **Create Router**: Configures all routes and middleware
///
/// # Error Handling
///
/// The function is designed to be resilient:
/// - Missing database: Server continues without database features
/// - Migration failures: Logged but don't prevent startup
/// - State restoration failures: Logged but don't prevent startup
#[cfg(feature = "ssr")]
pub async fn create_app() -> Router<()> {
    tracing::info!("Initializing XFCollab backend server");

    // Step 1: Create shared chat state
    // This stores messages and version history in memory
    // In a production app, this would be a database connection
    let chat_state = Arc::new(RwLock::new(ChatState::new()));

    // Step 1.5: Create shared collaborative editing state
    // This stores document CRDT state in memory
    let collab_state = Arc::new(RwLock::new(CollabState::new()));

    // Step 2: Create broadcast channels
    // Capacity of 1000 should be more than enough for a chat app
    let (message_broadcast, _) = broadcast::channel::<MessageEvent>(1000);

    // Create generic real-time event broadcast channel
    // This can handle any type of real-time event: messages, notifications, status updates, etc.
    let (realtime_broadcast, _) = broadcast::channel::<crate::shared::RealtimeEvent>(1000);

    tracing::info!("Chat state and broadcast channels initialized");

    // Step 3: Load optional services
    let db_pool = load_database().await;

    // Step 4: Restore chat state from database if available
    if let Some(pool) = &db_pool {
        restore_chat_state(pool, &chat_state).await;
    }

    // Step 5: Create app state
    let app_state = AppState {
        chat_state,
        collab_state,
        message_broadcast,
        realtime_broadcast,
        db_pool,
        messaging_broadcast: crate::backend::server::state::MessagingBroadcastState::new(),
        messaging_crdt: crate::backend::server::state::MessagingCrdtState::new(),
    };

    // Step 6: Create router with all routes
    let app = create_router(app_state.clone());

    // Step 7: Start periodic cleanup task for broadcast channels
    let cleanup_state = app_state.messaging_broadcast.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            cleanup_state.cleanup_inactive_channels();
            tracing::debug!("Cleaned up inactive messaging broadcast channels");
        }
    });

    tracing::info!("Router configured with periodic cleanup task");

    app
}

/// Restore chat state from database
/// 
/// This function loads persisted messages and version history from the database
/// and reconstructs the chat state. This allows the server to maintain state
/// across restarts.
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `chat_state` - The chat state to restore into
/// 
/// # Process
/// 
/// 1. Load all messages from database (ordered by creation time)
/// 2. Load version history (mapping version IDs to parent versions)
/// 3. Reconstruct chat state by adding messages in order with their parent versions
/// 
/// # Error Handling
/// 
/// Errors are logged but don't prevent server startup. If restoration fails,
/// the server starts with an empty chat state.
#[cfg(feature = "ssr")]
async fn restore_chat_state(
    pool: &sqlx::PgPool,
    chat_state: &Arc<RwLock<ChatState>>,
) {
    use crate::backend::chat::db::{load_messages, load_version_history};
    
    tracing::info!("Loading messages and version history from database...");
    
    let messages = match load_messages(pool).await {
        Ok(messages) => {
            tracing::info!("Loaded {} messages from database", messages.len());
            messages
        }
        Err(e) => {
            tracing::warn!("Failed to load messages from database (tables may not exist yet): {:?}", e);
            tracing::warn!("Starting with empty chat state - using Braid sync for messaging");
            Vec::new()
        }
    };
    
    let version_history = match load_version_history(pool).await {
        Ok(history) => {
            tracing::info!("Loaded {} version history entries from database", history.len());
            history
        }
        Err(e) => {
            tracing::warn!("Failed to load version history from database (tables may not exist yet): {:?}", e);
            tracing::warn!("Starting with empty version history");
            std::collections::HashMap::new()
        }
    };
    
    // Reconstruct ChatState with loaded data
    let mut loaded_state = ChatState::new();
    
    // Reconstruct messages in order, maintaining version history
    for message in messages {
        if let Some(ref version_id) = message.version {
            // Get parent versions from version history
            let parent_versions = version_history.get(version_id)
                .cloned()
                .unwrap_or_default();
            
            // Add message to state with its parent versions
            loaded_state.add_message(message, Some(parent_versions));
        } else {
            // If message has no version, add it without parents (shouldn't happen in practice)
            tracing::warn!("Message without version found, skipping: {}", message.text);
        }
    }
    
    // Update chat_state with loaded data
    *chat_state.write().await = loaded_state;
    tracing::info!("Chat state restored from database");
}


