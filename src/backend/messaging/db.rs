//! Database operations for messaging
//!
//! This module contains database operations for friend requests, contacts, and messages.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use crate::shared::messaging::{
    Contact, FriendRequest, FriendRequestStatus,
};

/// Create a new friend request
pub async fn create_friend_request(
    pool: &PgPool,
    from_user_id: Uuid,
    to_user_id: Uuid,
    from_username: &str,
    from_email: &str,
    to_email: &str,
    message: Option<&str>,
) -> Result<FriendRequest, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO friend_requests (id, from_user_id, to_user_id, from_username, from_email, to_email, message, status, created_at, responded_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, NULL)
        "#
    )
    .bind(id)
    .bind(from_user_id)
    .bind(to_user_id)
    .bind(from_username)
    .bind(from_email)
    .bind(to_email)
    .bind(message)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(FriendRequest {
        id,
        from_user_id,
        to_user_id,
        from_username: from_username.to_string(),
        from_email: from_email.to_string(),
        to_email: to_email.to_string(),
        message: message.map(|s| s.to_string()),
        status: FriendRequestStatus::Pending,
        created_at: now,
        responded_at: None,
    })
}

/// Get pending friend requests for a user
pub async fn get_pending_friend_requests(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<FriendRequest>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, from_user_id, to_user_id, from_username, from_email, to_email, message, status, created_at, responded_at
        FROM friend_requests
        WHERE to_user_id = $1 AND status = 'pending'
        ORDER BY created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| FriendRequest {
        id: row.get("id"),
        from_user_id: row.get("from_user_id"),
        to_user_id: row.get("to_user_id"),
        from_username: row.get("from_username"),
        from_email: row.get("from_email"),
        to_email: row.get("to_email"),
        message: row.get("message"),
        status: FriendRequestStatus::from_str(row.get::<String, _>("status").as_str()).unwrap_or(FriendRequestStatus::Pending),
        created_at: row.get("created_at"),
        responded_at: row.get("responded_at"),
    }).collect())
}

/// Get a friend request by ID
pub async fn get_friend_request_by_id(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<FriendRequest>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, from_user_id, to_user_id, from_username, from_email, to_email, message, status, created_at, responded_at
        FROM friend_requests
        WHERE id = $1
        "#
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| FriendRequest {
        id: r.get("id"),
        from_user_id: r.get("from_user_id"),
        to_user_id: r.get("to_user_id"),
        from_username: r.get("from_username"),
        from_email: r.get("from_email"),
        to_email: r.get("to_email"),
        message: r.get("message"),
        status: FriendRequestStatus::from_str(r.get::<String, _>("status").as_str()).unwrap_or(FriendRequestStatus::Pending),
        created_at: r.get("created_at"),
        responded_at: r.get("responded_at"),
    }))
}

/// Accept a friend request
pub async fn accept_friend_request(
    pool: &PgPool,
    request_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Update the request status
    sqlx::query(
        r#"
        UPDATE friend_requests
        SET status = 'accepted', responded_at = $1
        WHERE id = $2 AND to_user_id = $3
        "#
    )
    .bind(now)
    .bind(request_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Reject a friend request
pub async fn reject_friend_request(
    pool: &PgPool,
    request_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE friend_requests
        SET status = 'rejected', responded_at = $1
        WHERE id = $2 AND to_user_id = $3
        "#
    )
    .bind(now)
    .bind(request_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a contact entry (called when friend request is accepted)
pub async fn create_contact(
    pool: &PgPool,
    user_id: Uuid,
    contact_user_id: Uuid,
    username: &str,
    email: &str,
) -> Result<Contact, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO contacts (id, user_id, contact_user_id, username, email, created_at, last_seen, is_online)
        VALUES ($1, $2, $3, $4, $5, $6, $7, false)
        "#
    )
    .bind(id)
    .bind(user_id)
    .bind(contact_user_id)
    .bind(username)
    .bind(email)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Contact {
        id,
        user_id,
        contact_user_id,
        username: username.to_string(),
        email: email.to_string(),
        display_name: None,
        avatar_url: None,
        last_seen: now,
        is_online: false,
        created_at: now,
    })
}

/// Get all contacts for a user
pub async fn get_contacts_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Contact>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, contact_user_id, username, email, display_name, avatar_url, last_seen, is_online, created_at
        FROM contacts
        WHERE user_id = $1
        ORDER BY username ASC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| Contact {
        id: row.get("id"),
        user_id: row.get("user_id"),
        contact_user_id: row.get("contact_user_id"),
        username: row.get("username"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        last_seen: row.get("last_seen"),
        is_online: row.get("is_online"),
        created_at: row.get("created_at"),
    }).collect())
}

/// Delete a contact
pub async fn delete_contact(
    pool: &PgPool,
    contact_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM contacts
        WHERE id = $1 AND user_id = $2
        "#
    )
    .bind(contact_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a conversation between two users
pub async fn create_conversation(
    pool: &PgPool,
    user1_id: Uuid,
    user2_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let conversation_id = Uuid::new_v4();
    let now = Utc::now();

    // Create the conversation
    sqlx::query(
        r#"
        INSERT INTO conversations (id, created_at, updated_at)
        VALUES ($1, $2, $3)
        "#
    )
    .bind(conversation_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Add both participants
    sqlx::query(
        r#"
        INSERT INTO conversation_participants (conversation_id, user_id, joined_at)
        VALUES ($1, $2, $3), ($1, $4, $3)
        "#
    )
    .bind(conversation_id)
    .bind(user1_id)
    .bind(now)
    .bind(user2_id)
    .execute(pool)
    .await?;

    Ok(conversation_id)
}

/// Get conversations for a user
pub async fn get_conversations_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<crate::shared::messaging::Conversation>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.created_at, c.updated_at
        FROM conversations c
        INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
        WHERE cp.user_id = $1
        ORDER BY c.updated_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut conversations = Vec::new();
    for row in rows {
        let conv_id: Uuid = row.get("id");

        // Get participants
        let participant_rows = sqlx::query(
            r#"
            SELECT user_id FROM conversation_participants WHERE conversation_id = $1
            "#
        )
        .bind(conv_id)
        .fetch_all(pool)
        .await?;

        let participants: Vec<Uuid> = participant_rows.iter().map(|r| r.get("user_id")).collect();

        // Fetch other user's username (assuming 2-person conversations)
        let other_username = if participants.len() == 2 {
            let other_user_id = participants.iter().find(|&&p| p != user_id).unwrap();
            // Get the username from contacts table
            sqlx::query(
                r#"
                SELECT username FROM contacts WHERE user_id = $1 AND contact_user_id = $2
                "#
            )
            .bind(user_id)
            .bind(other_user_id)
            .fetch_optional(pool)
            .await?
            .map(|row| row.get::<String, _>("username"))
        } else {
            None
        };

        // Convert timestamps to RFC3339 strings
        let updated_at_dt: chrono::DateTime<chrono::Utc> = row.get("updated_at");
        let created_at_dt: chrono::DateTime<chrono::Utc> = row.get("created_at");

        conversations.push(crate::shared::messaging::Conversation {
            id: conv_id,
            participants,
            other_username,
            last_message: None,
            last_message_preview: String::new(),
            last_message_time: Some(updated_at_dt.to_rfc3339()),
            unread_count: 0,
            created_at: created_at_dt.to_rfc3339(),
        });
    }

    Ok(conversations)
}

/// Store a message in the database
pub async fn store_message(
    pool: &PgPool,
    message: &crate::shared::messaging::ChatMessage,
) -> Result<(), sqlx::Error> {
    // Convert RFC3339 string to chrono for DB
    let created_at_dt = chrono::DateTime::parse_from_rfc3339(&message.timestamp)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    sqlx::query(
        r#"
        INSERT INTO chat_messages (id, conversation_id, sender_id, content, message_type, is_read, is_delivered, crdt_timestamp, braid_version, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(message.id)
    .bind(message.conversation_id)
    .bind(message.sender_id)
    .bind(&message.content)
    .bind(message.message_type.to_string())
    .bind(message.is_read)
    .bind(message.is_delivered)
    .bind(message.crdt_timestamp as i64)
    .bind(&message.braid_version)
    .bind(created_at_dt)
    .execute(pool)
    .await?;

    // Update conversation updated_at
    sqlx::query(
        r#"
        UPDATE conversations SET updated_at = $1 WHERE id = $2
        "#
    )
    .bind(created_at_dt)
    .bind(message.conversation_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get messages for a conversation
pub async fn get_messages_for_conversation(
    pool: &PgPool,
    conversation_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::shared::messaging::ChatMessage>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, conversation_id, sender_id, content, message_type, is_read, is_delivered, crdt_timestamp, braid_version, created_at
        FROM chat_messages
        WHERE conversation_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| {
        let msg_type_str: String = row.get("message_type");
        let created_at_dt: chrono::DateTime<chrono::Utc> = row.get("created_at");
        crate::shared::messaging::ChatMessage {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            sender_id: row.get("sender_id"),
            content: row.get("content"),
            message_type: crate::shared::messaging::MessageType::from_str(&msg_type_str),
            timestamp: created_at_dt.to_rfc3339(),
            is_read: row.get("is_read"),
            is_delivered: row.get("is_delivered"),
            crdt_timestamp: row.get::<i64, _>("crdt_timestamp") as u64,
            braid_version: row.get("braid_version"),
            braid_parents: vec![],
            version_vector: crate::shared::messaging::message::VersionVector::default(),
        }
    }).collect())
}

/// Mark a message as read
pub async fn mark_message_read(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_messages SET is_read = true WHERE id = $1
        "#
    )
    .bind(message_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if a user is a participant in a conversation
pub async fn is_user_participant_in_conversation(
    pool: &PgPool,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM conversation_participants
        WHERE conversation_id = $1 AND user_id = $2
        "#
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let count: i64 = result.get("count");
    Ok(count > 0)
}

