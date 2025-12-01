//! Chat Message Data Structure
//!
//! Represents a message in a conversation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Version vector for CRDT causal ordering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VersionVector {
    /// Map of agent_id -> logical clock value
    pub versions: std::collections::HashMap<u64, u64>,
}

/// Type of message content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    /// Plain text message
    Text,
    /// Image message
    Image {
        url: String,
        thumbnail: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    /// File attachment
    File {
        filename: String,
        size: u64,
        mime_type: String,
        url: String,
    },
    /// System message (e.g., "User joined")
    System,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Text
    }
}

impl MessageType {
    /// Convert to string for database storage
    pub fn to_string(&self) -> String {
        match self {
            MessageType::Text => "text".to_string(),
            MessageType::Image { .. } => "image".to_string(),
            MessageType::File { .. } => "file".to_string(),
            MessageType::System => "system".to_string(),
        }
    }

    /// Parse from string (database)
    pub fn from_str(s: &str) -> Self {
        match s {
            "image" => MessageType::Image {
                url: String::new(),
                thumbnail: None,
                width: None,
                height: None,
            },
            "file" => MessageType::File {
                filename: String::new(),
                size: 0,
                mime_type: String::new(),
                url: String::new(),
            },
            "system" => MessageType::System,
            _ => MessageType::Text,
        }
    }
}

/// Represents a chat message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: Uuid,
    /// Conversation this message belongs to
    pub conversation_id: Uuid,
    /// User who sent the message
    pub sender_id: Uuid,
    /// Message content (text for Text type)
    pub content: String,
    /// Type of message
    #[serde(default)]
    pub message_type: MessageType,
    /// When the message was sent (RFC3339 string, unified across targets)
    pub timestamp: String,
    /// Whether the message has been read by recipient
    pub is_read: bool,
    /// Whether the message has been delivered
    pub is_delivered: bool,
    /// CRDT timestamp for ordering (Lamport-style)
    pub crdt_timestamp: u64,
    /// Braid version ID
    pub braid_version: String,
    /// Braid parent versions
    pub braid_parents: Vec<String>,
    /// Version vector for causal ordering
    #[serde(default)]
    pub version_vector: VersionVector,
}

impl ChatMessage {
    /// Create a new text message
    pub fn new_text(
        conversation_id: Uuid,
        sender_id: Uuid,
        content: String,
        crdt_timestamp: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            content,
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: false,
            is_delivered: false,
            crdt_timestamp,
            braid_version: Uuid::new_v4().to_string(),
            braid_parents: Vec::new(),
            version_vector: VersionVector::default(),
        }
    }

    /// Get a preview of the message (first N characters)
    pub fn preview(&self, max_len: usize) -> String {
        if self.content.len() <= max_len {
            self.content.clone()
        } else {
            let mut preview: String = self.content.chars().take(max_len - 3).collect();
            preview.push_str("...");
            preview
        }
    }
}

/// Request to send a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    #[serde(default)]
    pub message_type: MessageType,
}

/// Response after sending a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub message: Option<ChatMessage>,
    pub error: Option<String>,
}

/// Request to list messages in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesRequest {
    pub conversation_id: Uuid,
    pub limit: Option<u32>,
    pub before_timestamp: Option<String>,
}

/// Response for listing messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<ChatMessage>,
    pub has_more: bool,
}

