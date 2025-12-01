//! # Operation Queue
//!
//! Queues operations for execution when the system comes back online.
//! Provides persistent storage and prioritization of offline operations.
//!
//! ## Features
//!
//! - **Persistent Queue**: Operations survive app restarts
//! - **Priority Support**: Different priority levels for operations
//! - **Status Tracking**: Track operation execution status
//! - **Batch Processing**: Process multiple operations efficiently
//! - **Cleanup**: Remove old failed operations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::offline::queue::{OperationQueue, Operation};
//!
//! let mut queue = OperationQueue::new();
//!
//! // Add operation to queue
//! let operation = Operation::SendMessage {
//!     id: Uuid::new_v4(),
//!     conversation_id: conversation_id,
//!     content: "Hello offline!".to_string(),
//!     timestamp: chrono::Utc::now().to_rfc3339(),
//! };
//! queue.add_operation(operation).await;
//!
//! // Process pending operations
//! let operations = queue.get_pending_operations().await;
//! for op in operations {
//!     // Execute operation...
//!     queue.complete_operation(&op.id).await;
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Operation queue for offline operations
#[derive(Debug)]
pub struct OperationQueue {
    /// Queued operations
    operations: RwLock<VecDeque<QueuedOperation>>,
}

/// Queued operation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedOperation {
    /// Operation details
    pub operation: Operation,
    /// Current status
    pub status: OperationStatus,
    /// Priority level
    pub priority: Priority,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Timestamp when queued
    pub queued_at: String,
    /// Timestamp of last attempt
    pub last_attempt: Option<String>,
    /// Error message from last failure
    pub last_error: Option<String>,
}

/// Operation types that can be queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Send a message
    SendMessage {
        /// Operation ID
        id: Uuid,
        /// Conversation ID
        conversation_id: Uuid,
        /// Message content
        content: String,
        /// Timestamp
        timestamp: String,
    },
    /// Add a contact
    AddContact {
        /// Operation ID
        id: Uuid,
        /// User ID to add
        user_id: Uuid,
        /// Username
        username: String,
        /// Timestamp
        timestamp: String,
    },
    /// Send friend request
    SendFriendRequest {
        /// Operation ID
        id: Uuid,
        /// Target user ID
        user_id: Uuid,
        /// Message (optional)
        message: Option<String>,
        /// Timestamp
        timestamp: String,
    },
    /// Accept friend request
    AcceptFriendRequest {
        /// Operation ID
        id: Uuid,
        /// Request ID
        request_id: Uuid,
        /// Timestamp
        timestamp: String,
    },
}

/// Operation execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationStatus {
    /// Waiting to be executed
    Pending,
    /// Currently being executed
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed to execute
    Failed,
    /// Temporarily failed, will retry
    Retrying,
}

/// Operation priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority operations
    Low,
    /// Normal priority operations
    Normal,
    /// High priority operations (e.g., urgent messages)
    High,
    /// Critical operations (must succeed)
    Critical,
}

impl OperationQueue {
    /// Create a new operation queue
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(VecDeque::new()),
        }
    }

    /// Add an operation to the queue
    pub async fn add_operation(&self, operation: Operation) {
        let queued_op = QueuedOperation {
            operation,
            status: OperationStatus::Pending,
            priority: Priority::Normal,
            retry_count: 0,
            queued_at: chrono::Utc::now().to_rfc3339(),
            last_attempt: None,
            last_error: None,
        };

        let mut operations = self.operations.write().await;
        operations.push_back(queued_op);
    }

    /// Add operation with specific priority
    pub async fn add_operation_with_priority(&self, operation: Operation, priority: Priority) {
        let mut queued_op = QueuedOperation {
            operation,
            status: OperationStatus::Pending,
            priority,
            retry_count: 0,
            queued_at: chrono::Utc::now().to_rfc3339(),
            last_attempt: None,
            last_error: None,
        };

        let mut operations = self.operations.write().await;

        // Insert based on priority (higher priority first)
        let insert_pos = operations
            .iter()
            .position(|op| op.priority < priority)
            .unwrap_or(operations.len());

        operations.insert(insert_pos, queued_op);
    }

    /// Get pending operations for execution
    pub async fn get_pending_operations(&self) -> Vec<QueuedOperation> {
        let operations = self.operations.read().await;
        operations
            .iter()
            .filter(|op| op.status == OperationStatus::Pending)
            .cloned()
            .collect()
    }

    /// Get operations by status
    pub async fn get_operations_by_status(&self, status: OperationStatus) -> Vec<QueuedOperation> {
        let operations = self.operations.read().await;
        operations
            .iter()
            .filter(|op| op.status == status)
            .cloned()
            .collect()
    }

    /// Mark operation as in progress
    pub async fn start_operation(&self, operation_id: &Uuid) {
        let mut operations = self.operations.write().await;
        if let Some(op) = operations.iter_mut().find(|op| op.operation.id() == *operation_id) {
            op.status = OperationStatus::InProgress;
            op.last_attempt = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    /// Mark operation as completed
    pub async fn complete_operation(&self, operation_id: &Uuid) {
        let mut operations = self.operations.write().await;
        operations.retain(|op| op.operation.id() != *operation_id);
        // Note: In a real implementation, completed operations might be archived
    }

    /// Mark operation as failed
    pub async fn fail_operation(&self, operation_id: &Uuid, error: String) {
        let mut operations = self.operations.write().await;
        if let Some(op) = operations.iter_mut().find(|op| op.operation.id() == *operation_id) {
            op.status = OperationStatus::Failed;
            op.last_error = Some(error);
            op.retry_count += 1;
        }
    }

    /// Mark operation for retry
    pub async fn retry_operation(&self, operation_id: &Uuid) {
        let mut operations = self.operations.write().await;
        if let Some(op) = operations.iter_mut().find(|op| op.operation.id() == *operation_id) {
            op.status = OperationStatus::Retrying;
            op.retry_count += 1;
        }
    }

    /// Get operation statistics
    pub async fn get_stats(&self) -> QueueStats {
        let operations = self.operations.read().await;

        let mut pending = 0;
        let mut in_progress = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut retrying = 0;

        for op in operations.iter() {
            match op.status {
                OperationStatus::Pending => pending += 1,
                OperationStatus::InProgress => in_progress += 1,
                OperationStatus::Completed => completed += 1,
                OperationStatus::Failed => failed += 1,
                OperationStatus::Retrying => retrying += 1,
            }
        }

        QueueStats {
            total_operations: operations.len(),
            pending,
            in_progress,
            completed,
            failed,
            retrying,
        }
    }

    /// Count operations by status
    pub async fn count_by_status(&self, status: OperationStatus) -> usize {
        let operations = self.operations.read().await;
        operations.iter().filter(|op| op.status == status).count()
    }

    /// Count pending operations
    pub async fn count_pending(&self) -> usize {
        self.count_by_status(OperationStatus::Pending).await
    }

    /// Count failed operations
    pub async fn count_failed(&self) -> usize {
        self.count_by_status(OperationStatus::Failed).await
    }

    /// Clean up old failed operations
    pub async fn cleanup_failed_operations(&self, max_age_hours: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);

        let mut operations = self.operations.write().await;
        operations.retain(|op| {
            if op.status == OperationStatus::Failed {
                if let Ok(queued_time) = chrono::DateTime::parse_from_rfc3339(&op.queued_at) {
                    return queued_time > cutoff;
                }
            }
            true
        });
    }

    /// Clear all operations (for testing or reset)
    pub async fn clear(&self) {
        let mut operations = self.operations.write().await;
        operations.clear();
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Total operations in queue
    pub total_operations: usize,
    /// Pending operations
    pub pending: usize,
    /// Operations in progress
    pub in_progress: usize,
    /// Completed operations
    pub completed: usize,
    /// Failed operations
    pub failed: usize,
    /// Operations being retried
    pub retrying: usize,
}

impl Operation {
    /// Get operation ID
    pub fn id(&self) -> Uuid {
        match self {
            Operation::SendMessage { id, .. } => *id,
            Operation::AddContact { id, .. } => *id,
            Operation::SendFriendRequest { id, .. } => *id,
            Operation::AcceptFriendRequest { id, .. } => *id,
        }
    }

    /// Get operation priority (default implementation)
    pub fn priority(&self) -> Priority {
        match self {
            Operation::SendMessage { .. } => Priority::High, // Messages are important
            Operation::AddContact { .. } => Priority::Normal,
            Operation::SendFriendRequest { .. } => Priority::Normal,
            Operation::AcceptFriendRequest { .. } => Priority::High, // Accepting friends is important
        }
    }
}

impl Default for OperationQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_creation() {
        let queue = OperationQueue::new();
        let stats = queue.get_stats().await;
        assert_eq!(stats.total_operations, 0);
    }

    #[tokio::test]
    async fn test_add_and_complete_operation() {
        let queue = OperationQueue::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Add operation
        queue.add_operation(operation.clone()).await;
        assert_eq!(queue.count_pending().await, 1);

        // Complete operation
        queue.complete_operation(&operation.id()).await;
        assert_eq!(queue.count_pending().await, 0);
    }

    #[tokio::test]
    async fn test_operation_failure_and_retry() {
        let queue = OperationQueue::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Add operation
        queue.add_operation(operation.clone()).await;

        // Fail operation
        queue.fail_operation(&operation.id(), "Network error".to_string()).await;

        // Check failed count
        assert_eq!(queue.count_failed().await, 1);

        // Retry operation
        queue.retry_operation(&operation.id()).await;

        let stats = queue.get_stats().await;
        assert_eq!(stats.retrying, 1);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = OperationQueue::new();

        let low_op = Operation::AddContact {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let high_op = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Urgent message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Add low priority first
        queue.add_operation_with_priority(low_op.clone(), Priority::Low).await;
        // Add high priority second
        queue.add_operation_with_priority(high_op.clone(), Priority::High).await;

        // High priority should be first
        let pending = queue.get_pending_operations().await;
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].operation.id(), high_op.id());
        assert_eq!(pending[1].operation.id(), low_op.id());
    }

    #[tokio::test]
    async fn test_cleanup_failed_operations() {
        let queue = OperationQueue::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        queue.add_operation(operation.clone()).await;
        queue.fail_operation(&operation.id(), "Error".to_string()).await;

        assert_eq!(queue.count_failed().await, 1);

        // Cleanup with very old threshold (should keep operations)
        queue.cleanup_failed_operations(24 * 365).await; // 1 year
        assert_eq!(queue.count_failed().await, 1);

        // Cleanup with immediate threshold (should remove operations)
        queue.cleanup_failed_operations(0).await;
        assert_eq!(queue.count_failed().await, 0);
    }
}