/**
 * Braid PUT Handler
 * 
 * This module implements the Braid PUT protocol handler for adding new messages
 * via PUT /chat requests.
 * 
 * # Braid Protocol (draft-toomim-httpbis-braid-http-04)
 * 
 * The Braid protocol allows clients to send updates using PUT requests
 * with versioning information (Section 2.2):
 * - `Parents:` header specifies the parent version(s) for the DAG
 *   - Format: Structured Headers (RFC 8941) - comma-separated JSON-stringified strings
 *   - Example: `"parent1"` or `"parent1", "parent2"`
 * - `Version:` header is optional (server will generate if not provided)
 * - Response includes `Version:` header with the assigned version ID
 *   - Format: Structured Headers - JSON-stringified string: `"version-id"`
 * 
 * Version and Parents headers use Structured Headers format (RFC 8941):
 * - Single value: `"value"` (JSON-stringified string)
 * - Multiple values: `"value1", "value2"` (comma-separated)
 * 
 * Reference: https://github.com/braid-org/braid-spec/blob/master/draft-toomim-httpbis-braid-http-04.txt
 */

use crate::shared::Message;
use crate::backend::server::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::Response,
};

/// Handle Braid PUT request (PUT /chat with Version/Parents headers)
/// 
/// This handler implements the Braid PUT protocol per
/// draft-toomim-httpbis-braid-http-04 specification:
/// - Accepts new messages with optional Version and Parents headers
/// - Creates new version ID if not provided
/// - Updates chat state with new message
/// - Returns success response with Version header (Structured Headers format)
/// 
/// # Authentication
/// 
/// This endpoint requires authentication via JWT token in the `Authorization`
/// header. The token is verified and the user ID is extracted for:
/// - Message ownership tracking
/// - Database persistence
/// 
/// # Message Validation
/// 
/// The handler validates:
/// - Message text is not empty
/// - Message text length is within limits (10,000 characters)
/// - Author name is not empty
/// - Author name length is within limits (100 characters)
/// - Parent version IDs are valid (if provided)
/// 
/// # Arguments
/// 
/// * `State(app_state)` - Application state containing chat state and broadcast sender
/// * `headers` - Request headers (to check for Version and Parents)
/// * `body` - Request body containing the message JSON
/// 
/// # Returns
/// 
/// HTTP response with status code and Version header (Structured Headers format), or an error status code
/// 
/// # Errors
/// 
/// * `400 Bad Request` - If the request body cannot be parsed as a Message or validation fails
/// * `401 Unauthorized` - If authentication token is missing or invalid
/// * `500 Internal Server Error` - If state update fails
/// 
/// # Example Request
/// 
/// ```http
/// PUT /chat HTTP/1.1
/// Authorization: Bearer <token>
/// Content-Type: application/json
/// Parents: "abc123"
/// 
/// {"text":"Hello, world!","author":"Alice","timestamp":"2024-01-01T00:00:00Z"}
/// ```
/// 
/// # Example Response
/// 
/// ```http
/// HTTP/1.1 200 OK
/// Version: "xyz789"
/// ```
/// 
/// Reference: Section 2.2 of draft-toomim-httpbis-braid-http-04.txt
#[cfg(feature = "ssr")]
pub async fn handle_braid_put(
    State(app_state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    // Require authentication for PUT requests
    use crate::backend::auth::sessions::verify_token;
    use axum::http::header::AUTHORIZATION;
    
    // Get Authorization header
    let auth_header = headers.get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("PUT /chat requires authentication");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Extract token (format: "Bearer <token>")
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| {
            tracing::warn!("Invalid Authorization header format");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Verify token
    let claims = verify_token(token)
        .map_err(|e| {
            tracing::warn!("Invalid token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Parse user ID from claims (we'll use this for message ownership)
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|e| {
            tracing::error!("Invalid user ID in token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    tracing::info!("[Server] Authenticated PUT request from user: {}", user_id);
    
    // Parse message from request body
    // The body should be a JSON-serialized Message
    let message: Message = serde_json::from_slice(&body)
        .map_err(|e| {
            tracing::error!("Failed to parse message from request body: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Validate message content
    // Check that text is not empty and not too long
    if message.text.trim().is_empty() {
        tracing::warn!("[Server] Rejected message with empty text from: {}", message.author);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Limit message length to prevent abuse (e.g., 10000 characters)
    const MAX_MESSAGE_LENGTH: usize = 10000;
    if message.text.len() > MAX_MESSAGE_LENGTH {
        tracing::warn!("[Server] Rejected message that is too long ({} chars) from: {}", message.text.len(), message.author);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate author name
    if message.author.trim().is_empty() {
        tracing::warn!("[Server] Rejected message with empty author");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Limit author name length
    const MAX_AUTHOR_LENGTH: usize = 100;
    if message.author.len() > MAX_AUTHOR_LENGTH {
        tracing::warn!("[Server] Rejected message with author name too long ({} chars)", message.author.len());
        return Err(StatusCode::BAD_REQUEST);
    }
    
    tracing::info!("[Server] Received PUT request with message from: {}", message.author);
    
    // Get Parents header for version DAG
    // Parents header uses Structured Headers format (RFC 8941) - comma-separated
    // JSON-stringified strings, e.g., "parent1", "parent2"
    // Reference: Section 2 of draft-toomim-httpbis-braid-http-04.txt
    let parents = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Parse Structured Headers format: comma-separated JSON-stringified strings
            // Example: "parent1", "parent2" or "parent1"
            s.split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    // Remove JSON string quotes if present (from Structured Headers format)
                    // Structured Headers use JSON-stringified strings: "value" or "value1", "value2"
                    if trimmed.starts_with('"') && trimmed.ends_with('"') {
                        trimmed[1..trimmed.len()-1].to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|v| !v.is_empty());
    
    // Validate version IDs in Parents header
    // Version IDs should be valid UUIDs (or at least reasonable length)
    if let Some(ref p) = parents {
        const MAX_VERSION_ID_LENGTH: usize = 200;
        for parent_version in p.iter() {
            if parent_version.len() > MAX_VERSION_ID_LENGTH {
                tracing::warn!("[Server] Rejected message with invalid parent version ID (too long): {}", parent_version);
                return Err(StatusCode::BAD_REQUEST);
            }
            // Additional validation: version IDs should not contain invalid characters
            if parent_version.contains('\n') || parent_version.contains('\r') {
                tracing::warn!("[Server] Rejected message with invalid parent version ID (contains newline): {}", parent_version);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        tracing::info!("[Server] Parents header: {:?}", p);
    }
    
    // Add message to state
    // This will generate a new version ID and update the version history
    let (version_id, parent_versions) = {
        let mut state_write = app_state.chat_state.write().await;
        let v_id = state_write.add_message(message.clone(), parents.clone());
        // Get the parent versions that were actually set in state
        let parents = state_write.version_history.get(&v_id)
            .cloned()
            .unwrap_or_default();
        (v_id, parents)
    };
    
    // Save message and version history to database (if available)
    if let Some(pool) = &app_state.db_pool {
        use crate::backend::chat::db::{save_message, save_version_history};
        
        // Save message to database
        if let Err(e) = save_message(pool, user_id, &message, &version_id).await {
            tracing::error!("[Server] Failed to save message to database: {:?}", e);
            // Don't fail the request if database save fails
        }
        
        // Save version history to database
        if let Err(e) = save_version_history(pool, user_id, &version_id, &parent_versions).await {
            tracing::error!("[Server] Failed to save version history to database: {:?}", e);
            // Don't fail the request if database save fails
        }
    }
    
    tracing::info!("[Server] New message added with version: {}", version_id);
    
    // Get all current messages to broadcast
    // The client replaces the entire message list, so we need to send all messages
    let messages_to_broadcast = {
        let state_read = app_state.chat_state.read().await;
        state_read.messages.clone()
    };
    
    tracing::info!("[Server] Broadcasting {} messages to all subscribers with version: {}", messages_to_broadcast.len(), version_id);
    
    // Broadcast all messages to all SSE subscribers
    // This immediately notifies all connected clients of the updated state
    let broadcast_result = app_state.message_broadcast.send((messages_to_broadcast, version_id.clone()));
    
    match broadcast_result {
        Ok(subscriber_count) => {
            tracing::info!("[Server] Successfully broadcast to {} subscribers", subscriber_count);
        }
        Err(e) => {
            // No subscribers yet, that's okay
            tracing::debug!("[Server] No subscribers to receive broadcast: {:?}", e);
        }
    }
    
    // Return success response with Version header
    // Version header uses Structured Headers format (RFC 8941) - JSON-stringified string
    // Reference: draft-toomim-httpbis-braid-http-04.txt Section 2.2 (PUT a new version)
    // Reference: braid-http-server.js lines 678-680 (Structured Headers format)
    // Format: "version-id" (single version) or "version1", "version2" (multiple versions)
    let version_header_value = format!("\"{}\"", version_id);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Version", version_header_value)
        .body(Body::empty())
        .map_err(|e| {
            tracing::error!("[Server] Failed to build response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};
    use crate::backend::server::state::AppState;
    use crate::backend::auth::sessions::create_token;
    use crate::backend::auth::users::create_user;
    use crate::shared::Message;
    use tests::common::database::TestDatabase;
    use bcrypt;

    fn create_test_app_state(pool: Option<sqlx::PgPool>) -> AppState {
        use leptos::get_configuration;
        let conf = get_configuration(Some("Cargo.toml")).unwrap();
        AppState {
            leptos_options: conf.leptos_options,
            db_pool: pool,
            chat_state: std::sync::Arc::new(tokio::sync::RwLock::new(crate::backend::chat::state::ChatState::new())),
            realtime_broadcast: tokio::sync::broadcast::channel(1000).0,
            message_broadcast: tokio::sync::broadcast::channel(1000).0,
        }
    }

    #[tokio::test]
    async fn test_put_message_success() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        let token = create_token(user.id, user.email.clone()).unwrap();
        
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", token).parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        
        let message = Message::new("Hello".to_string(), "test@example.com".to_string());
        let body = Body::from(serde_json::to_string(&message).unwrap());
        
        let app_state = create_test_app_state(Some(pool.clone()));
        let result = handle_braid_put(State(app_state), headers, body).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_put_message_no_auth() {
        let app_state = create_test_app_state(None);
        let headers = HeaderMap::new();
        let message = Message::new("Hello".to_string(), "test".to_string());
        let body = Body::from(serde_json::to_string(&message).unwrap());
        
        let result = handle_braid_put(State(app_state), headers, body).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_put_message_empty_text() {
        let db = TestDatabase::new().await;
        let pool = db.pool();
        
        let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
        let user = create_user(pool, "test@example.com".to_string(), password_hash).await.unwrap();
        let token = create_token(user.id, user.email.clone()).unwrap();
        
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", token).parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        
        let mut message = Message::new("".to_string(), "test@example.com".to_string());
        let body = Body::from(serde_json::to_string(&message).unwrap());
        
        let app_state = create_test_app_state(Some(pool.clone()));
        let result = handle_braid_put(State(app_state), headers, body).await;
        
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }
}

