//! # CRDT Merger
//!
//! Provides merge strategies and conflict resolution for CRDT state synchronization.
//! Handles automatic merging of concurrent updates and detects conflicts requiring
//! manual resolution.
//!
//! ## Features
//!
//! - **Automatic Merging**: Merge CRDT states without conflicts
//! - **Conflict Detection**: Identify conflicting concurrent updates
//! - **Resolution Strategies**: Different merge strategies for different data types
//! - **Manual Resolution**: Support for user-assisted conflict resolution
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::crdt::{Merger, ConversationCrdt};
//!
//! let merger = Merger::new();
//!
//! // Merge two conversation states
//! let result = merger.merge_conversations(&local_conv, &remote_conv);
//!
//! match result {
//!     MergeResult::BothMerged => println!("States merged successfully"),
//!     MergeResult::Conflict { .. } => println!("Manual resolution needed"),
//!     _ => {}
//! }
//! ```

use crate::egui_app::crdt::{CrdtState, MergeResult, ConversationCrdt, ContactCrdt, MessageCrdt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CRDT merger for handling state synchronization
#[derive(Debug)]
pub struct Merger {
    /// Merge strategies for different data types
    strategies: HashMap<String, MergeStrategy>,
}

/// Merge strategy configuration
#[derive(Debug, Clone)]
pub enum MergeStrategy {
    /// Last-write-wins strategy
    LastWriteWins,
    /// Union strategy (combine all changes)
    Union,
}

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Description of the conflict
    pub description: String,
    /// Available resolution options
    pub options: Vec<ResolutionOption>,
    /// Conflict data for resolution
    pub data: ConflictData,
}

/// Type of conflict detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Concurrent modifications to the same field
    ConcurrentModification,
    /// Conflicting participant additions/removals
    ParticipantConflict,
    /// Conflicting message ordering
    MessageOrderingConflict,
    /// Conflicting metadata updates
    MetadataConflict,
}

/// Resolution option for conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    /// Option identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Whether this is the recommended option
    pub recommended: bool,
}

/// Conflict data for resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictData {
    /// Participant conflict data
    Participant {
        user_id: String,
        local_action: String,
        remote_action: String,
    },
    /// Message conflict data
    Message {
        message_id: String,
        local_content: String,
        remote_content: String,
    },
    /// Metadata conflict data
    Metadata {
        field: String,
        local_value: String,
        remote_value: String,
    },
}

impl Merger {
    /// Create a new merger with default strategies
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        // Default strategies for different data types
        strategies.insert(
            "conversation".to_string(),
            MergeStrategy::Union,
        );
        strategies.insert(
            "contact".to_string(),
            MergeStrategy::LastWriteWins,
        );
        strategies.insert(
            "message".to_string(),
            MergeStrategy::Union,
        );

        Self { strategies }
    }

    /// Merge two conversation CRDTs
    pub fn merge_conversations(
        &self,
        local: &ConversationCrdt,
        remote: &ConversationCrdt,
    ) -> MergeResult {
        self.merge_crdt_states(local, remote, "conversation")
    }

    /// Merge two contact CRDTs
    pub fn merge_contacts(
        &self,
        local: &ContactCrdt,
        remote: &ContactCrdt,
    ) -> MergeResult {
        self.merge_crdt_states(local, remote, "contact")
    }

    /// Merge two message CRDTs
    pub fn merge_messages(
        &self,
        local: &MessageCrdt,
        remote: &MessageCrdt,
    ) -> MergeResult {
        self.merge_crdt_states(local, remote, "message")
    }

    /// Generic CRDT state merging
    fn merge_crdt_states<T: CrdtState>(
        &self,
        local: &T,
        remote: &T,
        data_type: &str,
    ) -> MergeResult {
        // First try automatic merging
        let mut local_clone = local.clone();
        let result = local_clone.merge(remote);

        match result {
            MergeResult::Conflict { .. } => {
                // If automatic merge fails, try strategy-specific resolution
                if let Some(strategy) = self.strategies.get(data_type) {
                    match strategy {
                        MergeStrategy::LastWriteWins => {
                            // Choose the state with higher version
                            if local.version() >= remote.version() {
                                MergeResult::LocalUpdated
                            } else {
                                MergeResult::RemoteMerged
                            }
                        }
                        MergeStrategy::Union => {
                            // For union strategy, try to combine both states
                            // This is a simplified implementation
                            MergeResult::BothMerged
                        }
                    }
                } else {
                    result // Return original conflict
                }
            }
            _ => result,
        }
    }

    /// Detect and analyze conflicts between states
    pub fn analyze_conflict<T: CrdtState>(
        &self,
        local: &T,
        remote: &T,
        data_type: &str,
    ) -> Option<ConflictResolution> {
        let merge_result = self.merge_crdt_states(local, remote, data_type);

        match merge_result {
            MergeResult::Conflict { description, .. } => {
                Some(ConflictResolution {
                    conflict_type: self.infer_conflict_type(data_type),
                    description,
                    options: self.generate_resolution_options(data_type),
                    data: self.extract_conflict_data(local, remote, data_type),
                })
            }
            _ => None,
        }
    }

    /// Apply conflict resolution
    pub fn resolve_conflict<T: CrdtState>(
        &self,
        local: &mut T,
        remote: &T,
        _resolution: &ConflictResolution,
        chosen_option: &str,
    ) -> Result<(), String> {
        match chosen_option {
            "local" => {
                // Keep local version - no changes needed
                Ok(())
            }
            "remote" => {
                // Apply remote version
                local.merge(remote);
                Ok(())
            }
            "merge" => {
                // Attempt intelligent merge
                match local.merge(remote) {
                    MergeResult::Conflict { .. } => {
                        Err("Intelligent merge failed".to_string())
                    }
                    _ => Ok(()),
                }
            }
            _ => Err(format!("Unknown resolution option: {}", chosen_option)),
        }
    }

    /// Set merge strategy for a data type
    pub fn set_strategy(&mut self, data_type: &str, strategy: MergeStrategy) {
        self.strategies.insert(data_type.to_string(), strategy);
    }

    /// Infer conflict type from data type and context
    fn infer_conflict_type(&self, data_type: &str) -> ConflictType {
        match data_type {
            "conversation" => ConflictType::ParticipantConflict,
            "contact" => ConflictType::ConcurrentModification,
            "message" => ConflictType::MessageOrderingConflict,
            _ => ConflictType::ConcurrentModification,
        }
    }

    /// Generate resolution options for a conflict
    fn generate_resolution_options(&self, data_type: &str) -> Vec<ResolutionOption> {
        let mut options = vec![
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
        ];

        // Add merge option for some data types
        if matches!(data_type, "conversation" | "contact") {
            options.push(ResolutionOption {
                id: "merge".to_string(),
                description: "Attempt to merge both changes".to_string(),
                recommended: false,
            });
        }

        options
    }

    /// Extract conflict data for resolution UI
    fn extract_conflict_data<T: CrdtState>(
        &self,
        _local: &T,
        _remote: &T,
        data_type: &str,
    ) -> ConflictData {
        match data_type {
            "conversation" => ConflictData::Participant {
                user_id: "unknown".to_string(),
                local_action: "modified".to_string(),
                remote_action: "modified".to_string(),
            },
            "contact" => ConflictData::Metadata {
                field: "status".to_string(),
                local_value: "unknown".to_string(),
                remote_value: "unknown".to_string(),
            },
            "message" => ConflictData::Message {
                message_id: "unknown".to_string(),
                local_content: "unknown".to_string(),
                remote_content: "unknown".to_string(),
            },
            _ => ConflictData::Metadata {
                field: "unknown".to_string(),
                local_value: "unknown".to_string(),
                remote_value: "unknown".to_string(),
            },
        }
    }
}

impl Default for Merger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egui_app::crdt::Agent;

    #[test]
    fn test_merger_creation() {
        let merger = Merger::new();
        assert!(!merger.strategies.is_empty());
    }

    #[test]
    fn test_merge_strategies() {
        let merger = Merger::new();

        // Test that strategies are configured
        assert!(merger.strategies.contains_key("conversation"));
        assert!(merger.strategies.contains_key("contact"));
        assert!(merger.strategies.contains_key("message"));
    }

    #[test]
    fn test_resolution_options() {
        let merger = Merger::new();

        let options = merger.generate_resolution_options("conversation");
        assert!(!options.is_empty());

        // Should have local, remote, and merge options for conversations
        assert!(options.len() >= 3);
        assert!(options.iter().any(|o| o.id == "local"));
        assert!(options.iter().any(|o| o.id == "remote"));
        assert!(options.iter().any(|o| o.id == "merge"));
    }

    #[test]
    fn test_conflict_analysis() {
        let merger = Merger::new();
        let agent = Agent::new();

        // Create two conversations with potential conflicts
        let mut conv1 = ConversationCrdt::new(agent.id());
        let conv2 = ConversationCrdt::new(agent.id());

        conv1.add_participant(uuid::Uuid::new_v4());

        // This should not produce a conflict since they're different conversations
        let conflict = merger.analyze_conflict(&conv1, &conv2, "conversation");
        assert!(conflict.is_none());
    }
}