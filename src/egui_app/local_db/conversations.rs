//! # Local Conversation Operations
//!
//! Provides comprehensive CRUD operations for conversations in the local SQLite database.
//! Manages conversation metadata, participant lists, and synchronization state.
//!
//! ## Features
//!
//! - **Conversation Storage**: Store and retrieve conversation metadata
//! - **Participant Management**: Track conversation participants
//! - **Sync Tracking**: Track synchronization state for conversations
//! - **Efficient Queries**: Fast conversation retrieval with participant filtering
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::local_db::LocalDatabase;
//!
//! let mut db = LocalDatabase::new().unwrap();
//!
//! // Store a conversation
//! db.store_conversation(&conversation).unwrap();
//!
//! // Get user conversations
//! let conversations = db.get_conversations(Some(&user_id)).unwrap();
//!
//! // Get conversation participants
//! let participants = db.get_conversation_participants(&conversation_id).unwrap();
//! ```

use crate::shared::messaging::Conversation;
use crate::egui_app::local_db::LocalDatabase;
use sqlx::{Result as SqlxResult, Row};
use uuid::Uuid;

/// Result type alias for conversation operations
pub type Result<T> = SqlxResult<T>;

impl LocalDatabase {
    /// Store a conversation locally
    pub async fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        // Store conversation
        sqlx::query(
            "INSERT OR REPLACE INTO conversations (
                id, created_at, updated_at, needs_sync
            ) VALUES (?, ?, ?, ?)",
        )
        .bind(conversation.id.to_string())
        .bind(&conversation.created_at)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(true) // Mark as needing sync
        .execute(&self.pool)
        .await?;

        // Store participants
        for participant_id in &conversation.participants {
            sqlx::query(
                "INSERT OR REPLACE INTO conversation_participants (
                    conversation_id, user_id, role, joined_at, needs_sync
                ) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(conversation.id.to_string())
            .bind(participant_id.to_string())
            .bind("member")
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(true)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get all conversations for current user
    pub async fn get_conversations(&self, current_user_id: Option<&Uuid>) -> Result<Vec<Conversation>> {
        if let Some(user_id) = current_user_id {
            let rows = sqlx::query(
                "SELECT DISTINCT c.id, c.created_at
                 FROM conversations c
                 INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
                 WHERE cp.user_id = ?
                 ORDER BY c.updated_at DESC"
            )
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;

            let mut conversations = Vec::new();
            for row in rows {
                let mut conversation = self.row_to_conversation(&row)?;
                // Populate participants
                conversation.participants = self.get_conversation_participants(&conversation.id).await?;
                conversations.push(conversation);
            }
            Ok(conversations)
        } else {
            Ok(vec![])
        }
    }

    /// Get a conversation by ID
    pub async fn get_conversation(&self, conversation_id: &Uuid) -> Result<Option<Conversation>> {
        let row = sqlx::query(
            "SELECT id, name, conversation_type, created_by, created_at, updated_at
             FROM conversations
             WHERE id = ?"
        )
        .bind(conversation_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_conversation(&row)?)),
            None => Ok(None),
        }
    }

    /// Get conversation participants
    pub async fn get_conversation_participants(&self, conversation_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            "SELECT user_id FROM conversation_participants WHERE conversation_id = ?"
        )
        .bind(conversation_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut participants = Vec::new();
        for row in rows {
            let user_id_str: String = row.try_get("user_id")?;
            participants.push(Uuid::parse_str(&user_id_str).unwrap_or_default());
        }
        Ok(participants)
    }

    /// Mark conversation as synced
    pub async fn mark_conversation_synced(&self, conversation_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE conversations SET needs_sync = 0, last_synced_at = ? WHERE id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(conversation_id.to_string())
        .execute(&self.pool)
        .await?;

        // Also mark participants as synced
        sqlx::query(
            "UPDATE conversation_participants SET needs_sync = 0, last_synced_at = ? WHERE conversation_id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(conversation_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get unsynced conversations
    pub async fn get_unsynced_conversations(&self) -> Result<Vec<Conversation>> {
        let rows = sqlx::query(
            "SELECT id, name, conversation_type, created_by, created_at, updated_at
             FROM conversations
             WHERE needs_sync = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(self.row_to_conversation(&row)?);
        }
        Ok(conversations)
    }

    /// Convert database row to Conversation (without participants - call get_conversation_participants separately)
    fn row_to_conversation(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Conversation> {
        let conversation_id = Uuid::parse_str(&row.try_get::<String, _>("id")?).unwrap_or_default();

        Ok(Conversation {
            id: conversation_id,
            participants: Vec::new(), // Will be populated by caller if needed
            other_username: None, // TODO: Calculate from participants
            last_message: None, // TODO: Get from messages
            last_message_preview: String::new(), // TODO: Calculate from last message
            last_message_time: None, // TODO: Get from last message
            unread_count: 0, // TODO: Calculate unread count
            created_at: row.try_get("created_at")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_conversation() {
        let db = LocalDatabase::new().await.unwrap();

        let conversation = Conversation {
            id: Uuid::new_v4(),
            participants: vec![Uuid::new_v4(), Uuid::new_v4()],
            other_username: None,
            last_message: None,
            last_message_preview: String::new(),
            last_message_time: None,
            unread_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store conversation
        db.store_conversation(&conversation).await.unwrap();

        // Retrieve conversation
        let retrieved = db.get_conversation(&conversation.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, conversation.id);
        assert_eq!(retrieved.participants.len(), conversation.participants.len());
    }

    #[tokio::test]
    async fn test_get_conversations_for_user() {
        let db = LocalDatabase::new().await.unwrap();

        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        let conversation1 = Conversation {
            id: Uuid::new_v4(),
            participants: vec![user_id, other_user_id],
            other_username: None,
            last_message: None,
            last_message_preview: String::new(),
            last_message_time: None,
            unread_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let conversation2 = Conversation {
            id: Uuid::new_v4(),
            participants: vec![user_id, Uuid::new_v4()],
            other_username: None,
            last_message: None,
            last_message_preview: String::new(),
            last_message_time: None,
            unread_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.store_conversation(&conversation1).await.unwrap();
        db.store_conversation(&conversation2).await.unwrap();

        // Get conversations for user
        let user_conversations = db.get_conversations(Some(&user_id)).await.unwrap();

        assert_eq!(user_conversations.len(), 2);
        assert!(user_conversations.iter().any(|c| c.id == conversation1.id));
        assert!(user_conversations.iter().any(|c| c.id == conversation2.id));
    }
}