//! Conversation Data Structure
//!
//! Represents a conversation between two or more users.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::message::ChatMessage;

/// Represents a conversation between users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    /// Unique conversation ID
    pub id: Uuid,
    /// Participant user IDs
    pub participants: Vec<Uuid>,
    /// Username of the other participant (for display in chat list)
    pub other_username: Option<String>,
    /// Last message in the conversation (for preview)
    pub last_message: Option<ChatMessage>,
    /// Preview text of last message
    pub last_message_preview: String,
    /// Timestamp of last message (RFC3339 string)
    pub last_message_time: Option<String>,
    /// Number of unread messages
    pub unread_count: u32,
    /// When the conversation was created (RFC3339 string)
    pub created_at: String,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(participants: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            participants,
            other_username: None,
            last_message: None,
            last_message_preview: String::new(),
            last_message_time: None,
            unread_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a new conversation between two users
    pub fn new_direct(user1: Uuid, user2: Uuid) -> Self {
        Self::new(vec![user1, user2])
    }

    /// Update the last message
    pub fn update_last_message(&mut self, message: &ChatMessage, preview_len: usize) {
        self.last_message_preview = message.preview(preview_len);
        self.last_message_time = Some(message.timestamp.clone());
        self.last_message = Some(message.clone());
    }

    /// Check if user is a participant
    pub fn has_participant(&self, user_id: Uuid) -> bool {
        self.participants.contains(&user_id)
    }

    /// Get the other participant (for direct messages)
    pub fn other_participant(&self, current_user_id: Uuid) -> Option<Uuid> {
        self.participants
            .iter()
            .find(|&&id| id != current_user_id)
            .copied()
    }
}

/// Response for listing conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
}

/// Request to create a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub participant_ids: Vec<Uuid>,
}

/// Response after creating a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationResponse {
    pub success: bool,
    pub conversation: Option<Conversation>,
    pub error: Option<String>,
}

