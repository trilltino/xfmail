#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
//! # Contact CRDT
//!
//! Conflict-free replicated data type for contact/friend relationship management.
//! Manages friend requests, acceptances, and contact state across devices.

use crate::egui_app::crdt::{CrdtState, MergeResult, OperationMeta, OperationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// CRDT state for contact management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCrdt {
    /// Local agent ID
    agent_id: u64,
    /// Current version number
    version: u64,
    /// Contact relationships
    relationships: HashMap<Uuid, ContactRelationship>,
    /// Operation history
    operations: Vec<OperationMeta>,
}

/// Contact relationship state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRelationship {
    /// Contact user ID
    user_id: Uuid,
    /// Current relationship status
    status: RelationshipStatus,
    /// When the relationship was established
    established_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipStatus {
    /// No relationship
    None,
    /// Friend request sent
    Requested,
    /// Friend request received
    Pending,
    /// Friends
    Friends,
    /// Relationship blocked
    Blocked,
}

impl ContactCrdt {
    /// Create a new contact CRDT
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            version: 0,
            relationships: HashMap::new(),
            operations: Vec::new(),
        }
    }

    /// Send a friend request
    pub fn send_request(&mut self, user_id: Uuid) {
        self.update_relationship(user_id, RelationshipStatus::Requested);
    }

    /// Accept a friend request
    pub fn accept_request(&mut self, user_id: Uuid) {
        self.update_relationship(user_id, RelationshipStatus::Friends);
    }

    /// Block a contact
    pub fn block_contact(&mut self, user_id: Uuid) {
        self.update_relationship(user_id, RelationshipStatus::Blocked);
    }

    /// Get relationship status
    pub fn get_status(&self, user_id: &Uuid) -> RelationshipStatus {
        self.relationships
            .get(user_id)
            .map(|r| r.status.clone())
            .unwrap_or(RelationshipStatus::None)
    }

    /// Update relationship status
    fn update_relationship(&mut self, user_id: Uuid, status: RelationshipStatus) {
        let relationship = self.relationships.entry(user_id).or_insert(ContactRelationship {
            user_id,
            status: RelationshipStatus::None,
            established_at: None,
        });

        relationship.status = status.clone();

        if matches!(status, RelationshipStatus::Friends) && relationship.established_at.is_none() {
            relationship.established_at = Some(chrono::Utc::now().to_rfc3339());
        }

        // Record operation
        let op_id = self.operations.len() as u64 + 1;
        let data = serde_json::to_vec(&ContactOperation { user_id, status })
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
}

impl CrdtState for ContactCrdt {
    fn merge(&mut self, other: &Self) -> MergeResult {
        // Simple last-write-wins strategy for contacts
        if self.version >= other.version {
            MergeResult::LocalUpdated
        } else {
            // Apply remote operations
            for op in &other.operations {
                if !self.operations.iter().any(|local_op| local_op.id == op.id) {
                    let _ = self.apply_operation(op);
                }
            }
            MergeResult::RemoteMerged
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
        if let Ok(contact_op) = serde_json::from_slice::<ContactOperation>(&op.data) {
            self.update_relationship(contact_op.user_id, contact_op.status);
        }
        Ok(())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactOperation {
    user_id: Uuid,
    status: RelationshipStatus,
}