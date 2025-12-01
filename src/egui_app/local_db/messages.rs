//! # Local Message Operations
//!
//! Provides comprehensive CRUD operations for messages in the local SQLite database.
//! Handles message storage, retrieval, synchronization metadata, and offline queuing.
//!
//! ## Features
//!
//! - **Message Storage**: Store and retrieve chat messages locally
//! - **Sync Tracking**: Track synchronization state for each message
//! - **Offline Support**: Queue messages for sending when offline
//! - **Cleanup**: Automatic cleanup of old read messages
//! - **Search**: Efficient message retrieval by conversation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::local_db::LocalDatabase;
//!
//! let db = LocalDatabase::new().await.unwrap();
//!
//! // Store a message
//! db.store_message(&message).await.unwrap();
//!
//! // Get recent messages
//! let messages = db.get_conversation_messages(&conversation_id, Some(50)).await.unwrap();
//!
//! // Mark as read
//! db.mark_message_read(&message_id).await.unwrap();
//! ```

use crate::shared::messaging::ChatMessage;
use crate::egui_app::local_db::LocalDatabase;
use sqlx::{Result as SqlxResult, Row};
use uuid::Uuid;

/// Result type alias for message operations
pub type Result<T> = SqlxResult<T>;

impl LocalDatabase {
    /// Store a message locally
    ///
    /// Stores a chat message in the local database with full metadata.
    /// Marks the message as needing synchronization if it's new.
    pub async fn store_message(&self, message: &ChatMessage) -> Result<()> {
        let braid_parents_json = serde_json::to_string(&message.braid_parents)
            .unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            "INSERT OR REPLACE INTO messages (
                id, conversation_id, sender_id, content, message_type,
                timestamp, is_read, is_delivered, crdt_timestamp,
                braid_version, braid_parents, delivery_status,
                created_at, updated_at, needs_sync
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(message.id.to_string())
        .bind(message.conversation_id.to_string())
        .bind(message.sender_id.to_string())
        .bind(&message.content)
        .bind(message.message_type.to_string())
        .bind(&message.timestamp)
        .bind(message.is_read)
        .bind(message.is_delivered)
        .bind(message.crdt_timestamp as i64)
        .bind(&message.braid_version)
        .bind(braid_parents_json)
        .bind("sent") // Default delivery status
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(true) // Mark as needing sync
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get messages for a conversation
    pub async fn get_conversation_messages(&self, conversation_id: &Uuid, limit: Option<i32>) -> Result<Vec<ChatMessage>> {
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let query = format!(
            "SELECT id, conversation_id, sender_id, content, message_type,
                    timestamp, is_read, is_delivered, crdt_timestamp,
                    braid_version, braid_parents
             FROM messages
             WHERE conversation_id = ?
             ORDER BY crdt_timestamp ASC
             {}",
            limit_clause
        );

        let rows = sqlx::query(&query)
            .bind(conversation_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(self.row_to_message(&row)?);
        }

        Ok(messages)
    }

    /// Get a single message by ID
    pub async fn get_message(&self, message_id: &Uuid) -> Result<Option<ChatMessage>> {
        let row = sqlx::query(
            "SELECT id, conversation_id, sender_id, content, message_type,
                    timestamp, is_read, is_delivered, crdt_timestamp,
                    braid_version, braid_parents
             FROM messages
             WHERE id = ?"
        )
        .bind(message_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_message(&row)?)),
            None => Ok(None),
        }
    }

    /// Mark message as read
    pub async fn mark_message_read(&self, message_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE messages SET is_read = 1, updated_at = ?, needs_sync = 1 WHERE id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(message_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark message as delivered
    pub async fn mark_message_delivered(&self, message_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE messages SET is_delivered = 1, delivery_status = 'delivered', updated_at = ?, needs_sync = 1 WHERE id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(message_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get unsynced messages
    pub async fn get_unsynced_messages(&self) -> Result<Vec<ChatMessage>> {
        let rows = sqlx::query(
            "SELECT id, conversation_id, sender_id, content, message_type,
                    timestamp, is_read, is_delivered, crdt_timestamp,
                    braid_version, braid_parents
             FROM messages
             WHERE needs_sync = 1
             ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(self.row_to_message(&row)?);
        }

        Ok(messages)
    }

    /// Mark message as synced
    pub async fn mark_message_synced(&self, message_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE messages SET needs_sync = 0, last_synced_at = ? WHERE id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(message_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete old messages (for storage management)
    pub async fn cleanup_old_messages(&self, days_old: i32) -> Result<usize> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        let result = sqlx::query(
            "DELETE FROM messages WHERE created_at < ? AND is_read = 1",
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// Convert database row to ChatMessage
    fn row_to_message(&self, row: &sqlx::sqlite::SqliteRow) -> Result<ChatMessage> {
        let braid_parents_json: String = row.try_get("braid_parents")?;
        let braid_parents: Vec<String> = serde_json::from_str(&braid_parents_json)
            .unwrap_or_default();

        Ok(ChatMessage {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).unwrap_or_default(),
            conversation_id: Uuid::parse_str(&row.try_get::<String, _>("conversation_id")?).unwrap_or_default(),
            sender_id: Uuid::parse_str(&row.try_get::<String, _>("sender_id")?).unwrap_or_default(),
            content: row.try_get("content")?,
            message_type: crate::shared::messaging::MessageType::from_str(&row.try_get::<String, _>("message_type")?),
            timestamp: row.try_get("timestamp")?,
            is_read: row.try_get("is_read")?,
            is_delivered: row.try_get("is_delivered")?,
            crdt_timestamp: row.try_get("crdt_timestamp")?,
            braid_version: row.try_get("braid_version")?,
            braid_parents,
            version_vector: crate::shared::messaging::message::VersionVector::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::messaging::MessageType;

    #[tokio::test]
    async fn test_store_and_retrieve_message() {
        let db = LocalDatabase::new().await.unwrap();

        let message = ChatMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            content: "Test message".to_string(),
            message_type: MessageType::Text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_read: false,
            is_delivered: false,
            crdt_timestamp: 12345,
            braid_version: "v1".to_string(),
            braid_parents: vec![],
            version_vector: crate::shared::messaging::message::VersionVector::default(),
        };

        // Store message
        db.store_message(&message).await.unwrap();

        // Retrieve message
        let retrieved = db.get_message(&message.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, message.id);
        assert_eq!(retrieved.content, message.content);
        assert_eq!(retrieved.crdt_timestamp, message.crdt_timestamp);
    }
}