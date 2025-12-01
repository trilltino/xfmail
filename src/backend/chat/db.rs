/**
 * Database Operations for Chat Messages and Version History
 * 
 * This module provides database operations for persisting chat messages
 * and version history to PostgreSQL.
 */

use crate::shared::Message;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Save a message to the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID who created the message
/// * `message` - Message to save
/// * `version_id` - Braid version ID
///
/// # Returns
/// Result indicating success or failure
#[cfg(feature = "ssr")]
pub async fn save_message(
    pool: &PgPool,
    user_id: Uuid,
    message: &Message,
    version_id: &str,
) -> Result<(), sqlx::Error> {
    // Parse timestamp from RFC3339 string to PostgreSQL TIMESTAMPTZ
    let timestamp = message.timestamp.parse::<DateTime<Utc>>()
        .map_err(|e| sqlx::Error::Decode(format!("Failed to parse timestamp: {}", e).into()))?;

    sqlx::query(
        r#"
        INSERT INTO messages (id, user_id, text, author, timestamp, version, created_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())
        ON CONFLICT (version) DO UPDATE SET
            text = EXCLUDED.text,
            author = EXCLUDED.author,
            timestamp = EXCLUDED.timestamp
        "#
    )
    .bind(user_id)
    .bind(&message.text)
    .bind(&message.author)
    .bind(timestamp)
    .bind(version_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Save version history to the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `version_id` - Braid version ID
/// * `parent_versions` - Parent version IDs (for Braid DAG)
///
/// # Returns
/// Result indicating success or failure
#[cfg(feature = "ssr")]
pub async fn save_version_history(
    pool: &PgPool,
    user_id: Uuid,
    version_id: &str,
    parent_versions: &[String],
) -> Result<(), sqlx::Error> {
    // Convert Vec<String> to PostgreSQL TEXT[] array
    sqlx::query(
        r#"
        INSERT INTO version_history (id, user_id, version_id, parent_versions, created_at)
        VALUES (gen_random_uuid(), $1, $2, $3, NOW())
        ON CONFLICT (user_id, version_id) DO UPDATE SET
            parent_versions = EXCLUDED.parent_versions
        "#
    )
    .bind(user_id)
    .bind(version_id)
    .bind(parent_versions)
    .execute(pool)
    .await?;

    Ok(())
}

/// Load all messages from the database
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// Vector of messages ordered by created_at, or error
#[cfg(feature = "ssr")]
pub async fn load_messages(pool: &PgPool) -> Result<Vec<Message>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct MessageRow {
        text: String,
        author: String,
        timestamp: DateTime<Utc>,
        version: Option<String>,
    }
    
    let rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT text, author, timestamp, version
        FROM messages
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let messages: Vec<Message> = rows
        .into_iter()
        .map(|row| Message {
            text: row.text,
            author: row.author,
            timestamp: row.timestamp.to_rfc3339(),
            version: row.version,
        })
        .collect();
    
    Ok(messages)
}

/// Load version history from the database
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// HashMap mapping version_id to parent_versions, or error
#[cfg(feature = "ssr")]
pub async fn load_version_history(
    pool: &PgPool,
) -> Result<HashMap<String, Vec<String>>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct VersionHistoryRow {
        version_id: String,
        parent_versions: Vec<String>,
    }
    
    let rows = sqlx::query_as::<_, VersionHistoryRow>(
        r#"
        SELECT version_id, parent_versions
        FROM version_history
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let mut version_history = HashMap::new();
    for row in rows {
        version_history.insert(row.version_id, row.parent_versions);
    }
    
    Ok(version_history)
}

/// Get or create a system bot user ID
/// 
/// This function creates a system user for bot messages if it doesn't exist.
/// The system user has email "system@bot" and a dummy password hash.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// System bot user ID, or error
#[cfg(feature = "ssr")]
pub async fn get_or_create_bot_user_id(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    use crate::backend::auth::users::get_user_by_email;

    // Try to get existing system bot user
    if let Ok(Some(user)) = get_user_by_email(pool, "system@bot").await {
        return Ok(user.id);
    }

    // Create system bot user if it doesn't exist
    // Use a dummy password hash (bot users don't need to login)
    let dummy_password_hash = "$2b$12$dummyhashforbotuserdonotuseinproduction";

    let user_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    // Try to insert the user
    #[derive(sqlx::FromRow)]
    struct IdRow {
        id: Uuid,
    }

    match sqlx::query_as::<_, IdRow>(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET id = users.id
        RETURNING id
        "#
    )
    .bind(user_id)
    .bind("system_bot")
    .bind("system@bot")
    .bind(dummy_password_hash)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    {
        Ok(row) => Ok(row.id),
        Err(_e) => {
            // If insert failed, try to get the existing user
            match get_user_by_email(pool, "system@bot").await {
                Ok(Some(user)) => Ok(user.id),
                Ok(None) => Err(sqlx::Error::RowNotFound),
                Err(err) => Err(err),
            }
        }
    }
}

/// Get or create a system user ID for global chat
///
/// This function creates a system user for global chat messages if it doesn't exist.
/// Used for messages that aren't associated with a specific authenticated user.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// System user ID, or error
#[cfg(feature = "ssr")]
pub async fn get_or_create_system_user_id(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    use crate::backend::auth::users::get_user_by_email;

    // Try to get existing system user
    if let Ok(Some(user)) = get_user_by_email(pool, "system@global").await {
        return Ok(user.id);
    }

    // Create system user if it doesn't exist
    let dummy_password_hash = "$2b$12$dummyhashforsystemuserdonotuseinproduction";

    let user_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    // Try to insert the user
    #[derive(sqlx::FromRow)]
    struct IdRow {
        id: Uuid,
    }

    match sqlx::query_as::<_, IdRow>(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET id = users.id
        RETURNING id
        "#
    )
    .bind(user_id)
    .bind("system_global")
    .bind("system@global")
    .bind(dummy_password_hash)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    {
        Ok(row) => Ok(row.id),
        Err(_e) => {
            // If insert failed, try to get the existing user
            match get_user_by_email(pool, "system@global").await {
                Ok(Some(user)) => Ok(user.id),
                Ok(None) => Err(sqlx::Error::RowNotFound),
                Err(err) => Err(err),
            }
        }
    }
}

