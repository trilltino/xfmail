//! Message Sync Handlers (Braid-HTTP)
//!
//! This module implements Braid protocol handlers for real-time message synchronization.
//! Each conversation is a separate Braid resource.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Response, Sse},
    Json,
};
use bytes::Bytes;
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
// use tokio::sync::broadcast; // not directly referenced, channel used via state
use std::convert::Infallible;

use crate::backend::auth::sessions::verify_token;
use crate::backend::messaging::db::{is_user_participant_in_conversation, get_messages_for_conversation, store_message};
use crate::backend::server::state::MessagingBroadcastState;
use crate::shared::messaging::ChatMessage;
// use crate::shared::messaging::message::VersionVector; // currently unused
// use chrono::Utc; // timestamp handled inline where needed

/// Request to send a new message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: Option<String>,
}

/// Response after sending a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub message_id: Option<Uuid>,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Extract and verify JWT token from headers
fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    // TODO: Re-enable authentication after debugging connection issues
    // For now, bypass JWT for all requests during debugging
    
    // Development bypass: allow requests without Authorization when DEV_AUTH_BYPASS=1
    // Client should send X-Dev-User-Id: <uuid>
    if std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1" {
        if let Some(user_id_val) = headers.get("x-dev-user-id").and_then(|h| h.to_str().ok()) {
            if let Ok(uid) = Uuid::parse_str(user_id_val) {
                tracing::warn!("[DEV_AUTH] Bypassing auth for user_id={}", uid);
                return Ok(uid);
            }
        }
    }
    
    // Temporary bypass for debugging - use a fixed UUID if no auth provided
    // In production, this should require proper authentication
    if let Some(user_id_val) = headers.get("x-dev-user-id").and_then(|h| h.to_str().ok()) {
        if let Ok(uid) = Uuid::parse_str(user_id_val) {
            tracing::debug!("[MessageSync] Using X-Dev-User-Id: {}", uid);
            return Ok(uid);
        }
    }
    
    // Try to get JWT token if provided
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(header_str) = auth_header.to_str() {
            if let Some(token) = header_str.strip_prefix("Bearer ") {
                if let Ok(claims) = verify_token(token) {
                    if let Ok(uid) = Uuid::parse_str(&claims.sub) {
                        tracing::debug!("[MessageSync] Using JWT user: {}", uid);
                        return Ok(uid);
                    }
                }
            }
        }
    }
    
    // Fallback: use a default debugging UUID
    let debug_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Failed to parse debug UUID");
    tracing::warn!("[MessageSync] No auth provided, using debug UUID: {}", debug_uuid);
    Ok(debug_uuid)
}

/// Handle Braid subscription for conversation messages
/// GET /sync/conversations/{conversation_id}/messages
#[cfg(feature = "ssr")]
pub async fn handle_message_subscription(
    State(db_pool): State<Option<PgPool>>,
    State(broadcast_state): State<MessagingBroadcastState>,
    Path(conversation_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Sse<impl StreamExt<Item = Result<axum::response::sse::Event, Infallible>>>, StatusCode> {
    eprintln!("[GET-SUB] Subscription request for conversation: {}", conversation_id);
    tracing::warn!("[MessageSync] Subscription request for conversation: {}", conversation_id);
    
    // Extract user ID first - this shouldn't fail now with bypass enabled
    let user_id = extract_user_id(&headers)?;
    eprintln!("[GET-SUB] User ID: {}", user_id);
    tracing::debug!("[MessageSync] User ID: {}", user_id);
    
    // Accept subscription regardless of explicit Subscribe header (some clients may omit it)
    // If you want to enforce the header, re-enable the check below.
    // if !headers.contains_key("subscribe") {
    //     return Err(StatusCode::BAD_REQUEST);
    // }

    // Log incoming headers for diagnostics
    for (name, value) in headers.iter() {
        if let Ok(val) = value.to_str() {
            tracing::debug!("[MessageSync] Subscribe headers: {}: {}", name.as_str(), val);
        }
    }

    // Load existing messages from database if available
    let messages = if let Some(pool) = db_pool.as_ref() {
        // Verify user is participant in conversation (skip in DEV_AUTH_BYPASS mode)
        let dev_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
        if !dev_bypass {
            tracing::debug!("[MessageSync] Checking if user {} is participant in conversation {}", user_id, conversation_id);
            match is_user_participant_in_conversation(pool, user_id, conversation_id).await {
                Ok(is_participant) => {
                    if !is_participant {
                        tracing::warn!("[MessageSync] User {} is not a participant in conversation {}", user_id, conversation_id);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
                Err(e) => {
                    tracing::error!("[MessageSync] Failed to check participant status: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        } else {
            tracing::debug!("[MessageSync] DEV_AUTH_BYPASS enabled, skipping participant check");
        }

        // Load existing messages from database
        tracing::debug!("[MessageSync] Loading messages for conversation {}", conversation_id);
        match get_messages_for_conversation(pool, conversation_id, 50, 0).await {
            Ok(msgs) => {
                tracing::info!("[MessageSync] Loaded {} messages for conversation {}", msgs.len(), conversation_id);
                msgs
            }
            Err(e) => {
                tracing::error!("[MessageSync] Failed to load messages: {:?}", e);
                Vec::new() // Return empty list and continue
            }
        }
    } else {
        tracing::warn!("[MessageSync] Database pool not available, starting with no initial messages");
        Vec::new()
    };

    // Subscribe to broadcast channel for new messages
    let broadcast_rx = broadcast_state.get_sender(conversation_id).subscribe();
    tracing::debug!("[MessageSync] Subscribed to broadcast channel for conversation {}", conversation_id);

    // Create SSE stream combining initial messages + live updates
    let stream = stream::select(
        // Send initial message history
        stream::iter(messages.into_iter().map(|msg| {
            Ok(axum::response::sse::Event::default()
                .event("message")
                .data(serde_json::to_string(&msg).unwrap()))
        })),

        // Send live broadcast messages
        stream::unfold(broadcast_rx, |mut rx| async move {
            match rx.recv().await {
                Ok(message) => Some((
                    Ok(axum::response::sse::Event::default()
                        .event("message")
                        .data(serde_json::to_string(&message).unwrap())),
                    rx
                )),
                Err(_) => None, // Channel closed
            }
        })
    );

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keep-alive")
    ))
}

/// Handle Braid PUT for sending a message
/// PUT /sync/conversations/{conversation_id}/messages/{message_id}
#[cfg(feature = "ssr")]
pub async fn handle_message_put(
    State(db_pool): State<Option<PgPool>>,
    State(broadcast_state): State<MessagingBroadcastState>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(request): Json<SendMessageRequest>,
) -> Result<Response<Body>, StatusCode> {
    eprintln!("[PUT-MSG] PUT request: msg={}, conv={}", message_id, conversation_id);
    
    let pool = db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = extract_user_id(&headers)?;

    eprintln!("[PUT-MSG] From user: {}, content length: {}", user_id, request.content.len());
    tracing::warn!(
        "[BRAID] PUT message {} in conversation {} from user {}",
        message_id,
        conversation_id,
        user_id
    );

    // Verify user is participant in conversation (skip in DEV_AUTH_BYPASS mode)
    let dev_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
    if !dev_bypass {
        let is_participant = is_user_participant_in_conversation(pool, user_id, conversation_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !is_participant {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Extract Braid headers
    let version_header = headers.get("version")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("initial");

    let parents_header = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let parents: Vec<String> = if parents_header.is_empty() {
        Vec::new()
    } else {
        // Parse structured headers format: "version1", "version2"
        parents_header.split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect()
    };

    tracing::info!(
        "[BRAID] Message version: {}, parents: {:?}",
        version_header, parents
    );

    // Create the message with CRDT metadata
    let message = ChatMessage {
        id: message_id,
        conversation_id,
        sender_id: user_id,
        content: request.content,
        message_type: crate::shared::messaging::MessageType::Text,
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_read: false,
        is_delivered: true,
        crdt_timestamp: 0, // TODO: Implement proper CRDT timestamp from version vector
        braid_version: version_header.to_string(),
        braid_parents: parents,
        version_vector: crate::shared::messaging::message::VersionVector::default(), // TODO: Parse from headers
    };

    // Store message in database
    store_message(pool, &message).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("[BRAID] Message stored in database: {}", message_id);

    // Broadcast to other subscribers
    broadcast_state.broadcast(conversation_id, message.clone());

    tracing::info!("[BRAID] Message broadcast to {} subscribers", broadcast_state.get_subscriber_count(conversation_id));

    // Return success with version
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Version", format!("\"{}\"", version_header))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&SendMessageResponse {
                success: true,
                message_id: Some(message_id),
                version: Some(version_header.to_string()),
                error: None,
            })
            .unwrap_or_default(),
        ))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

/// Format messages as Braid update
fn format_braid_message_update(
    messages: &[ChatMessage],
    version: Option<&str>,
) -> Result<Bytes, StatusCode> {
    let update = serde_json::json!({
        "version": version.unwrap_or("initial"),
        "messages": messages,
    });

    serde_json::to_vec(&update)
        .map(Bytes::from)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

