//! # Message CRDT - Braid Protocol Implementation
//!
//! Conflict-free replicated data type for message ordering and delivery status.
//! Implements the Braid protocol with version vectors for causal ordering and conflict resolution.

use crate::egui_app::crdt::{CrdtState, MergeResult, OperationMeta};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

/// Version vector for tracking causality across multiple agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionVector {
    /// Map of agent_id -> logical clock value
    versions: HashMap<u64, u64>,
}

impl VersionVector {
    /// Create a new empty version vector
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// Increment the version for a specific agent
    pub fn increment(&mut self, agent_id: u64) -> VersionVector {
        let current = self.versions.get(&agent_id).unwrap_or(&0);
        self.versions.insert(agent_id, current + 1);
        self.clone()
    }

    /// Merge with another version vector (take maximum values)
    pub fn merge(&mut self, other: &VersionVector) {
        for (agent_id, version) in &other.versions {
            let current = self.versions.get(agent_id).unwrap_or(&0);
            if *version > *current {
                self.versions.insert(*agent_id, *version);
            }
        }
    }

    /// Check if this vector dominates another (all values >= other)
    pub fn dominates(&self, other: &VersionVector) -> bool {
        // Check if self has all versions >= other's versions
        for (agent_id, other_version) in &other.versions {
            let self_version = self.versions.get(agent_id).unwrap_or(&0);
            if self_version < other_version {
                return false;
            }
        }
        true
    }

    /// Check if two version vectors are concurrent (neither dominates the other)
    pub fn concurrent(&self, other: &VersionVector) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }

    /// Get the maximum version across all agents
    pub fn max_version(&self) -> u64 {
        self.versions.values().max().unwrap_or(&0).clone()
    }
}

/// CRDT state for message management with Braid protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCrdt {
    /// Conversation this CRDT manages
    conversation_id: Uuid,
    /// Local agent ID
    agent_id: u64,
    /// Current version vector
    version_vector: VersionVector,
    /// Lamport timestamp for total ordering
    lamport_clock: u64,
    /// Messages ordered by version vector (for causal ordering)
    messages: BTreeMap<String, MessageEntry>, // Key is version string
    /// Delivery status tracking
    delivery_status: HashMap<Uuid, MessageStatus>,
    /// Operation history for synchronization
    operations: Vec<OperationMeta>,
    /// Pending messages for offline sync
    pending_messages: Vec<MessageEntry>,
}

/// Message entry with full CRDT metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    /// Message ID
    pub id: Uuid,
    /// Conversation ID
    pub conversation_id: Uuid,
    /// Message content
    pub content: String,
    /// Message type
    pub message_type: String,
    /// Sender ID
    pub sender_id: Uuid,
    /// Version vector when message was created
    pub version_vector: VersionVector,
    /// Lamport timestamp
    pub lamport_timestamp: u64,
    /// Braid version string (for HTTP headers)
    pub braid_version: String,
    /// Parent versions (Braid parents)
    pub braid_parents: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Delivery status
    pub is_delivered: bool,
    pub is_read: bool,
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message sent but not delivered
    Sent,
    /// Message delivered to recipient
    Delivered,
    /// Message read by recipient
    Read,
}

impl MessageCrdt {
    /// Create a new message CRDT for a conversation
    pub fn new(conversation_id: Uuid, agent_id: u64) -> Self {
        Self {
            conversation_id,
            agent_id,
            version_vector: VersionVector::new(),
            lamport_clock: 0,
            messages: BTreeMap::new(),
            delivery_status: HashMap::new(),
            operations: Vec::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Create a new message with proper CRDT versioning
    pub fn create_message(
        &mut self,
        content: String,
        message_type: String,
        sender_id: Uuid,
    ) -> MessageEntry {
        // Increment version vector for this agent
        let new_version_vector = self.version_vector.increment(self.agent_id);

        // Increment Lamport clock
        self.lamport_clock += 1;

        // Generate braid version string
        let braid_version = format!("msg-{}-{}", self.agent_id, self.lamport_clock);

        // Get parent versions (all current message versions)
        let braid_parents = self.messages.keys().cloned().collect();

        let message = MessageEntry {
            id: Uuid::new_v4(),
            conversation_id: self.conversation_id,
            content,
            message_type,
            sender_id,
            version_vector: new_version_vector.clone(),
            lamport_timestamp: self.lamport_clock,
            braid_version: braid_version.clone(),
            braid_parents,
            created_at: chrono::Utc::now(),
            is_delivered: false,
            is_read: false,
        };

        // Update our version vector
        self.version_vector = new_version_vector;

        // Add to pending messages (will be synced when online)
        self.pending_messages.push(message.clone());

        message
    }

    /// Add a received message from another client
    pub fn add_received_message(&mut self, message: MessageEntry) {
        // Update our version vector with the received message's vector
        self.version_vector.merge(&message.version_vector);

        // Update Lamport clock
        self.lamport_clock = self.lamport_clock.max(message.lamport_timestamp) + 1;

        // Store message using braid version as key
        self.messages.insert(message.braid_version.clone(), message);
    }

    /// Get messages in causal order (sorted by version vector)
    pub fn get_messages(&self) -> Vec<&MessageEntry> {
        // For now, return in insertion order - full causal sorting is complex
        // TODO: Implement proper causal ordering
        self.messages.values().collect()
    }

    /// Get messages sorted by Lamport timestamp
    pub fn get_messages_chronological(&self) -> Vec<&MessageEntry> {
        let mut messages: Vec<&MessageEntry> = self.messages.values().collect();
        messages.sort_by_key(|m| m.lamport_timestamp);
        messages
    }

    /// Mark message as delivered
    pub fn mark_delivered(&mut self, message_id: Uuid) {
        if let Some(message) = self.messages.values_mut().find(|m| m.id == message_id) {
            message.is_delivered = true;
        }
        self.delivery_status.insert(message_id, MessageStatus::Delivered);
    }

    /// Mark message as read
    pub fn mark_read(&mut self, message_id: Uuid) {
        if let Some(message) = self.messages.values_mut().find(|m| m.id == message_id) {
            message.is_read = true;
        }
        self.delivery_status.insert(message_id, MessageStatus::Read);
    }

    /// Get pending messages for sync
    pub fn pending_messages(&self) -> &[MessageEntry] {
        &self.pending_messages
    }

    /// Clear pending messages after successful sync
    pub fn clear_pending_messages(&mut self) {
        self.pending_messages.clear();
    }

    /// Get current version vector
    pub fn version_vector(&self) -> &VersionVector {
        &self.version_vector
    }

    /// Get current Lamport clock
    pub fn lamport_clock(&self) -> u64 {
        self.lamport_clock
    }

    /// Check if we have a specific message
    pub fn has_message(&self, braid_version: &str) -> bool {
        self.messages.contains_key(braid_version)
    }

    /// Get missing message versions compared to another CRDT
    pub fn missing_versions(&self, other: &MessageCrdt) -> Vec<String> {
        other.messages.keys()
            .filter(|version| !self.messages.contains_key(*version))
            .cloned()
            .collect()
    }
}

impl CrdtState for MessageCrdt {
    fn merge(&mut self, other: &Self) -> MergeResult {
        if self.conversation_id != other.conversation_id {
            return MergeResult::Conflict {
                description: "Cannot merge CRDTs from different conversations".to_string(),
                local_data: vec![],
                remote_data: vec![],
            };
        }

        let mut has_local_changes = false;
        let mut has_remote_changes = false;
        let mut conflicts = Vec::new();

        // Merge version vectors
        let _original_vector = self.version_vector.clone();
        self.version_vector.merge(&other.version_vector);

        // Update Lamport clock
        self.lamport_clock = self.lamport_clock.max(other.lamport_clock);

        // Merge messages - add any missing messages from remote
        for (version_key, remote_message) in &other.messages {
            if !self.messages.contains_key(version_key) {
                // We don't have this message, add it
                self.messages.insert(version_key.clone(), remote_message.clone());
                has_remote_changes = true;

                // Update delivery status
                self.delivery_status.insert(
                    remote_message.id,
                    if remote_message.is_read {
                        MessageStatus::Read
                    } else if remote_message.is_delivered {
                        MessageStatus::Delivered
                    } else {
                        MessageStatus::Sent
                    }
                );
            } else {
                // We have this message, merge delivery status
                if let Some(local_message) = self.messages.get_mut(version_key) {
                    let local_status = self.delivery_status.get(&local_message.id);
                    let remote_status = if remote_message.is_read {
                        MessageStatus::Read
                    } else if remote_message.is_delivered {
                        MessageStatus::Delivered
                    } else {
                        MessageStatus::Sent
                    };

                    // Merge delivery status (take the more advanced status)
                    match (local_status, &remote_status) {
                        (Some(MessageStatus::Sent), MessageStatus::Delivered) |
                        (Some(MessageStatus::Sent), MessageStatus::Read) |
                        (Some(MessageStatus::Delivered), MessageStatus::Read) => {
                            self.delivery_status.insert(local_message.id, remote_status);
                            local_message.is_delivered = remote_message.is_delivered;
                            local_message.is_read = remote_message.is_read;
                            has_remote_changes = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check if we have messages that remote doesn't have
        for version_key in self.messages.keys() {
            if !other.messages.contains_key(version_key) {
                has_local_changes = true;
            }
        }

        // Check for concurrent messages (same content, different versions)
        // This is a simplified conflict detection - in practice, you'd want more sophisticated logic
        let local_versions: HashSet<String> = self.messages.keys().cloned().collect();
        let remote_versions: HashSet<String> = other.messages.keys().cloned().collect();

        // Find messages that exist in both but might have conflicts
        for version_key in local_versions.intersection(&remote_versions) {
            let local_msg = self.messages.get(version_key).unwrap();
            let remote_msg = other.messages.get(version_key).unwrap();

            // Check for content conflicts (same ID but different content - shouldn't happen in practice)
            if local_msg.content != remote_msg.content {
                conflicts.push(format!("Message {} has content conflict", local_msg.id));
            }
        }

        // Return appropriate merge result
        if !conflicts.is_empty() {
            MergeResult::Conflict {
                description: format!("Content conflicts detected: {:?}", conflicts),
                local_data: serde_json::to_vec(&self.messages).unwrap_or_default(),
                remote_data: serde_json::to_vec(&other.messages).unwrap_or_default(),
            }
        } else {
            match (has_local_changes, has_remote_changes) {
                (false, false) => MergeResult::Identical,
                (true, false) => MergeResult::LocalUpdated,
                (false, true) => MergeResult::RemoteMerged,
                (true, true) => MergeResult::BothMerged,
            }
        }
    }

    fn operations_since(&self, version: u64) -> Vec<OperationMeta> {
        self.operations
            .iter()
            .filter(|op| op.timestamp > version)
            .cloned()
            .collect()
    }

    fn apply_operation(&mut self, _op: &OperationMeta) -> Result<(), String> {
        // For now, operations are handled through direct message merging
        // In a full implementation, this would apply individual operations
        Ok(())
    }

    fn version(&self) -> u64 {
        self.lamport_clock
    }

    fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Message operations for CRDT synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageOperation {
    /// Add a new message
    Add {
        message_id: Uuid,
        braid_version: String,
        content: String,
        message_type: String,
        sender_id: Uuid,
        version_vector: VersionVector,
        lamport_timestamp: u64,
        braid_parents: Vec<String>,
    },
    /// Update delivery status
    UpdateStatus {
        message_id: Uuid,
        is_delivered: bool,
        is_read: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_vector_operations() {
        let mut vv1 = VersionVector::new();
        let mut vv2 = VersionVector::new();

        // Increment different agents
        vv1.increment(1);
        vv2.increment(2);

        // Merge
        vv1.merge(&vv2);

        assert_eq!(vv1.versions.get(&1), Some(&1));
        assert_eq!(vv1.versions.get(&2), Some(&1));
    }

    #[test]
    fn test_message_crdt_creation() {
        let conversation_id = Uuid::new_v4();
        let crdt = MessageCrdt::new(conversation_id, 1);

        assert_eq!(crdt.conversation_id, conversation_id);
        assert_eq!(crdt.agent_id, 1);
        assert!(crdt.messages.is_empty());
    }

    #[test]
    fn test_message_creation() {
        let conversation_id = Uuid::new_v4();
        let mut crdt = MessageCrdt::new(conversation_id, 1);

        let message = crdt.create_message(
            "Hello world".to_string(),
            "text".to_string(),
            Uuid::new_v4(),
        );

        assert_eq!(message.content, "Hello world");
        assert_eq!(message.conversation_id, conversation_id);
        assert!(!message.braid_version.is_empty());
        assert_eq!(crdt.pending_messages().len(), 1);
    }

    #[test]
    fn test_message_merging() {
        let conversation_id = Uuid::new_v4();
        let mut crdt1 = MessageCrdt::new(conversation_id, 1);
        let mut crdt2 = MessageCrdt::new(conversation_id, 2);

        // Create message in crdt1
        let message = crdt1.create_message(
            "Test message".to_string(),
            "text".to_string(),
            Uuid::new_v4(),
        );

        // Add message to crdt2 (simulating receiving from network)
        crdt2.add_received_message(message);

        // Merge crdt2 into crdt1
        let result = crdt1.merge(&crdt2);

        assert!(matches!(result, MergeResult::BothMerged));
        assert_eq!(crdt1.messages.len(), 1);
    }
}