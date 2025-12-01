//! # Optimistic UI Updates
//!
//! Provides optimistic UI updates for offline-first interactions.
//! Updates the UI immediately while queuing operations for background execution.
//!
//! ## Features
//!
//! - **Immediate UI Updates**: UI responds instantly to user actions
//! - **Rollback Support**: Revert optimistic updates on failure
//! - **Confirmation**: Confirm successful operations
//! - **State Tracking**: Track pending optimistic updates
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::offline::OptimisticManager;
//!
//! let mut manager = OptimisticManager::new();
//!
//! // Apply optimistic update
//! manager.apply_update(operation).await;
//!
//! // Confirm successful operation
//! manager.confirm_operation(&operation_id).await;
//!
//! // Get updates for UI rendering
//! let updates = manager.get_pending_updates().await;
//! ```

use crate::egui_app::offline::queue::Operation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Optimistic update manager
#[derive(Debug)]
pub struct OptimisticManager {
    /// Pending optimistic updates
    updates: RwLock<HashMap<Uuid, OptimisticUpdate>>,
}

/// Optimistic update for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimisticUpdate {
    /// Unique operation ID
    pub id: Uuid,
    /// Operation type
    pub operation: Operation,
    /// Timestamp when applied
    pub applied_at: String,
    /// UI state to display
    pub ui_state: UiState,
}

/// UI state for optimistic rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiState {
    /// Optimistically added message
    MessageAdded {
        conversation_id: Uuid,
        message_content: String,
        temporary_id: Uuid,
    },
    /// Optimistically added contact
    ContactAdded {
        user_id: Uuid,
        username: String,
    },
    /// Optimistically accepted friend request
    FriendRequestAccepted {
        user_id: Uuid,
    },
    /// Optimistically sent friend request
    FriendRequestSent {
        user_id: Uuid,
    },
}

impl OptimisticManager {
    /// Create a new optimistic manager
    pub fn new() -> Self {
        Self {
            updates: RwLock::new(HashMap::new()),
        }
    }

    /// Apply an optimistic update
    pub async fn apply_update(&self, operation: Operation) {
        let update_id = operation.id;
        let applied_at = chrono::Utc::now().to_rfc3339();

        let ui_state = match &operation {
            Operation::SendMessage { conversation_id, content, .. } => {
                UiState::MessageAdded {
                    conversation_id: *conversation_id,
                    message_content: content.clone(),
                    temporary_id: update_id,
                }
            }
            Operation::AddContact { user_id, username, .. } => {
                UiState::ContactAdded {
                    user_id: *user_id,
                    username: username.clone(),
                }
            }
            Operation::AcceptFriendRequest { user_id, .. } => {
                UiState::FriendRequestAccepted {
                    user_id: *user_id,
                }
            }
            Operation::SendFriendRequest { user_id, .. } => {
                UiState::FriendRequestSent {
                    user_id: *user_id,
                }
            }
        };

        let update = OptimisticUpdate {
            id: update_id,
            operation,
            applied_at,
            ui_state,
        };

        let mut updates = self.updates.write().await;
        updates.insert(update_id, update);
    }

    /// Confirm a successful operation
    pub async fn confirm_operation(&self, operation_id: &Uuid) {
        let mut updates = self.updates.write().await;
        updates.remove(operation_id);
        // In a real implementation, this would also update the UI to show
        // the confirmed state (e.g., change temporary message to confirmed)
    }

    /// Rollback a failed operation
    pub async fn rollback_operation(&self, operation_id: &Uuid) {
        let mut updates = self.updates.write().await;
        updates.remove(operation_id);
        // In a real implementation, this would trigger UI updates to
        // remove the optimistic changes
    }

    /// Get all pending optimistic updates
    pub async fn get_pending_updates(&self) -> Vec<OptimisticUpdate> {
        let updates = self.updates.read().await;
        updates.values().cloned().collect()
    }

    /// Get updates for a specific conversation
    pub async fn get_conversation_updates(&self, conversation_id: &Uuid) -> Vec<OptimisticUpdate> {
        let updates = self.updates.read().await;
        updates
            .values()
            .filter(|update| match &update.ui_state {
                UiState::MessageAdded { conversation_id: conv_id, .. } => conv_id == conversation_id,
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Check if an operation has an optimistic update
    pub async fn has_optimistic_update(&self, operation_id: &Uuid) -> bool {
        let updates = self.updates.read().await;
        updates.contains_key(operation_id)
    }

    /// Get the count of pending optimistic updates
    pub async fn count_pending(&self) -> usize {
        let updates = self.updates.read().await;
        updates.len()
    }

    /// Clear all optimistic updates (e.g., on app restart)
    pub async fn clear_all(&self) {
        let mut updates = self.updates.write().await;
        updates.clear();
    }

    /// Clean up old optimistic updates
    pub async fn cleanup_old_updates(&self, max_age_seconds: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(max_age_seconds);

        let mut updates = self.updates.write().await;
        updates.retain(|_, update| {
            if let Ok(applied_time) = chrono::DateTime::parse_from_rfc3339(&update.applied_at) {
                applied_time > cutoff
            } else {
                true // Keep if we can't parse the timestamp
            }
        });
    }
}

impl Default for OptimisticManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimistic_manager_creation() {
        let manager = OptimisticManager::new();
        assert_eq!(manager.count_pending().await, 0);
    }

    #[tokio::test]
    async fn test_apply_and_confirm_update() {
        let manager = OptimisticManager::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Apply update
        manager.apply_update(operation.clone()).await;
        assert_eq!(manager.count_pending().await, 1);

        // Confirm operation
        manager.confirm_operation(&operation.id).await;
        assert_eq!(manager.count_pending().await, 0);
    }

    #[tokio::test]
    async fn test_rollback_update() {
        let manager = OptimisticManager::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Apply update
        manager.apply_update(operation.clone()).await;
        assert_eq!(manager.count_pending().await, 1);

        // Rollback operation
        manager.rollback_operation(&operation.id).await;
        assert_eq!(manager.count_pending().await, 0);
    }

    #[tokio::test]
    async fn test_conversation_updates() {
        let manager = OptimisticManager::new();
        let conversation_id = Uuid::new_v4();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id,
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        manager.apply_update(operation).await;

        let updates = manager.get_conversation_updates(&conversation_id).await;
        assert_eq!(updates.len(), 1);

        let other_conversation = Uuid::new_v4();
        let other_updates = manager.get_conversation_updates(&other_conversation).await;
        assert_eq!(other_updates.len(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_updates() {
        let manager = OptimisticManager::new();

        // This test would need to manipulate timestamps to be meaningful
        // For now, just ensure the method doesn't panic
        manager.cleanup_old_updates(3600).await;
        assert_eq!(manager.count_pending().await, 0);
    }
}