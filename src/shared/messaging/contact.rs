//! Contact Data Structure
//!
//! Represents a user's contact (friend) in the messaging system.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};

/// Represents a contact (friend) in the messaging system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contact {
    /// Unique contact ID
    pub id: Uuid,
    /// The user who owns this contact entry
    pub user_id: Uuid,
    /// The user ID of the contact
    pub contact_user_id: Uuid,
    /// Contact's username
    pub username: String,
    /// Contact's email (used for lookup)
    pub email: String,
    /// Optional display name
    pub display_name: Option<String>,
    /// Optional avatar URL
    pub avatar_url: Option<String>,
    /// Last seen timestamp
    #[cfg(feature = "ssr")]
    pub last_seen: DateTime<Utc>,
    #[cfg(not(feature = "ssr"))]
    pub last_seen: String,
    /// Whether the contact is currently online
    pub is_online: bool,
    /// When the contact was added
    #[cfg(feature = "ssr")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "ssr"))]
    pub created_at: String,
}

impl Contact {
    /// Create a new contact
    #[cfg(feature = "ssr")]
    pub fn new(
        user_id: Uuid,
        contact_user_id: Uuid,
        username: String,
        email: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            contact_user_id,
            username,
            email,
            display_name: None,
            avatar_url: None,
            last_seen: now,
            is_online: false,
            created_at: now,
        }
    }

    /// Get display name or fallback to username
    pub fn display_name_or_username(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }

    /// Get avatar initial (first letter of username)
    pub fn avatar_initial(&self) -> char {
        self.username.chars().next().unwrap_or('?').to_ascii_uppercase()
    }
}

/// Response type for listing contacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContactsResponse {
    pub contacts: Vec<Contact>,
}

/// Response type for getting a single contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContactResponse {
    pub contact: Contact,
}

