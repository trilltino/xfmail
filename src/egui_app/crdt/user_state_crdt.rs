//! # User State CRDT
//!
//! Conflict-free replicated data type for user presence and status management.
//! Tracks online/offline status, last seen times, and user activity.

use crate::egui_app::crdt::{CrdtState, MergeResult, OperationMeta, OperationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// CRDT state for user presence management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStateCrdt {
    /// Local agent ID
    agent_id: u64,
    /// Current version number
    version: u64,
    /// User presence states
    presence: HashMap<Uuid, UserPresence>,
    /// Operation history
    operations: Vec<OperationMeta>,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User ID
    user_id: Uuid,
    /// Current status
    status: PresenceStatus,
    /// Last seen timestamp
    last_seen: String,
    /// Current activity
    activity: Option<String>,
}

/// User presence status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    /// User is online
    Online,
    /// User is away
    Away,
    /// User is offline
    Offline,
    /// User is busy
    Busy,
}

impl UserStateCrdt {
    /// Create a new user state CRDT
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            version: 0,
            presence: HashMap::new(),
            operations: Vec::new(),
        }
    }

    /// Update user presence
    pub fn update_presence(&mut self, user_id: Uuid, status: PresenceStatus, activity: Option<String>) {
        let presence = self.presence.entry(user_id).or_insert(UserPresence {
            user_id,
            status: PresenceStatus::Offline,
            last_seen: chrono::Utc::now().to_rfc3339(),
            activity: None,
        });

        let status_clone = status.clone();
        let activity_clone = activity.clone();

        presence.status = status_clone;
        presence.last_seen = chrono::Utc::now().to_rfc3339();
        presence.activity = activity.clone();

        // Record operation
        let op_id = self.operations.len() as u64 + 1;
        let data = serde_json::to_vec(&PresenceOperation { user_id, status, activity: activity_clone })
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

    /// Get user presence
    pub fn get_presence(&self, user_id: &Uuid) -> Option<&UserPresence> {
        self.presence.get(user_id)
    }

    /// Get all online users
    pub fn get_online_users(&self) -> Vec<Uuid> {
        self.presence
            .iter()
            .filter(|(_, presence)| presence.status == PresenceStatus::Online)
            .map(|(user_id, _)| *user_id)
            .collect()
    }
}

impl CrdtState for UserStateCrdt {
    fn merge(&mut self, other: &Self) -> MergeResult {
        let mut has_changes = false;

        // Merge presence information (last-write-wins for each user)
        for (user_id, remote_presence) in &other.presence {
            match self.presence.get(user_id) {
                Some(local_presence) => {
                    // Compare timestamps to decide which update is newer
                    if remote_presence.last_seen > local_presence.last_seen {
                        self.presence.insert(*user_id, remote_presence.clone());
                        has_changes = true;
                    }
                }
                None => {
                    self.presence.insert(*user_id, remote_presence.clone());
                    has_changes = true;
                }
            }
        }

        if has_changes {
            self.version = self.version.max(other.version);
            MergeResult::BothMerged
        } else {
            MergeResult::Identical
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
        if let Ok(presence_op) = serde_json::from_slice::<PresenceOperation>(&op.data) {
            self.update_presence(presence_op.user_id, presence_op.status, presence_op.activity);
        }
        Ok(())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn is_empty(&self) -> bool {
        self.presence.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PresenceOperation {
    user_id: Uuid,
    status: PresenceStatus,
    activity: Option<String>,
}