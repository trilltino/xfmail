//! # Offline Messaging System
//!
//! Provides offline-first messaging capabilities with optimistic UI updates,
//! operation queuing, and automatic reconciliation when connectivity returns.
//!
//! ## Architecture
//!
//! The offline system consists of:
//! - **Optimistic UI**: Immediate UI updates for user actions
//! - **Operation Queue**: Queues operations for execution when online
//! - **Reconciliation**: Merges local and remote state changes
//! - **Retry Logic**: Automatic retry with exponential backoff
//!
//! ## Key Components
//!
//! - `optimistic.rs`: Optimistic update management
//! - `queue.rs`: Operation queuing system
//! - `retry.rs`: Retry logic and backoff strategies
//! - `reconciliation.rs`: State reconciliation logic
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::offline::{OfflineManager, Operation};
//!
//! let mut offline_manager = OfflineManager::new();
//!
//! // Queue an operation for offline execution
//! let operation = Operation::SendMessage {
//!     conversation_id: conversation_id,
//!     content: "Hello offline!".to_string(),
//! };
//! offline_manager.queue_operation(operation).await;
//!
//! // Process pending operations when online
//! offline_manager.process_queue().await;
//! ```

pub mod optimistic;
pub mod queue;
pub mod retry;
pub mod reconciliation;

// Re-export main types
pub use optimistic::{OptimisticManager, OptimisticUpdate};
pub use queue::{OperationQueue, Operation, OperationStatus};
pub use retry::{RetryManager, BackoffStrategy};
pub use reconciliation::{ReconciliationManager, ReconciliationResult};

use crate::egui_app::local_db::LocalDatabase;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main offline manager coordinating all offline functionality
#[derive(Debug)]
pub struct OfflineManager {
    /// Local database for offline storage
    local_db: Arc<RwLock<LocalDatabase>>,
    /// Operation queue for offline operations
    queue: OperationQueue,
    /// Optimistic update manager
    optimistic: OptimisticManager,
    /// Retry manager for failed operations
    retry: RetryManager,
    /// Reconciliation manager
    reconciliation: ReconciliationManager,
    /// Network connectivity status
    is_online: Arc<RwLock<bool>>,
}

impl OfflineManager {
    /// Create a new offline manager
    pub fn new(local_db: Arc<RwLock<LocalDatabase>>) -> Self {
        Self {
            local_db,
            queue: OperationQueue::new(),
            optimistic: OptimisticManager::new(),
            retry: RetryManager::new(),
            reconciliation: ReconciliationManager::new(),
            is_online: Arc::new(RwLock::new(true)), // Assume online initially
        }
    }

    /// Check if the system is currently online
    pub async fn is_online(&self) -> bool {
        *self.is_online.read().await
    }

    /// Update online status
    pub async fn set_online(&self, online: bool) {
        let mut status = self.is_online.write().await;
        *status = online;

        if online {
            // Process queued operations when coming online
            self.process_queue().await;
        }
    }

    /// Queue an operation for offline execution
    pub async fn queue_operation(&mut self, operation: Operation) {
        if self.is_online().await {
            // Execute immediately if online
            match self.execute_operation(&operation).await {
                Ok(_) => {
                    // Operation succeeded
                    self.optimistic.confirm_operation(&operation.id).await;
                }
                Err(_) => {
                    // Operation failed, queue for retry
                    self.queue.add_operation(operation).await;
                }
            }
        } else {
            // Queue for later execution
            self.queue.add_operation(operation).await;
            // Apply optimistic update
            self.optimistic.apply_update(operation).await;
        }
    }

    /// Process the operation queue
    pub async fn process_queue(&self) {
        if !self.is_online().await {
            return; // Can't process if offline
        }

        let operations = self.queue.get_pending_operations().await;

        for operation in operations {
            match self.execute_operation(&operation).await {
                Ok(_) => {
                    // Success - remove from queue and confirm optimistic update
                    self.queue.complete_operation(&operation.id).await;
                    self.optimistic.confirm_operation(&operation.id).await;
                }
                Err(_) => {
                    // Failed - schedule retry
                    self.retry.schedule_retry(operation).await;
                }
            }
        }
    }

    /// Execute a single operation
    async fn execute_operation(&self, operation: &Operation) -> Result<(), String> {
        // This would integrate with the actual backend APIs
        // For now, simulate execution
        match operation {
            Operation::SendMessage { .. } => {
                // Simulate sending message
                Ok(())
            }
            Operation::AddContact { .. } => {
                // Simulate adding contact
                Ok(())
            }
            Operation::AcceptFriendRequest { .. } => {
                // Simulate accepting friend request
                Ok(())
            }
        }
    }

    /// Get optimistic updates for UI rendering
    pub async fn get_optimistic_updates(&self) -> Vec<OptimisticUpdate> {
        self.optimistic.get_pending_updates().await
    }

    /// Reconcile local and remote state
    pub async fn reconcile_state(&self) -> Result<ReconciliationResult, String> {
        // This would compare local and remote state and merge changes
        self.reconciliation.reconcile().await
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> QueueStats {
        let pending = self.queue.count_pending().await;
        let failed = self.queue.count_failed().await;
        let retrying = self.retry.count_retrying().await;

        QueueStats {
            pending_operations: pending,
            failed_operations: failed,
            retrying_operations: retrying,
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of pending operations
    pub pending_operations: usize,
    /// Number of failed operations
    pub failed_operations: usize,
    /// Number of operations currently being retried
    pub retrying_operations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egui_app::local_db::LocalDatabase;

    #[tokio::test]
    async fn test_offline_manager_creation() {
        let local_db = Arc::new(RwLock::new(LocalDatabase::new().unwrap()));
        let manager = OfflineManager::new(local_db);

        assert!(manager.is_online().await);
    }

    #[tokio::test]
    async fn test_online_status() {
        let local_db = Arc::new(RwLock::new(LocalDatabase::new().unwrap()));
        let manager = OfflineManager::new(local_db);

        // Test going offline
        manager.set_online(false).await;
        assert!(!manager.is_online().await);

        // Test coming back online
        manager.set_online(true).await;
        assert!(manager.is_online().await);
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let local_db = Arc::new(RwLock::new(LocalDatabase::new().unwrap()));
        let manager = OfflineManager::new(local_db);

        let stats = manager.get_queue_stats().await;
        assert_eq!(stats.pending_operations, 0);
        assert_eq!(stats.failed_operations, 0);
        assert_eq!(stats.retrying_operations, 0);
    }
}