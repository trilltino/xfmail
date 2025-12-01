//! Messaging HTTP Handlers
//!
//! This module contains the HTTP handlers for friend requests and messaging.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::backend::auth::sessions::verify_token;
use crate::shared::messaging::{
    SendFriendRequestRequest, SendFriendRequestResponse,
    RespondFriendRequestRequest, RespondFriendRequestResponse, ListFriendRequestsResponse,
    ListContactsResponse,
};
use super::db;

/// Extract and verify JWT token from headers
fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

/// Send a friend request
pub async fn send_friend_request(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
    Json(request): Json<SendFriendRequestRequest>,
) -> Result<Json<SendFriendRequestResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let from_user_id = extract_user_id(&headers)?;

    // Get the sender's info
    let from_user = crate::backend::auth::users::get_user_by_id(pool, from_user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Find the recipient by email
    let to_user = crate::backend::auth::users::get_user_by_email(pool, &request.to_email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let to_user = match to_user {
        Some(user) => user,
        None => {
            return Ok(Json(SendFriendRequestResponse {
                success: false,
                request_id: None,
                error: Some("User not found".to_string()),
            }));
        }
    };

    // Check if already friends or request pending
    let existing_contact = db::get_contacts_for_user(pool, from_user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|c| c.email == request.to_email);

    if existing_contact.is_some() {
        return Ok(Json(SendFriendRequestResponse {
            success: false,
            request_id: None,
            error: Some("Already friends".to_string()),
        }));
    }

    // Check for pending requests in both directions
    let pending_requests = db::get_pending_friend_requests(pool, from_user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let has_pending_request = pending_requests.iter().any(|r| r.to_email == request.to_email);

    if has_pending_request {
        return Ok(Json(SendFriendRequestResponse {
            success: false,
            request_id: None,
            error: Some("Friend request already pending".to_string()),
        }));
    }

    // Create the friend request
    let friend_request = db::create_friend_request(
        pool,
        from_user_id,
        to_user.id,
        &from_user.username,
        &from_user.email,
        &to_user.email,
        None,
    )
    .await
    .map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("duplicate key") || error_msg.contains("unique") {
            tracing::warn!("Friend request already exists or duplicate: {:?}", e);
            StatusCode::CONFLICT
        } else {
            tracing::error!("Failed to create friend request: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(SendFriendRequestResponse {
        success: true,
        request_id: Some(friend_request.id),
        error: None,
    }))
}

/// Get pending friend requests for the current user
pub async fn get_friend_requests(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
) -> Result<Json<ListFriendRequestsResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    let requests = db::get_pending_friend_requests(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get friend requests: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ListFriendRequestsResponse { requests }))
}

/// Respond to a friend request (accept or reject)
pub async fn respond_to_friend_request(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
    Json(request): Json<RespondFriendRequestRequest>,
) -> Result<Json<RespondFriendRequestResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    if request.accept {
        // Get the friend request BEFORE accepting (so we can retrieve data while status is still 'pending')
        let friend_request = db::get_friend_request_by_id(pool, request.request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Only the recipient can accept
        if friend_request.to_user_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }

        // Now mark it as accepted
        db::accept_friend_request(pool, request.request_id, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to accept friend request: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Get the sender's user info
        let sender = crate::backend::auth::users::get_user_by_id(pool, friend_request.from_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Get the recipient's user info
        let recipient = crate::backend::auth::users::get_user_by_id(pool, user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Create contact for recipient (pointing to sender)
        db::create_contact(
            pool,
            user_id,
            sender.id,
            &sender.username,
            &sender.email,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create contact for recipient: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Create contact for sender (pointing to recipient)
        db::create_contact(
            pool,
            sender.id,
            user_id,
            &recipient.username,
            &recipient.email,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create contact for sender: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Create a conversation between them
        db::create_conversation(pool, user_id, sender.id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create conversation: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        db::reject_friend_request(pool, request.request_id, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to reject friend request: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(Json(RespondFriendRequestResponse {
        success: true,
        error: None,
    }))
}

/// Get contacts for the current user
pub async fn get_contacts(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
) -> Result<Json<ListContactsResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    let contacts = db::get_contacts_for_user(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get contacts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ListContactsResponse { contacts }))
}

/// Get conversations for the current user
pub async fn get_conversations(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
) -> Result<Json<crate::shared::messaging::ListConversationsResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    let conversations = db::get_conversations_for_user(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get conversations: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(crate::shared::messaging::ListConversationsResponse { conversations }))
}

/// Get messages for a conversation
pub async fn get_messages(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
    axum::extract::Path(conversation_id): axum::extract::Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<ListMessagesParams>,
) -> Result<Json<crate::shared::messaging::ListMessagesResponse>, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    // Verify user is participant in conversation
    let is_participant = db::is_user_participant_in_conversation(pool, user_id, conversation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_participant {
        return Err(StatusCode::FORBIDDEN);
    }

    let limit = params.limit.unwrap_or(50) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    let messages = db::get_messages_for_conversation(pool, conversation_id, limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get messages: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let has_more = messages.len() as i64 == limit;
    Ok(Json(crate::shared::messaging::ListMessagesResponse {
        messages,
        has_more,
    }))
}

/// Query parameters for listing messages
#[derive(Debug, serde::Deserialize)]
pub struct ListMessagesParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Mark a message as read
pub async fn mark_message_read(
    State(db_pool): State<Option<PgPool>>,
    headers: HeaderMap,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let _user_id = extract_user_id(&headers)?;

    db::mark_message_read(pool, message_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark message as read: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

