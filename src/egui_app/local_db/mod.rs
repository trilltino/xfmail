//! # Local Database Module
//!
//! This module provides local SQLite database functionality for offline-first operations.
//! It implements a complete local-first storage system with CRDT support, synchronization
//! metadata, and offline operation queuing.
//!
//! ## Architecture
//!
//! The local database mirrors the server schema while adding offline-specific features:
//! - **Local State**: Stores user data, messages, contacts, and conversations
//! - **Sync Metadata**: Tracks synchronization state and version vectors
//! - **Offline Queue**: Queues operations for execution when online
//! - **CRDT Support**: Stores CRDT state for conflict-free replication
//!
//! ## Key Components
//!
//! - `LocalDatabase`: Main database connection and schema management
//! - `schema.rs`: Database schema definitions and migrations
//! - `messages.rs`: Message storage and retrieval operations
//! - `contacts.rs`: Contact management operations
//! - `conversations.rs`: Conversation handling operations
//! - `sync.rs`: Synchronization metadata and offline queue management
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::local_db::LocalDatabase;
//!
//! // Initialize local database
//! let db = LocalDatabase::new().await.expect("Failed to open local database");
//!
//! // Store a message
//! db.store_message(&message).await.expect("Failed to store message");
//!
//! // Retrieve messages for a conversation
//! let messages = db.get_conversation_messages(&conversation_id, Some(50)).await
//!     .expect("Failed to retrieve messages");
//! ```

pub mod schema;
pub mod messages;
pub mod contacts;
pub mod conversations;
pub mod sync;

use sqlx::{SqlitePool, Result as SqlxResult};
use std::path::Path;

/// Result type for local database operations
pub type Result<T> = SqlxResult<T>;

/// Local database connection manager
///
/// Manages the SQLite database connection pool and provides high-level operations
/// for local data storage and synchronization.
#[derive(Debug)]
pub struct LocalDatabase {
    pool: SqlitePool,
}

impl LocalDatabase {
    /// Open or create local database
    ///
    /// Creates the database file if it doesn't exist and initializes the schema.
    /// Uses WAL mode for better concurrency and performance.
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path();

        // Ensure directory exists
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create connection URL
        let database_url = format!("sqlite:{}", db_path);

        // Create connection pool
        let pool = SqlitePool::connect(&database_url).await?;

        // Enable WAL mode and other optimizations
        sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous=NORMAL").execute(&pool).await?;
        sqlx::query("PRAGMA cache_size=1000").execute(&pool).await?;
        sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;
        sqlx::query("PRAGMA temp_store=MEMORY").execute(&pool).await?;

        let db = Self { pool };

        // Initialize schema
        db.init_schema().await?;

        Ok(db)
    }

    /// Get database file path
    ///
    /// Returns the platform-specific path for the local database file.
    /// Uses the system's data directory when available.
    fn get_db_path() -> String {
        // Use platform-specific data directory
        let mut path = dirs::data_dir()
            .unwrap_or_else(|| std::env::temp_dir());

        path.push("xfmail");
        path.push("local.db");
        path.to_string_lossy().to_string()
    }

    /// Initialize database schema
    ///
    /// Creates all necessary tables and runs any pending migrations.
    async fn init_schema(&self) -> Result<()> {
        // Create tables
        sqlx::query(include_str!("schema.sql"))
            .execute(&self.pool)
            .await?;

        // Run migrations if needed
        self.run_migrations().await?;

        Ok(())
    }

    /// Run database migrations
    ///
    /// Checks the current schema version and applies any pending migrations.
    async fn run_migrations(&self) -> Result<()> {
        // Create migrations table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // Get current version
        let current_version: (i32,) = sqlx::query_as(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        // Apply migrations
        if current_version.0 < 1 {
            self.apply_migration_1().await?;
        }

        Ok(())
    }

    /// Migration 1: Initial schema
    ///
    /// Sets up the initial database schema and marks migration as applied.
    async fn apply_migration_1(&self) -> Result<()> {
        sqlx::query(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?)",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get connection pool reference
    ///
    /// Provides access to the database connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get database statistics
    ///
    /// Returns basic statistics about the local database for debugging.
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let message_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages",
        )
        .fetch_one(&self.pool)
        .await?;

        let contact_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contacts",
        )
        .fetch_one(&self.pool)
        .await?;

        let conversation_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM conversations",
        )
        .fetch_one(&self.pool)
        .await?;

        let pending_operations: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM offline_queue",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            message_count: message_count.0 as u64,
            contact_count: contact_count.0 as u64,
            conversation_count: conversation_count.0 as u64,
            pending_operations: pending_operations.0 as u64,
        })
    }

    /// Clean up old data
    ///
    /// Removes old messages and failed operations to manage storage space.
    pub async fn cleanup(&self, days_old: i32) -> Result<CleanupStats> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        // Get count before deletion for old messages
        let old_messages_before: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages WHERE created_at < ? AND is_read = 1",
        )
        .bind(cutoff_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        // Delete old messages
        sqlx::query(
            "DELETE FROM messages WHERE created_at < ? AND is_read = 1",
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Get count before deletion for failed operations
        let failed_ops_before: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM offline_queue WHERE created_at < ? AND retry_count > 5",
        )
        .bind(cutoff_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        // Delete failed operations
        sqlx::query(
            "DELETE FROM offline_queue WHERE created_at < ? AND retry_count > 5",
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(CleanupStats {
            old_messages_removed: old_messages_before.0 as u64,
            failed_operations_removed: failed_ops_before.0 as u64,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of messages stored locally
    pub message_count: u64,
    /// Total number of contacts stored locally
    pub contact_count: u64,
    /// Total number of conversations stored locally
    pub conversation_count: u64,
    /// Number of pending offline operations
    pub pending_operations: u64,
}

/// Cleanup operation statistics
#[derive(Debug, Clone)]
pub struct CleanupStats {
    /// Number of old messages removed
    pub old_messages_removed: u64,
    /// Number of failed operations removed
    pub failed_operations_removed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = LocalDatabase::new().await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_database_stats() {
        let db = LocalDatabase::new().await.unwrap();
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.message_count, 0); // Should be empty initially
        assert_eq!(stats.contact_count, 0);
        assert_eq!(stats.conversation_count, 0);
        assert_eq!(stats.pending_operations, 0);
    }
}