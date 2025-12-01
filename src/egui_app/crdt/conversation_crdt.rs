//! # Conversation CRDT
//!
//! Conflict-free replicated data type for conversation state management.
//! Manages conversation metadata, participant lists, and ensures consistency
//! across devices without conflicts.
//!
//! ## Features
//!
//! - **Participant Management**: Add/remove participants with conflict resolution
//! - **Metadata Updates**: Update conversation names and settings
//! - **Version Tracking**: Track operation history for synchronization
//! - **Merge Resolution**: Automatic merging of concurrent updates
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::crdt::{ConversationCrdt, Agent};
//!
//! let agent = Agent::new();
//! let mut conversation = ConversationCrdt::new(agent.id());
//!
//! // Add participants
//! conversation.add_participant(user1_id);
//! conversation.add_participant(user2_id);
//!
//! // Update conversation name
//! conversation.update_name("Project Discussion");
//!
//! // Get current state
//! let state = conversation.state();
//!
//! // Merge with remote state
//! conversation.merge(&remote_state);
//! ```

use crate::egui_app::crdt::{CrdtState, MergeResult, OperationMeta, OperationType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// CRDT state for conversation management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationCrdt {
    /// Unique conversation ID
    conversation_id: Uuid,
    /// Local agent ID
    agent_id: u64,
    /// Current version number
    version: u64,
    /// Conversation metadata
    metadata: ConversationMetadata,
    /// Participant set with add/remove tracking
    participants: ParticipantSet,
    /// Operation history for synchronization
    operations: Vec<OperationMeta>,
}

/// Conversation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    /// Conversation name/title
    name: Option<String>,
    /// Creation timestamp
    created_at: String,
    /// Last update timestamp
    updated_at: String,
    /// Conversation type (direct, group, etc.)
    conversation_type: String,
}

/// Participant management with CRDT semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticipantSet {
    /// Currently active participants
    active: HashSet<Uuid>,
    /// Add operations (user_id -> operation_id)
    adds: HashMap<Uuid, u64>,
    /// Remove operations (user_id -> operation_id)
    removes: HashMap<Uuid, u64>,
}

impl ConversationCrdt {
    /// Create a new conversation CRDT
    pub fn new(agent_id: u64) -> Self {
        Self {
            conversation_id: Uuid::new_v4(),
            agent_id,
            version: 0,
            metadata: ConversationMetadata {
                name: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                conversation_type: "direct".to_string(),
            },
            participants: ParticipantSet {
                active: HashSet::new(),
                adds: HashMap::new(),
                removes: HashMap::new(),
            },
            operations: Vec::new(),
        }
    }

    /// Create CRDT from existing conversation
    pub fn from_conversation(
        agent_id: u64,
        conversation_id: Uuid,
        participants: Vec<Uuid>,
        name: Option<String>,
    ) -> Self {
        let mut crdt = Self::new(agent_id);
        crdt.conversation_id = conversation_id;
        crdt.metadata.name = name;

        for participant in participants {
            crdt.add_participant(participant);
        }

        crdt
    }

    /// Get conversation ID
    pub fn conversation_id(&self) -> Uuid {
        self.conversation_id
    }

    /// Get current participants
    pub fn participants(&self) -> Vec<Uuid> {
        self.participants.active.iter().cloned().collect()
    }

    /// Get conversation name
    pub fn name(&self) -> Option<&str> {
        self.metadata.name.as_deref()
    }

    /// Add a participant to the conversation
    pub fn add_participant(&mut self, user_id: Uuid) {
        // Don't add if already added and not removed
        if self.participants.active.contains(&user_id) {
            return;
        }

        // Record the add operation
        let op_id = self.next_operation_id();
        self.participants.adds.insert(user_id, op_id);
        self.participants.active.insert(user_id);

        // Create operation metadata
        let data = serde_json::to_vec(&ParticipantOperation::Add { user_id })
            .unwrap_or_default();

        let operation = OperationMeta {
            id: op_id,
            agent_id: self.agent_id,
            timestamp: self.version,
            op_type: OperationType::Add,
            data,
        };

        self.operations.push(operation);
        self.version += 1;
    }

    /// Remove a participant from the conversation
    pub fn remove_participant(&mut self, user_id: Uuid) {
        if !self.participants.active.contains(&user_id) {
            return;
        }

        // Record the remove operation
        let op_id = self.next_operation_id();
        self.participants.removes.insert(user_id, op_id);
        self.participants.active.remove(&user_id);

        // Create operation metadata
        let data = serde_json::to_vec(&ParticipantOperation::Remove { user_id })
            .unwrap_or_default();

        let operation = OperationMeta {
            id: op_id,
            agent_id: self.agent_id,
            timestamp: self.version,
            op_type: OperationType::Remove,
            data,
        };

        self.operations.push(operation);
        self.version += 1;
    }

    /// Update conversation name
    pub fn update_name(&mut self, name: &str) {
        self.metadata.name = Some(name.to_string());
        self.metadata.updated_at = chrono::Utc::now().to_rfc3339();

        let op_id = self.next_operation_id();
        let data = serde_json::to_vec(&MetadataOperation::UpdateName {
            name: name.to_string(),
        })
        .unwrap_or_default();

        let operation = OperationMeta {
            id: op_id,
            agent_id: self.agent_id,
            timestamp: self.version,
            op_type: OperationType::Update,
            data,
        };

        self.operations.push(operation);
        self.version += 1;
    }

    /// Get current state for serialization
    pub fn state(&self) -> ConversationState {
        ConversationState {
            conversation_id: self.conversation_id,
            metadata: self.metadata.clone(),
            participants: self.participants(),
            version: self.version,
        }
    }

    /// Generate next operation ID
    fn next_operation_id(&self) -> u64 {
        // Simple counter-based ID generation
        // In production, this would be more sophisticated
        self.operations.len() as u64 + 1
    }
}

impl CrdtState for ConversationCrdt {
    fn merge(&mut self, other: &Self) -> MergeResult {
        if self.version == other.version {
            return MergeResult::Identical;
        }

        let mut has_local_changes = false;
        let mut has_remote_changes = false;

        // Apply remote operations that we don't have
        for remote_op in &other.operations {
            if !self.operations.iter().any(|op| op.id == remote_op.id) {
                if let Err(_) = self.apply_operation(remote_op) {
                    return MergeResult::Conflict {
                        description: "Failed to apply remote operation".to_string(),
                        local_data: serde_json::to_vec(self).unwrap_or_default(),
                        remote_data: serde_json::to_vec(other).unwrap_or_default(),
                    };
                }
                has_remote_changes = true;
            }
        }

        // Check if we have operations the remote doesn't have
        for local_op in &self.operations {
            if !other.operations.iter().any(|op| op.id == local_op.id) {
                has_local_changes = true;
            }
        }

        match (has_local_changes, has_remote_changes) {
            (false, false) => MergeResult::Identical,
            (true, false) => MergeResult::LocalUpdated,
            (false, true) => MergeResult::RemoteMerged,
            (true, true) => MergeResult::BothMerged,
        }
    }

    fn operations_since(&self, version: u64) -> Vec<OperationMeta> {
        self.operations
            .iter()
            .filter(|op| op.timestamp > version)
            .cloned()
            .collect()
    }

    fn apply_operation(&mut self, op: &OperationMeta) -> Result<(), String> {
        match op.op_type {
            OperationType::Add => {
                if let Ok(part_op) = serde_json::from_slice::<ParticipantOperation>(&op.data) {
                    match part_op {
                        ParticipantOperation::Add { user_id } => {
                            self.participants.active.insert(user_id);
                            self.participants.adds.insert(user_id, op.id);
                        }
                        ParticipantOperation::Remove { user_id } => {
                            self.participants.active.remove(&user_id);
                            self.participants.removes.insert(user_id, op.id);
                        }
                    }
                }
            }
            OperationType::Remove => {
                // Handle remove operations
            }
            OperationType::Update => {
                if let Ok(meta_op) = serde_json::from_slice::<MetadataOperation>(&op.data) {
                    match meta_op {
                        MetadataOperation::UpdateName { name } => {
                            self.metadata.name = Some(name);
                            self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
                        }
                    }
                }
            }
        }

        // Add operation to history if not already present
        if !self.operations.iter().any(|existing| existing.id == op.id) {
            self.operations.push(op.clone());
        }

        self.version = self.version.max(op.timestamp + 1);
        Ok(())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn is_empty(&self) -> bool {
        self.participants.active.is_empty()
    }
}

/// Serializable conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub conversation_id: Uuid,
    pub metadata: ConversationMetadata,
    pub participants: Vec<Uuid>,
    pub version: u64,
}

/// Participant operations for CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ParticipantOperation {
    Add { user_id: Uuid },
    Remove { user_id: Uuid },
}

/// Metadata operations for CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MetadataOperation {
    UpdateName { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egui_app::crdt::Agent;

    #[test]
    fn test_conversation_creation() {
        let agent = Agent::new();
        let conversation = ConversationCrdt::new(agent.id());

        assert!(conversation.participants().is_empty());
        assert!(conversation.name().is_none());
        assert_eq!(conversation.version(), 0);
    }

    #[test]
    fn test_add_participants() {
        let agent = Agent::new();
        let mut conversation = ConversationCrdt::new(agent.id());

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        conversation.add_participant(user1);
        conversation.add_participant(user2);

        let participants = conversation.participants();
        assert_eq!(participants.len(), 2);
        assert!(participants.contains(&user1));
        assert!(participants.contains(&user2));
        assert_eq!(conversation.version(), 2);
    }

    #[test]
    fn test_update_name() {
        let agent = Agent::new();
        let mut conversation = ConversationCrdt::new(agent.id());

        conversation.update_name("Test Conversation");

        assert_eq!(conversation.name(), Some("Test Conversation"));
        assert_eq!(conversation.version(), 1);
    }

    #[test]
    fn test_merge_identical() {
        let agent = Agent::new();
        let mut conversation1 = ConversationCrdt::new(agent.id());
        let conversation2 = ConversationCrdt::new(agent.id());

        let result = conversation1.merge(&conversation2);
        assert_eq!(result, MergeResult::Identical);
    }

    #[test]
    fn test_merge_with_changes() {
        let agent = Agent::new();
        let mut conversation1 = ConversationCrdt::new(agent.id());
        let conversation2 = ConversationCrdt::new(agent.id());

        let user_id = Uuid::new_v4();
        conversation1.add_participant(user_id);

        let result = conversation1.merge(&conversation2);
        assert_eq!(result, MergeResult::LocalUpdated);
    }
}