//! Message CRDT Types
//!
//! This module defines CRDT types for message ordering and conflict resolution.
//! Uses Lamport timestamps for causal ordering of messages.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lamport timestamp for message ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportTimestamp {
    /// The logical clock value
    pub counter: u64,
    /// The node/user ID for tie-breaking
    pub node_id: Uuid,
}

impl LamportTimestamp {
    /// Create a new timestamp
    pub fn new(counter: u64, node_id: Uuid) -> Self {
        Self { counter, node_id }
    }

    /// Create the initial timestamp for a node
    pub fn initial(node_id: Uuid) -> Self {
        Self { counter: 0, node_id }
    }

    /// Increment the timestamp
    pub fn increment(&self) -> Self {
        Self {
            counter: self.counter + 1,
            node_id: self.node_id,
        }
    }

    /// Update timestamp based on received message
    pub fn update(&self, received: &LamportTimestamp) -> Self {
        Self {
            counter: std::cmp::max(self.counter, received.counter) + 1,
            node_id: self.node_id,
        }
    }

    /// Convert to string for Braid version header
    pub fn to_version_string(&self) -> String {
        format!("{}-{}", self.counter, self.node_id)
    }

    /// Parse from Braid version string
    pub fn from_version_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, '-').collect();
        if parts.len() != 2 {
            return None;
        }
        let counter = parts[0].parse().ok()?;
        let node_id = Uuid::parse_str(parts[1]).ok()?;
        Some(Self { counter, node_id })
    }
}

/// Message operation for CRDT sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOperation {
    /// The operation type
    pub op_type: MessageOpType,
    /// Lamport timestamp for ordering
    pub timestamp: LamportTimestamp,
    /// Message ID
    pub message_id: Uuid,
    /// Conversation ID
    pub conversation_id: Uuid,
}

/// Types of message operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageOpType {
    /// Send a new message
    Send {
        sender_id: Uuid,
        content: String,
        message_type: String,
    },
    /// Edit an existing message
    Edit {
        new_content: String,
    },
    /// Delete a message
    Delete,
    /// Mark message as read
    Read {
        reader_id: Uuid,
    },
}

/// Message state after applying operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageState {
    /// All messages in the conversation, ordered by timestamp
    pub messages: Vec<MessageEntry>,
    /// Current Lamport timestamp
    pub current_timestamp: LamportTimestamp,
}

/// A single message entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub timestamp: LamportTimestamp,
    pub is_deleted: bool,
    pub read_by: Vec<Uuid>,
}

impl MessageState {
    /// Create a new empty message state
    pub fn new(node_id: Uuid) -> Self {
        Self {
            messages: Vec::new(),
            current_timestamp: LamportTimestamp::initial(node_id),
        }
    }

    /// Apply an operation to the state
    pub fn apply(&mut self, op: MessageOperation) {
        // Update our timestamp
        self.current_timestamp = self.current_timestamp.update(&op.timestamp);

        match op.op_type {
            MessageOpType::Send {
                sender_id,
                content,
                message_type,
            } => {
                let entry = MessageEntry {
                    id: op.message_id,
                    sender_id,
                    content,
                    message_type,
                    timestamp: op.timestamp,
                    is_deleted: false,
                    read_by: Vec::new(),
                };
                // Insert in sorted order by timestamp
                let pos = self
                    .messages
                    .binary_search_by(|m| m.timestamp.cmp(&op.timestamp))
                    .unwrap_or_else(|p| p);
                self.messages.insert(pos, entry);
            }
            MessageOpType::Edit { new_content } => {
                if let Some(msg) = self.messages.iter_mut().find(|m| m.id == op.message_id) {
                    msg.content = new_content;
                }
            }
            MessageOpType::Delete => {
                if let Some(msg) = self.messages.iter_mut().find(|m| m.id == op.message_id) {
                    msg.is_deleted = true;
                }
            }
            MessageOpType::Read { reader_id } => {
                if let Some(msg) = self.messages.iter_mut().find(|m| m.id == op.message_id) {
                    if !msg.read_by.contains(&reader_id) {
                        msg.read_by.push(reader_id);
                    }
                }
            }
        }
    }
}

