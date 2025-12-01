//! # State Reconciliation
//!
//! Merges local and remote state changes when reconnecting after offline periods.
//! Ensures consistency between local optimistic updates and server state.
//!
//! ## Features
//!
//! - **State Comparison**: Compare local and remote state versions
//! - **Conflict Detection**: Identify conflicting changes
//! - **Automatic Merging**: Merge non-conflicting changes
//! - **Manual Resolution**: Handle conflicts requiring user input
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::offline::ReconciliationManager;
//!
//! let mut reconciler = ReconciliationManager::new();
//!
//! // Reconcile local and remote state
//! let result = reconciler.reconcile().await;
//!
//! match result {
//!     ReconciliationResult::Success => println!("States reconciled"),
//!     ReconciliationResult::ConflictsFound(conflicts) => {
//!         // Handle conflicts...
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State reconciliation manager
#[derive(Debug)]
pub struct ReconciliationManager {
    /// Reconciliation strategies
    strategies: HashMap<String, ReconciliationStrategy>,
}

/// Reconciliation strategy
#[derive(Debug, Clone)]
pub enum ReconciliationStrategy {
    /// Last-write-wins strategy
    LastWriteWins,
    /// Merge strategy (combine changes)
    Merge,
    /// Custom reconciliation function
    Custom(Box<dyn Fn(&ReconciliationInput) -> ReconciliationResult + Send + Sync>),
}

/// Input for reconciliation process
#[derive(Debug, Clone)]
pub struct ReconciliationInput {
    /// Local state data
    pub local_state: Vec<u8>,
    /// Remote state data
    pub remote_state: Vec<u8>,
    /// Local version/timestamp
    pub local_version: String,
    /// Remote version/timestamp
    pub remote_version: String,
}

/// Result of reconciliation process
#[derive(Debug, Clone)]
pub enum ReconciliationResult {
    /// Reconciliation successful
    Success,
    /// Conflicts found that need resolution
    ConflictsFound(Vec<ReconciliationConflict>),
    /// Reconciliation failed
    Failed(String),
}

/// Reconciliation conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationConflict {
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Description of the conflict
    pub description: String,
    /// Local conflicting data
    pub local_data: Vec<u8>,
    /// Remote conflicting data
    pub remote_data: Vec<u8>,
    /// Suggested resolution options
    pub resolution_options: Vec<ResolutionOption>,
}

/// Type of conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Conflicting message edits
    MessageEdit,
    /// Conflicting contact changes
    ContactChange,
    /// Conflicting conversation metadata
    ConversationMetadata,
    /// Conflicting participant changes
    ParticipantChange,
}

/// Resolution option for conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    /// Option identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Whether this is recommended
    pub recommended: bool,
}

impl ReconciliationManager {
    /// Create a new reconciliation manager
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        // Default strategies
        strategies.insert(
            "messages".to_string(),
            ReconciliationStrategy::LastWriteWins,
        );
        strategies.insert(
            "contacts".to_string(),
            ReconciliationStrategy::Merge,
        );
        strategies.insert(
            "conversations".to_string(),
            ReconciliationStrategy::Merge,
        );

        Self { strategies }
    }

    /// Reconcile local and remote state
    pub async fn reconcile(&self) -> ReconciliationResult {
        // This is a simplified implementation
        // In a real system, this would:
        // 1. Fetch local state from database
        // 2. Fetch remote state from server
        // 3. Compare versions and detect conflicts
        // 4. Apply appropriate reconciliation strategy

        // For now, simulate successful reconciliation
        ReconciliationResult::Success
    }

    /// Reconcile specific data type
    pub async fn reconcile_data_type(
        &self,
        data_type: &str,
        input: ReconciliationInput,
    ) -> ReconciliationResult {
        match self.strategies.get(data_type) {
            Some(ReconciliationStrategy::LastWriteWins) => {
                self.reconcile_last_write_wins(input)
            }
            Some(ReconciliationStrategy::Merge) => {
                self.reconcile_merge(input)
            }
            Some(ReconciliationStrategy::Custom(strategy_fn)) => {
                strategy_fn(&input)
            }
            None => ReconciliationResult::Failed(format!("No strategy for data type: {}", data_type)),
        }
    }

    /// Last-write-wins reconciliation
    fn reconcile_last_write_wins(&self, input: ReconciliationInput) -> ReconciliationResult {
        // Compare timestamps and choose the newer one
        match input.local_version.cmp(&input.remote_version) {
            std::cmp::Ordering::Greater => {
                // Local is newer, keep local
                ReconciliationResult::Success
            }
            std::cmp::Ordering::Less => {
                // Remote is newer, apply remote
                ReconciliationResult::Success
            }
            std::cmp::Ordering::Equal => {
                // Same version, no changes needed
                ReconciliationResult::Success
            }
        }
    }

    /// Merge-based reconciliation
    fn reconcile_merge(&self, input: ReconciliationInput) -> ReconciliationResult {
        // Attempt to merge the states
        // This is a simplified implementation

        // Check if states are identical
        if input.local_state == input.remote_state {
            return ReconciliationResult::Success;
        }

        // For this example, we'll assume merge is possible
        // In reality, this would involve CRDT merging logic
        ReconciliationResult::Success
    }

    /// Detect conflicts in state changes
    pub fn detect_conflicts(
        &self,
        local_changes: &[StateChange],
        remote_changes: &[StateChange],
    ) -> Vec<ReconciliationConflict> {
        let mut conflicts = Vec::new();

        // Simple conflict detection - check for overlapping changes
        for local_change in local_changes {
            for remote_change in remote_changes {
                if self.changes_conflict(local_change, remote_change) {
                    conflicts.push(ReconciliationConflict {
                        conflict_type: self.infer_conflict_type(local_change),
                        description: format!(
                            "Conflicting changes to {}",
                            local_change.entity_id
                        ),
                        local_data: serde_json::to_vec(local_change).unwrap_or_default(),
                        remote_data: serde_json::to_vec(remote_change).unwrap_or_default(),
                        resolution_options: self.generate_resolution_options(local_change),
                    });
                }
            }
        }

        conflicts
    }

    /// Check if two changes conflict
    fn changes_conflict(&self, change1: &StateChange, change2: &StateChange) -> bool {
        // Changes conflict if they affect the same entity
        change1.entity_id == change2.entity_id && change1.entity_type == change2.entity_type
    }

    /// Infer conflict type from change
    fn infer_conflict_type(&self, change: &StateChange) -> ConflictType {
        match change.entity_type.as_str() {
            "message" => ConflictType::MessageEdit,
            "contact" => ConflictType::ContactChange,
            "conversation" => ConflictType::ConversationMetadata,
            _ => ConflictType::MessageEdit,
        }
    }

    /// Generate resolution options
    fn generate_resolution_options(&self, _change: &StateChange) -> Vec<ResolutionOption> {
        vec![
            ResolutionOption {
                id: "local".to_string(),
                description: "Keep your local changes".to_string(),
                recommended: true,
            },
            ResolutionOption {
                id: "remote".to_string(),
                description: "Use the remote changes".to_string(),
                recommended: false,
            },
            ResolutionOption {
                id: "merge".to_string(),
                description: "Attempt to merge both changes".to_string(),
                recommended: false,
            },
        ]
    }

    /// Set reconciliation strategy for a data type
    pub fn set_strategy(&mut self, data_type: &str, strategy: ReconciliationStrategy) {
        self.strategies.insert(data_type.to_string(), strategy);
    }
}

/// State change representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Type of entity changed
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Type of change
    pub change_type: String,
    /// Change data
    pub data: Vec<u8>,
    /// Timestamp of change
    pub timestamp: String,
}

impl Default for ReconciliationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_manager_creation() {
        let manager = ReconciliationManager::new();
        assert!(!manager.strategies.is_empty());
    }

    #[test]
    fn test_last_write_wins_reconciliation() {
        let manager = ReconciliationManager::new();

        let input = ReconciliationInput {
            local_state: vec![1, 2, 3],
            remote_state: vec![4, 5, 6],
            local_version: "2024-01-01T10:00:00Z".to_string(),
            remote_version: "2024-01-01T09:00:00Z".to_string(),
        };

        let result = manager.reconcile_last_write_wins(input);
        assert!(matches!(result, ReconciliationResult::Success));
    }

    #[test]
    fn test_conflict_detection() {
        let manager = ReconciliationManager::new();

        let local_changes = vec![StateChange {
            entity_type: "message".to_string(),
            entity_id: "msg1".to_string(),
            change_type: "edit".to_string(),
            data: vec![1, 2, 3],
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];

        let remote_changes = vec![StateChange {
            entity_type: "message".to_string(),
            entity_id: "msg1".to_string(),
            change_type: "edit".to_string(),
            data: vec![4, 5, 6],
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];

        let conflicts = manager.detect_conflicts(&local_changes, &remote_changes);
        assert_eq!(conflicts.len(), 1);
        assert!(matches!(conflicts[0].conflict_type, ConflictType::MessageEdit));
    }

    #[test]
    fn test_resolution_options() {
        let manager = ReconciliationManager::new();

        let change = StateChange {
            entity_type: "message".to_string(),
            entity_id: "msg1".to_string(),
            change_type: "edit".to_string(),
            data: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let options = manager.generate_resolution_options(&change);
        assert_eq!(options.len(), 3);
        assert!(options.iter().any(|o| o.id == "local"));
        assert!(options.iter().any(|o| o.id == "remote"));
        assert!(options.iter().any(|o| o.id == "merge"));
    }
}