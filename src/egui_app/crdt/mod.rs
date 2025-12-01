//! # CRDT State Management System
//!
//! Conflict-free Replicated Data Types (CRDTs) for local-first collaborative features.
//! Provides distributed state management with automatic conflict resolution.
//!
//! ## Architecture
//!
//! The CRDT system manages local state that can be synchronized across devices
//! without conflicts. Each data type has its own CRDT implementation:
//!
//! - **Conversation CRDT**: Manages conversation metadata and participant lists
//! - **Contact CRDT**: Manages contact relationships and friend states
//! - **Message CRDT**: Manages message ordering and delivery status
//! - **User State CRDT**: Manages user presence and status information
//!
//! ## Key Components
//!
//! - `agent.rs`: Local agent management and unique ID generation
//! - `merger.rs`: CRDT merge strategies and conflict resolution
//! - `serializer.rs`: Efficient CRDT state serialization
//! - Individual CRDT modules for each data type
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::crdt::{ConversationCrdt, Agent};
//!
//! // Create local agent
//! let agent = Agent::new();
//!
//! // Create conversation CRDT
//! let mut conversation_crdt = ConversationCrdt::new(agent.id());
//!
//! // Add participant
//! conversation_crdt.add_participant(user_id);
//!
//! // Get current state
//! let state = conversation_crdt.state();
//! ```
//!
//! ## Synchronization
//!
//! CRDTs synchronize by merging states from different devices:
//!
//! ```rust,no_run
//! // Merge remote state
//! conversation_crdt.merge(remote_state);
//!
//! // Get operations to send to other devices
//! let operations = conversation_crdt.operations_since(last_sync_version);
//! ```

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod agent;
pub mod conversation_crdt;
pub mod contact_crdt;
pub mod message_crdt;
pub mod user_state_crdt;
pub mod merger;
pub mod serializer;

// Re-export main types
pub use agent::Agent;
pub use conversation_crdt::ConversationCrdt;
pub use contact_crdt::ContactCrdt;
pub use message_crdt::MessageCrdt;
pub use user_state_crdt::UserStateCrdt;
// Re-export merger type. `MergeResult` is defined in this module below.
pub use merger::Merger;
pub use serializer::{CrdtSerializer, SerializedState};

use serde::{Deserialize, Serialize};

/// Common CRDT operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Add operation (insert new item)
    Add,
    /// Remove operation (delete item)
    Remove,
    /// Update operation (modify existing item)
    Update,
}

/// CRDT operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMeta {
    /// Unique operation ID
    pub id: u64,
    /// Agent that created the operation
    pub agent_id: u64,
    /// Lamport timestamp for ordering
    pub timestamp: u64,
    /// Type of operation
    pub op_type: OperationType,
    /// Operation data (serialized)
    pub data: Vec<u8>,
}

/// CRDT state interface
pub trait CrdtState: Clone + Send + Sync {
    /// Merge this state with another state
    fn merge(&mut self, other: &Self) -> MergeResult;

    /// Generate operations since given version
    fn operations_since(&self, version: u64) -> Vec<OperationMeta>;

    /// Apply an operation to this state
    fn apply_operation(&mut self, op: &OperationMeta) -> Result<(), String>;

    /// Get current version number
    fn version(&self) -> u64;

    /// Check if state is empty
    fn is_empty(&self) -> bool;
}

/// Result of a merge operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// States were identical
    Identical,
    /// Local state was updated
    LocalUpdated,
    /// Remote state was incorporated
    RemoteMerged,
    /// Both states had changes that were merged
    BothMerged,
    /// Merge conflict that requires manual resolution
    Conflict {
        /// Description of the conflict
        description: String,
        /// Local version of conflicting data
        local_data: Vec<u8>,
        /// Remote version of conflicting data
        remote_data: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_types() {
        assert_eq!(OperationType::Add, OperationType::Add);
        assert_ne!(OperationType::Add, OperationType::Remove);
    }

    #[test]
    fn test_merge_results() {
        assert_eq!(MergeResult::Identical, MergeResult::Identical);
        assert_ne!(MergeResult::LocalUpdated, MergeResult::RemoteMerged);
    }
}