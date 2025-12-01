//! # Sync Metadata Operations
//!
//! Manages synchronization metadata, offline operation queuing, and sync state tracking.
//! Provides the foundation for background synchronization and offline-first functionality.
//!
//! ## Features
//!
//! - **Offline Queue**: Queue operations for execution when online
//! - **Sync Metadata**: Track synchronization state and timestamps
//! - **Retry Logic**: Automatic retry with exponential backoff
//! - **Cleanup**: Remove old failed operations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::local_db::{LocalDatabase, sync::SyncOperation};
//!
//! let mut db = LocalDatabase::new().unwrap();
//!
//! // Queue an operation for later sync
//! let operation = SyncOperation::SendMessage {
//!     conversation_id: conversation_id,
//!     content: "Hello!".to_string(),
//! };
//! db.add_to_offline_queue(operation).unwrap();
//!
//! // Process pending operations
//! let operations = db.get_pending_operations().unwrap();
//! for op in operations {
//!     // Process operation...
//!     db.complete_operation(&op.id).unwrap();
//! }
//! ```

use crate::egui_app::local_db::LocalDatabase;
use sqlx::{Result as SqlxResult, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result type alias for sync operations
pub type Result<T> = SqlxResult<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    SendMessage { conversation_id: Uuid, content: String },
    MarkMessageRead { message_id: Uuid },
    AddContact { contact_user_id: Uuid },
    SendFriendRequest { target_email: String, message: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueueItem {
    pub id: String,
    pub operation: SyncOperation,
    pub created_at: String,
    pub retry_count: i32,
    pub last_attempt: Option<String>,
    pub error_message: Option<String>,
}

impl LocalDatabase {
    /// Add operation to offline queue
    pub async fn add_to_offline_queue(&self, operation: SyncOperation) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let data = serde_json::to_string(&operation)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization error: {}", e)))
            .unwrap(); // This should never fail for our operation types

        sqlx::query(
            "INSERT INTO offline_queue (id, operation_type, data, created_at, retry_count)
             VALUES (?, ?, ?, ?, 0)",
        )
        .bind(&id)
        .bind(Self::operation_type_name(&operation))
        .bind(&data)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get pending offline operations
    pub async fn get_pending_operations(&self) -> Result<Vec<OfflineQueueItem>> {
        let rows = sqlx::query(
            "SELECT id, operation_type, data, created_at, retry_count, last_attempt, error_message
             FROM offline_queue
             ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut operations = Vec::new();
        for row in rows {
            let operation_type: String = row.try_get("operation_type")?;
            let data: String = row.try_get("data")?;

            let operation = match operation_type.as_str() {
                "send_message" => serde_json::from_str::<SyncOperation>(&data),
                "mark_read" => serde_json::from_str::<SyncOperation>(&data),
                "add_contact" => serde_json::from_str::<SyncOperation>(&data),
                "friend_request" => serde_json::from_str::<SyncOperation>(&data),
                _ => continue, // Skip unknown operation types
            };

            let operation = match operation {
                Ok(op) => op,
                Err(_) => continue, // Skip malformed operations
            };

            operations.push(OfflineQueueItem {
                id: row.try_get("id")?,
                operation,
                created_at: row.try_get("created_at")?,
                retry_count: row.try_get("retry_count")?,
                last_attempt: row.try_get("last_attempt")?,
                error_message: row.try_get("error_message")?,
            });
        }

        Ok(operations)
    }

    /// Mark operation as completed
    pub async fn complete_operation(&self, operation_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM offline_queue WHERE id = ?",
        )
        .bind(operation_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update operation retry status
    pub async fn update_operation_retry(&self, operation_id: &str, error_message: Option<&str>) -> Result<()> {
        sqlx::query(
            "UPDATE offline_queue SET
                retry_count = retry_count + 1,
                last_attempt = ?,
                error_message = ?
             WHERE id = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(error_message)
        .bind(operation_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Clean up old failed operations
    pub async fn cleanup_failed_operations(&self, max_age_days: i32, max_retries: i32) -> Result<u64> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);

        let result = sqlx::query(
            "DELETE FROM offline_queue
             WHERE (created_at < ? AND retry_count > ?) OR retry_count > ?",
        )
        .bind(cutoff_date.to_rfc3339())
        .bind(max_retries / 2)
        .bind(max_retries)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Set sync metadata
    pub async fn set_sync_metadata(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO sync_metadata (key, value, updated_at)
             VALUES (?, ?, ?)",
        )
        .bind(key)
        .bind(value)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get sync metadata
    pub async fn get_sync_metadata(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query(
            "SELECT value FROM sync_metadata WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row.try_get("value")?)),
            None => Ok(None),
        }
    }

    /// Get last sync timestamp
    pub async fn get_last_sync_time(&self) -> Result<Option<String>> {
        self.get_sync_metadata("last_sync_time").await
    }

    /// Set last sync timestamp
    pub async fn set_last_sync_time(&self) -> Result<()> {
        self.set_sync_metadata("last_sync_time", &chrono::Utc::now().to_rfc3339()).await
    }

    /// Get operation type name for database storage
    fn operation_type_name(operation: &SyncOperation) -> &'static str {
        match operation {
            SyncOperation::SendMessage { .. } => "send_message",
            SyncOperation::MarkMessageRead { .. } => "mark_read",
            SyncOperation::AddContact { .. } => "add_contact",
            SyncOperation::SendFriendRequest { .. } => "friend_request",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_offline_queue_operations() {
        let db = LocalDatabase::new().await.unwrap();

        let operation = SyncOperation::SendMessage {
            conversation_id: Uuid::new_v4(),
            content: "Test message".to_string(),
        };

        // Add to queue
        let operation_id = db.add_to_offline_queue(operation.clone()).await.unwrap();

        // Retrieve from queue
        let operations = db.get_pending_operations().await.unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, operation_id);

        // Complete operation
        db.complete_operation(&operation_id).await.unwrap();

        // Verify queue is empty
        let operations = db.get_pending_operations().await.unwrap();
        assert_eq!(operations.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_metadata() {
        let db = LocalDatabase::new().await.unwrap();

        // Set metadata
        db.set_sync_metadata("test_key", "test_value").await.unwrap();

        // Get metadata
        let value = db.get_sync_metadata("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Get non-existent metadata
        let value = db.get_sync_metadata("non_existent").await.unwrap();
        assert_eq!(value, None);
    }
}