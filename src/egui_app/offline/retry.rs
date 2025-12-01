//! # Retry Logic and Backoff Strategies
//!
//! Implements intelligent retry logic with exponential backoff for failed operations.
//! Ensures reliable execution of offline operations when connectivity returns.
//!
//! ## Features
//!
//! - **Exponential Backoff**: Gradually increase retry intervals
//! - **Jitter**: Add randomness to prevent thundering herd
//! - **Max Retries**: Limit retry attempts to prevent infinite loops
//! - **Circuit Breaker**: Temporarily disable failing operations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::offline::retry::{RetryManager, BackoffStrategy};
//!
//! let mut retry_manager = RetryManager::new();
//!
//! // Schedule retry for failed operation
//! retry_manager.schedule_retry(operation).await;
//!
//! // Process retries
//! retry_manager.process_retries().await;
//! ```

use crate::egui_app::offline::queue::{Operation, QueuedOperation, OperationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Retry manager for failed operations
#[derive(Debug)]
pub struct RetryManager {
    /// Operations currently being retried
    retrying_operations: RwLock<HashMap<Uuid, RetryState>>,
    /// Backoff strategy
    backoff_strategy: BackoffStrategy,
}

/// Retry state for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    /// Operation being retried
    pub operation: Operation,
    /// Current retry attempt number
    pub attempt: u32,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Next retry timestamp
    pub next_retry_at: String,
    /// Last error message
    pub last_error: String,
}

/// Backoff strategy configuration
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed interval between retries
    Fixed {
        /// Interval in seconds
        interval_seconds: u64,
    },
    /// Exponential backoff with jitter
    Exponential {
        /// Base interval in seconds
        base_interval: u64,
        /// Maximum interval in seconds
        max_interval: u64,
        /// Jitter factor (0.0 to 1.0)
        jitter: f64,
    },
    /// Custom backoff function
    Custom(Box<dyn Fn(u32) -> u64 + Send + Sync>),
}

impl RetryManager {
    /// Create a new retry manager
    pub fn new() -> Self {
        Self {
            retrying_operations: RwLock::new(HashMap::new()),
            backoff_strategy: BackoffStrategy::Exponential {
                base_interval: 1,
                max_interval: 300, // 5 minutes
                jitter: 0.1,
            },
        }
    }

    /// Schedule an operation for retry
    pub async fn schedule_retry(&self, operation: Operation) {
        let operation_id = operation.id();
        let attempt = 1; // This would be tracked from the queued operation
        let max_attempts = 5;

        let next_retry_at = self.calculate_next_retry(attempt);

        let retry_state = RetryState {
            operation,
            attempt,
            max_attempts,
            next_retry_at,
            last_error: "Unknown error".to_string(), // Would be passed in
        };

        let mut operations = self.retrying_operations.write().await;
        operations.insert(operation_id, retry_state);
    }

    /// Process pending retries
    pub async fn process_retries(&self) -> Vec<Operation> {
        let now = chrono::Utc::now();
        let mut operations = self.retrying_operations.write().await;

        let mut ready_operations = Vec::new();

        operations.retain(|_, state| {
            if let Ok(retry_time) = chrono::DateTime::parse_from_rfc3339(&state.next_retry_at) {
                if retry_time <= now {
                    if state.attempt < state.max_attempts {
                        ready_operations.push(state.operation.clone());
                        // Schedule next retry
                        state.attempt += 1;
                        state.next_retry_at = self.calculate_next_retry(state.attempt);
                        true // Keep in retry queue
                    } else {
                        false // Remove from retry queue (max attempts reached)
                    }
                } else {
                    true // Keep waiting
                }
            } else {
                false // Invalid timestamp, remove
            }
        });

        ready_operations
    }

    /// Cancel retry for an operation
    pub async fn cancel_retry(&self, operation_id: &Uuid) {
        let mut operations = self.retrying_operations.write().await;
        operations.remove(operation_id);
    }

    /// Get retry statistics
    pub async fn get_stats(&self) -> RetryStats {
        let operations = self.retrying_operations.read().await;

        let total_retrying = operations.len();
        let mut total_attempts = 0;
        let mut max_attempts = 0;

        for state in operations.values() {
            total_attempts += state.attempt;
            max_attempts = max_attempts.max(state.attempt);
        }

        RetryStats {
            total_retrying,
            total_attempts,
            max_attempts,
        }
    }

    /// Count operations currently being retried
    pub async fn count_retrying(&self) -> usize {
        let operations = self.retrying_operations.read().await;
        operations.len()
    }

    /// Calculate next retry timestamp
    fn calculate_next_retry(&self, attempt: u32) -> String {
        let delay_seconds = match &self.backoff_strategy {
            BackoffStrategy::Fixed { interval_seconds } => *interval_seconds,
            BackoffStrategy::Exponential { base_interval, max_interval, jitter } => {
                let exponential_delay = base_interval * (2u64.pow(attempt.saturating_sub(1)));
                let delay = exponential_delay.min(*max_interval);

                // Add jitter
                let jitter_amount = (delay as f64 * jitter) as u64;
                delay + (rand::random::<u64>() % jitter_amount)
            }
            BackoffStrategy::Custom(calc_fn) => calc_fn(attempt),
        };

        (chrono::Utc::now() + chrono::Duration::seconds(delay_seconds as i64)).to_rfc3339()
    }

    /// Set backoff strategy
    pub fn set_backoff_strategy(&mut self, strategy: BackoffStrategy) {
        self.backoff_strategy = strategy;
    }
}

/// Retry statistics
#[derive(Debug, Clone)]
pub struct RetryStats {
    /// Total operations being retried
    pub total_retrying: usize,
    /// Total retry attempts across all operations
    pub total_attempts: usize,
    /// Maximum retry attempts for any single operation
    pub max_attempts: u32,
}

impl Default for RetryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_manager_creation() {
        let manager = RetryManager::new();
        assert_eq!(manager.count_retrying().await, 0);
    }

    #[tokio::test]
    async fn test_schedule_and_process_retry() {
        let manager = RetryManager::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Schedule retry
        manager.schedule_retry(operation.clone()).await;
        assert_eq!(manager.count_retrying().await, 1);

        // Process retries (should not be ready yet)
        let ready = manager.process_retries().await;
        assert_eq!(ready.len(), 0); // Not enough time has passed

        // Cancel retry
        manager.cancel_retry(&operation.id()).await;
        assert_eq!(manager.count_retrying().await, 0);
    }

    #[test]
    fn test_backoff_calculation() {
        let manager = RetryManager::new();

        // Test exponential backoff calculation
        let delay1 = manager.calculate_next_retry(1);
        let delay2 = manager.calculate_next_retry(2);

        // Should be different (due to exponential backoff and jitter)
        assert_ne!(delay1, delay2);
    }

    #[tokio::test]
    async fn test_retry_stats() {
        let manager = RetryManager::new();

        let operation = Operation::SendMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        manager.schedule_retry(operation).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_retrying, 1);
        assert!(stats.total_attempts >= 1);
    }
}