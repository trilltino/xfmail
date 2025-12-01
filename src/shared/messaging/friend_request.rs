//! Friend Request Data Structure
//!
//! Represents friend requests between users.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};

/// Status of a friend request
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestStatus {
    /// Request is pending
    Pending,
    /// Request was accepted
    Accepted,
    /// Request was rejected
    Rejected,
    /// User is blocked
    Blocked,
}

impl Default for FriendRequestStatus {
    fn default() -> Self {
        FriendRequestStatus::Pending
    }
}

impl FriendRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FriendRequestStatus::Pending => "pending",
            FriendRequestStatus::Accepted => "accepted",
            FriendRequestStatus::Rejected => "rejected",
            FriendRequestStatus::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(FriendRequestStatus::Pending),
            "accepted" => Some(FriendRequestStatus::Accepted),
            "rejected" => Some(FriendRequestStatus::Rejected),
            "blocked" => Some(FriendRequestStatus::Blocked),
            _ => None,
        }
    }
}

/// Represents a friend request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FriendRequest {
    /// Unique request ID
    pub id: Uuid,
    /// User who sent the request
    pub from_user_id: Uuid,
    /// User who received the request
    pub to_user_id: Uuid,
    /// Username of the sender
    pub from_username: String,
    /// Email of the sender
    pub from_email: String,
    /// Email of the recipient
    pub to_email: String,
    /// Optional message with the request
    pub message: Option<String>,
    /// Current status of the request
    #[serde(default)]
    pub status: FriendRequestStatus,
    /// When the request was created
    #[cfg(feature = "ssr")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "ssr"))]
    pub created_at: String,
    /// When the request was responded to
    #[cfg(feature = "ssr")]
    pub responded_at: Option<DateTime<Utc>>,
    #[cfg(not(feature = "ssr"))]
    pub responded_at: Option<String>,
}

impl FriendRequest {
    /// Create a new friend request
    #[cfg(feature = "ssr")]
    pub fn new(
        from_user_id: Uuid,
        to_user_id: Uuid,
        from_username: String,
        from_email: String,
        to_email: String,
        message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            from_username,
            from_email,
            to_email,
            message,
            status: FriendRequestStatus::Pending,
            created_at: Utc::now(),
            responded_at: None,
        }
    }

    /// Check if the request is pending
    pub fn is_pending(&self) -> bool {
        self.status == FriendRequestStatus::Pending
    }
}

/// Request to send a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFriendRequestRequest {
    /// Email of the user to send request to
    pub to_email: String,
}

/// Response after sending a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFriendRequestResponse {
    pub success: bool,
    pub request_id: Option<Uuid>,
    pub error: Option<String>,
}

/// Request to respond to a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondFriendRequestRequest {
    /// ID of the request to respond to
    pub request_id: Uuid,
    /// Whether to accept (true) or reject (false)
    pub accept: bool,
}

/// Response after responding to a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondFriendRequestResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Response for listing friend requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFriendRequestsResponse {
    pub requests: Vec<FriendRequest>,
}

