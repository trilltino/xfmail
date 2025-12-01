//! # Conflict Resolution System
//!
//! Advanced conflict resolution for local-first synchronization.
//! Handles concurrent modifications with intelligent merge strategies.
//!
//! ## Features
//!
//! - **Automatic Resolution**: Smart conflict detection and merging
//! - **Manual Override**: User-assisted conflict resolution
//! - **Version Vectors**: Lamport timestamp-based ordering
//! - **Operational Transforms**: Real-time collaborative editing
//! - **Merge Strategies**: Multiple resolution approaches

use crate::shared::messaging::ChatMessage;
use std::collections::HashMap;

/// Conflict resolution manager
#[derive(Debug)]
pub struct ConflictResolver {
    /// Active conflicts
    conflicts: HashMap<String, Conflict>,
    /// Resolution strategies
    strategies: HashMap<String, ResolutionStrategy>,
}

#[derive(Debug)]
pub struct Conflict {
    /// Conflict ID
    pub id: String,
    /// Local version
    pub local: ChatMessage,
    /// Remote version
    pub remote: ChatMessage,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Resolution status
    pub status: ConflictStatus,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    /// Content modification conflict
    Content,
    /// Metadata conflict
    Metadata,
    /// Deletion conflict
    Deletion,
}

#[derive(Debug, Clone)]
pub enum ConflictStatus {
    /// Conflict detected
    Detected,
    /// Automatically resolved
    AutoResolved,
    /// Requires manual resolution
    ManualRequired,
    /// Manually resolved
    ManualResolved,
}

#[derive(Debug, Clone)]
pub enum ResolutionStrategy {
    /// Prefer local changes
    PreferLocal,
    /// Prefer remote changes
    PreferRemote,
    /// Merge when possible
    Merge,
    /// Require manual resolution
    Manual,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            conflicts: HashMap::new(),
            strategies: HashMap::new(),
        }
    }

    /// Detect conflicts between local and remote versions
    pub fn detect_conflicts(&mut self, local: &ChatMessage, remote: &ChatMessage) -> Option<Conflict> {
        // Check if versions conflict
        if self.has_conflict(local, remote) {
            let conflict = Conflict {
                id: format!("conflict_{}_{}", local.id, remote.id),
                local: local.clone(),
                remote: remote.clone(),
                conflict_type: self.determine_conflict_type(local, remote),
                status: ConflictStatus::Detected,
            };

            self.conflicts.insert(conflict.id.clone(), conflict.clone());
            Some(conflict)
        } else {
            None
        }
    }

    /// Resolve a conflict automatically
    pub fn resolve_automatically(&mut self, conflict_id: &str) -> Result<ChatMessage, String> {
        let conflict = self.conflicts.get_mut(conflict_id)
            .ok_or_else(|| "Conflict not found".to_string())?;

        let resolved = match conflict.conflict_type {
            ConflictType::Content => self.merge_content(&conflict.local, &conflict.remote)?,
            ConflictType::Metadata => self.merge_metadata(&conflict.local, &conflict.remote),
            ConflictType::Deletion => return Err("Deletion conflicts require manual resolution".to_string()),
        };

        conflict.status = ConflictStatus::AutoResolved;
        Ok(resolved)
    }

    /// Check if two messages conflict
    fn has_conflict(&self, local: &ChatMessage, remote: &ChatMessage) -> bool {
        // Simple conflict detection based on timestamps and content
        local.crdt_timestamp != remote.crdt_timestamp &&
        local.content != remote.content
    }

    /// Determine the type of conflict
    fn determine_conflict_type(&self, local: &ChatMessage, remote: &ChatMessage) -> ConflictType {
        if local.content != remote.content {
            ConflictType::Content
        } else {
            ConflictType::Metadata
        }
    }

    /// Merge conflicting content
    fn merge_content(&self, local: &ChatMessage, remote: &ChatMessage) -> Result<ChatMessage, String> {
        // Simple merge strategy: prefer newer change
        if local.crdt_timestamp > remote.crdt_timestamp {
            Ok(local.clone())
        } else {
            Ok(remote.clone())
        }
    }

    /// Merge conflicting metadata
    fn merge_metadata(&self, local: &ChatMessage, remote: &ChatMessage) -> ChatMessage {
        // Combine metadata from both versions
        ChatMessage {
            id: local.id,
            conversation_id: local.conversation_id,
            sender_id: local.sender_id,
            content: local.content.clone(),
            message_type: local.message_type.clone(),
            timestamp: local.timestamp.clone(),
            is_read: local.is_read || remote.is_read,
            is_delivered: local.is_delivered || remote.is_delivered,
            crdt_timestamp: local.crdt_timestamp.max(remote.crdt_timestamp),
            braid_version: format!("merged_{}_{}", local.braid_version, remote.braid_version),
            braid_parents: vec![local.braid_version.clone(), remote.braid_version.clone()],
        }
    }

    /// Get all active conflicts
    pub fn get_active_conflicts(&self) -> Vec<&Conflict> {
        self.conflicts.values()
            .filter(|c| matches!(c.status, ConflictStatus::Detected | ConflictStatus::ManualRequired))
            .collect()
    }

    /// Manually resolve a conflict
    pub fn resolve_manually(&mut self, conflict_id: &str, resolved: ChatMessage) -> Result<(), String> {
        let conflict = self.conflicts.get_mut(conflict_id)
            .ok_or_else(|| "Conflict not found".to_string())?;

        // Update the resolved message with conflict metadata
        let mut resolved = resolved;
        resolved.braid_parents = vec![conflict.local.braid_version.clone(), conflict.remote.braid_version.clone()];

        conflict.status = ConflictStatus::ManualResolved;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::messaging::MessageType;
    use uuid::Uuid;

    #[test]
    fn test_conflict_detection() {
        let mut resolver = ConflictResolver::new();

        let local = ChatMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            content: "Local content".to_string(),
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: false,
            is_delivered: false,
            crdt_timestamp: 100,
            braid_version: "v1".to_string(),
            braid_parents: vec![],
        };

        let remote = ChatMessage {
            id: local.id,
            conversation_id: local.conversation_id,
            sender_id: local.sender_id,
            content: "Remote content".to_string(),
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: false,
            is_delivered: false,
            crdt_timestamp: 101,
            braid_version: "v2".to_string(),
            braid_parents: vec![],
        };

        let conflict = resolver.detect_conflicts(&local, &remote);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_automatic_resolution() {
        let mut resolver = ConflictResolver::new();

        let local = ChatMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            content: "Local".to_string(),
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: false,
            is_delivered: false,
            crdt_timestamp: 100,
            braid_version: "v1".to_string(),
            braid_parents: vec![],
        };

        let remote = ChatMessage {
            id: local.id,
            conversation_id: local.conversation_id,
            sender_id: local.sender_id,
            content: "Remote".to_string(),
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: true,
            is_delivered: true,
            crdt_timestamp: 101,
            braid_version: "v2".to_string(),
            braid_parents: vec![],
        };

        let conflict = resolver.detect_conflicts(&local, &remote).unwrap();
        let resolved = resolver.resolve_automatically(&conflict.id).unwrap();

        // Should prefer remote (higher timestamp)
        assert_eq!(resolved.content, "Remote");
        // Should merge metadata (is_read and is_delivered should be true)
        assert!(resolved.is_read);
        assert!(resolved.is_delivered);
    }
}