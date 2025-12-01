//! # Local Contact Operations
//!
//! Provides comprehensive CRUD operations for contacts in the local SQLite database.
//! Manages friend relationships, contact information, and synchronization state.
//!
//! ## Features
//!
//! - **Contact Storage**: Store and retrieve contact information locally
//! - **Search**: Efficient contact search by username or email
//! - **Sync Tracking**: Track synchronization state for contacts
//! - **Relationship Management**: Handle friend request states
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::local_db::LocalDatabase;
//!
//! let db = LocalDatabase::new().await.unwrap();
//!
//! // Store a contact
//! db.store_contact(&contact).await.unwrap();
//!
//! // Search contacts
//! let results = db.search_contacts("alice").await.unwrap();
//!
//! // Get contact by user ID
//! let contact = db.get_contact_by_user_id(&user_id).await.unwrap();
//! ```

use crate::shared::messaging::Contact;
use crate::egui_app::local_db::LocalDatabase;
use sqlx::{Result as SqlxResult, Row};
use uuid::Uuid;

/// Result type alias for contact operations
pub type Result<T> = SqlxResult<T>;

impl LocalDatabase {
    /// Store a contact locally
    pub async fn store_contact(&self, contact: &Contact) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO contacts (
                id, contact_user_id, username, email, display_name,
                created_at, updated_at, needs_sync
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(contact.id.to_string())
        .bind(contact.contact_user_id.to_string())
        .bind(&contact.username)
        .bind(&contact.email)
        .bind(&contact.display_name)
        .bind(&contact.created_at)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(true) // Mark as needing sync
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        let rows = sqlx::query(
            "SELECT id, contact_user_id, username, email, display_name
             FROM contacts
             WHERE status = 'active'
             ORDER BY username ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            contacts.push(self.row_to_contact(&row)?);
        }

        Ok(contacts)
    }

    /// Get a contact by user ID
    pub async fn get_contact_by_user_id(&self, user_id: &Uuid) -> Result<Option<Contact>> {
        let row = sqlx::query(
            "SELECT id, contact_user_id, username, email, display_name
             FROM contacts
             WHERE contact_user_id = ? AND status = 'active'"
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_contact(&row)?)),
            None => Ok(None),
        }
    }

    /// Search contacts by username or email
    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        let rows = sqlx::query(
            "SELECT id, contact_user_id, username, email, display_name
             FROM contacts
             WHERE status = 'active' AND (
                 LOWER(username) LIKE ? OR
                 LOWER(email) LIKE ? OR
                 LOWER(display_name) LIKE ?
             )
             ORDER BY username ASC"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            contacts.push(self.row_to_contact(&row)?);
        }

        Ok(contacts)
    }

    /// Update contact information
    pub async fn update_contact(&self, contact: &Contact) -> Result<()> {
        sqlx::query(
            "UPDATE contacts SET
                username = ?, email = ?, display_name = ?,
                updated_at = ?, needs_sync = 1
             WHERE contact_user_id = ?",
        )
        .bind(&contact.username)
        .bind(&contact.email)
        .bind(&contact.display_name)
        .bind(chrono::Utc::now())
        .bind(contact.contact_user_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark contact as synced
    pub async fn mark_contact_synced(&self, contact_user_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE contacts SET needs_sync = 0, last_synced_at = ? WHERE contact_user_id = ?",
        )
        .bind(chrono::Utc::now())
        .bind(contact_user_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get unsynced contacts
    pub async fn get_unsynced_contacts(&self) -> Result<Vec<Contact>> {
        let rows = sqlx::query(
            "SELECT id, contact_user_id, username, email, display_name
             FROM contacts
             WHERE needs_sync = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            contacts.push(self.row_to_contact(&row)?);
        }

        Ok(contacts)
    }

    /// Convert database row to Contact
    fn row_to_contact(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Contact> {
        Ok(Contact {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).unwrap_or_default(),
            user_id: Uuid::nil(), // TODO: Store and retrieve user_id
            contact_user_id: Uuid::parse_str(&row.try_get::<String, _>("contact_user_id")?).unwrap_or_default(),
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            avatar_url: None, // TODO: Add avatar support
            #[cfg(feature = "ssr")]
            last_seen: chrono::Utc::now(), // TODO: Store last_seen
            #[cfg(not(feature = "ssr"))]
            last_seen: chrono::Utc::now().to_rfc3339(), // TODO: Store last_seen
            is_online: false, // TODO: Add online status
            #[cfg(feature = "ssr")]
            created_at: chrono::Utc::now(), // TODO: Store created_at
            #[cfg(not(feature = "ssr"))]
            created_at: chrono::Utc::now().to_rfc3339(), // TODO: Store created_at
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_contact() {
        let db = LocalDatabase::new().await.unwrap();

        let contact = Contact {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            contact_user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            last_seen: chrono::Utc::now().to_rfc3339(),
            is_online: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store contact
        db.store_contact(&contact).await.unwrap();

        // Retrieve contact
        let retrieved = db.get_contact_by_user_id(&contact.contact_user_id).await.unwrap().unwrap();

        assert_eq!(retrieved.contact_user_id, contact.contact_user_id);
        assert_eq!(retrieved.username, contact.username);
        assert_eq!(retrieved.email, contact.email);
        assert_eq!(retrieved.display_name, contact.display_name);
    }

    #[tokio::test]
    async fn test_search_contacts() {
        let db = LocalDatabase::new().await.unwrap();

        let contact1 = Contact {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            contact_user_id: Uuid::new_v4(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            display_name: Some("Alice Smith".to_string()),
            avatar_url: None,
            last_seen: chrono::Utc::now().to_rfc3339(),
            is_online: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let contact2 = Contact {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            contact_user_id: Uuid::new_v4(),
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            display_name: Some("Bob Johnson".to_string()),
            avatar_url: None,
            last_seen: chrono::Utc::now().to_rfc3339(),
            is_online: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.store_contact(&contact1).await.unwrap();
        db.store_contact(&contact2).await.unwrap();

        // Search by username
        let results = db.search_contacts("alice").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].username, "alice");

        // Search by display name
        let results = db.search_contacts("smith").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, Some("Alice Smith".to_string()));
    }
}