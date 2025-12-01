/**
 * Braid Subscription Handler
 * 
 * This module implements the Braid subscription protocol handler for
 * GET /chat requests with the Subscribe header.
 * 
 * # Braid Protocol (draft-toomim-httpbis-braid-http-04)
 * 
 * The Braid protocol allows clients to subscribe to resource updates
 * by sending a GET request with a `Subscribe:` header. The server
 * MUST include a Subscribe header in its response (Section 4.1).
 * The server responds with a stream of updates in pure Braid format:
 * ```
 * Version: "version-id"
 * Content-Length: <size>
 * 
 * <json-body>
 * ```
 * 
 * Version and Parents headers use Structured Headers format (RFC 8941):
 * - Single version: `"version-id"` (JSON-stringified string)
 * - Multiple versions: `"version1", "version2"` (comma-separated)
 * 
 * For reconnection, clients can send a `Parents:` header with their
 * last known version (Section 4.3), and the server will only send
 * updates since that version.
 * 
 * All line endings use CRLF (\r\n) per HTTP specification.
 * 
 * Reference: https://github.com/braid-org/braid-spec/blob/master/draft-toomim-httpbis-braid-http-04.txt
 */

use crate::shared::Message;
#[cfg(feature = "ssr")]
use crate::backend::server::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::Response,
};
use bytes::Bytes;
use futures_util::stream;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper function to format a Braid update as bytes
/// 
/// Formats an update in pure Braid format per draft-toomim-httpbis-braid-http-04 spec:
/// ```
/// Version: <version-id>
/// Content-Length: <size>
/// 
/// <json-body>
/// ```
/// 
/// Version header uses Structured Headers format (RFC 8941) - comma-separated
/// JSON-stringified strings. For a single version, it's formatted as: "version-id"
/// 
/// All line endings use CRLF (\r\n) as per HTTP spec.
/// 
/// For subscriptions, blank lines should separate updates (spec section 4.2):
/// - After each update body, add blank lines (\r\n\r\n) to separate from next update
/// - This helps keep connections alive and ensures intermediaries flush data
/// 
/// # Arguments
/// 
/// * `version` - Optional version ID to include in Version header
/// * `messages` - Messages to include in the update body
/// * `add_trailing_newlines` - Whether to add trailing newlines for subscription streams
/// 
/// # Returns
/// 
/// Formatted Braid update as bytes, or an error status code
/// 
/// Reference: Section 4.2 of draft-toomim-httpbis-braid-http-04.txt
/// Reference: braid-http-server.js sendUpdate() (lines 703-714)
pub(crate) fn format_braid_update(
    version: Option<&String>,
    messages: &[Message],
    add_trailing_newlines: bool,
) -> Result<Bytes, StatusCode> {
    // Serialize messages to JSON
    let json_body = serde_json::to_string(messages)
        .map_err(|e| {
            tracing::error!("[Server] Failed to serialize messages to JSON: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let body_bytes = json_body.as_bytes();
    let content_length = body_bytes.len();
    
    // Build headers using CRLF line endings (\r\n) per spec
    // Structured Headers format (RFC 8941): Version values are JSON-stringified strings
    let mut header_lines = Vec::new();
    
    if let Some(ver) = version {
        // Format Version header in Structured Headers format (RFC 8941)
        // Single version ID: JSON-stringified (e.g., "version-id")
        // Multiple version IDs would be: "version1", "version2"
        // Reference: braid-http-server.js lines 678-680
        let version_header = format!("\"{}\"", ver); // JSON-stringify the version ID
        header_lines.push(format!("Version: {}\r\n", version_header));
    }
    
    header_lines.push(format!("Content-Length: {}\r\n", content_length));
    header_lines.push("\r\n".to_string()); // Empty line after headers (CRLF) - separates headers from body
    
    let headers_str = header_lines.join("");
    let headers_bytes = headers_str.as_bytes();
    
    // Calculate trailing newlines size if needed
    let trailing_newlines_size = if add_trailing_newlines { 4 } else { 0 }; // \r\n\r\n = 4 bytes for separation
    
    // Combine headers, body, and trailing newlines
    let mut result = Vec::with_capacity(headers_bytes.len() + body_bytes.len() + trailing_newlines_size);
    result.extend_from_slice(headers_bytes);
    result.extend_from_slice(body_bytes);
    
    // Add trailing blank lines to separate updates (per spec section 4.2)
    // Reference: braid-http-server.js lines 705-713
    // Spec allows one or more blank lines; we use \r\n\r\n (one blank line) to match reference
    if add_trailing_newlines {
        result.extend_from_slice(b"\r\n\r\n"); // Blank line after body to separate from next update
    }
    
    Ok(Bytes::from(result))
}

/// Handle Braid subscription request (GET /chat with Subscribe header)
/// 
/// This handler implements the Braid subscription protocol per
/// draft-toomim-httpbis-braid-http-04 specification:
/// - Returns pure Braid stream (209 Subscription status)
/// - Streams initial snapshot of messages
/// - Continues streaming updates as new messages arrive
/// - Supports reconnection with Parents header for catch-up
/// - Sends keep-alive heartbeats every 30 seconds (CRLF format)
/// 
/// # Arguments
/// 
/// * `State(app_state)` - Application state containing chat state and broadcast sender
/// * `headers` - Request headers (to check for Subscribe and Parents)
/// 
/// # Returns
/// 
/// Pure Braid stream (209 Subscription) with Braid protocol updates, or an error status code
/// 
/// # Errors
/// 
/// * `400 Bad Request` - If the Subscribe header is missing
/// * `500 Internal Server Error` - If JSON serialization fails
/// 
/// # Example Request
/// 
/// ```http
/// GET /chat HTTP/1.1
/// Subscribe:
/// Parents: "abc123"
/// ```
/// 
/// # Example Response
/// 
/// ```http
/// HTTP/1.1 209 Subscription
/// Subscribe:
/// Content-Type: application/json
/// Cache-Control: no-cache, no-transform, no-store
/// Connection: keep-alive
/// X-Accel-Buffering: no
/// 
/// Version: "xyz789"
/// Content-Length: 123
/// 
/// {"messages":[...]}
/// ```
/// 
/// Reference: Section 4 of draft-toomim-httpbis-braid-http-04.txt
#[cfg(feature = "ssr")]
pub async fn handle_braid_subscription(
    State(app_state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    tracing::info!("[Server] Braid subscription request received");
    tracing::info!("[Server] Request headers: {:?}", headers.keys().collect::<Vec<_>>());
    
    // Check if Subscribe header is present
    // The Braid protocol requires this header for subscriptions
    if !headers.contains_key("subscribe") {
        tracing::warn!("[Server] Subscribe header missing, returning BAD_REQUEST");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    tracing::info!("[Server] Subscribe header found");
    
    // Get Parents header for reconnection catch-up
    // Parents header uses Structured Headers format (RFC 8941) - comma-separated
    // JSON-stringified strings, e.g., "parent1", "parent2" or "parent1"
    // Reference: draft-toomim-httpbis-braid-http-04.txt Section 4.3 (Continuing a Subscription)
    // Reference: draft-toomim-httpbis-braid-http-04.txt Section 2.5 (Rules for Version and Parents headers)
    // For reconnection, we use the first/latest parent version as the catch-up point
    let parents_header = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Parse Structured Headers format: comma-separated JSON-stringified strings
            // Example: "parent1", "parent2" or "parent1"
            // For reconnection, we typically use the latest (first) parent version
            let parent_versions: Vec<String> = s.split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    // Remove JSON string quotes if present (from Structured Headers format)
                    if trimmed.starts_with('"') && trimmed.ends_with('"') {
                        trimmed[1..trimmed.len()-1].to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
            
            // Use the first parent version for reconnection catch-up
            // This represents the client's last known version
            parent_versions.first().cloned()
        })
        .flatten();
    
    if let Some(ref parents) = parents_header {
        tracing::info!("[Server] Parents header found: {}", parents);
    } else {
        tracing::info!("[Server] No Parents header (new subscription)");
    }
    
    // Get initial messages (either all or since parents version)
    let initial_messages = {
        let state_read = app_state.chat_state.read().await;
        state_read.get_messages_since(parents_header.as_ref())
    };
    
    tracing::info!("[Server] Initial messages count: {}", initial_messages.len());
    
    // Get current version for the initial snapshot
    let initial_version = {
        let state_read = app_state.chat_state.read().await;
        state_read.current_version.clone()
    };
    
    tracing::info!("[Server] Current version: {:?}", initial_version);
    
    // Subscribe to broadcast channel for real-time updates
    let broadcast_rx = app_state.message_broadcast.subscribe();
    
    tracing::info!("[Server] Subscribed to broadcast channel, creating pure Braid stream");
    
    // Convert initial_version to String for comparison
    let last_version_str = initial_version.as_ref().map(|s| s.clone()).unwrap_or_else(|| String::new());
    
    // Create a channel for sending updates and managing connection state
    // Use std::io::Error as the error type since it implements Into<BoxError>
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Bytes, std::io::Error>>();
    let connected = Arc::new(RwLock::new(true));
    let connected_clone = connected.clone();
    
    // Clone tx for heartbeat task before moving it into the first spawn
    let tx_heartbeat = tx.clone();
    
    // Spawn task to handle the stream
    let mut broadcast_rx_clone = broadcast_rx;
    let last_version_clone = last_version_str.clone();
    let initial_messages_clone = initial_messages.clone();
    let initial_version_clone = initial_version.clone();
    
    tokio::spawn(async move {
        // Send initial snapshot
        // add_trailing_newlines=true because this is part of a subscription stream
        let initial_update = format_braid_update(initial_version_clone.as_ref(), &initial_messages_clone, true);
        match initial_update {
            Ok(bytes) => {
                tracing::info!("[Server] Sending initial snapshot");
                if tx.send(Ok(bytes)).is_err() {
                    tracing::warn!("[Server] Failed to send initial snapshot (receiver dropped)");
                    return;
                }
            }
            Err(e) => {
                tracing::error!("[Server] Failed to format initial snapshot: {:?}", e);
                let io_err = std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to format update: {:?}", e));
                let _ = tx.send(Err(io_err));
                return;
            }
        }
        
        // Listen to broadcast channel for new messages
        let mut last_version = last_version_clone;
        loop {
            // Check if still connected
            {
                let connected_guard = connected_clone.read().await;
                if !*connected_guard {
                    tracing::info!("[Server] Connection closed, stopping stream");
                    break;
                }
            }
            
            match broadcast_rx_clone.recv().await {
                Ok((new_messages, new_version)) => {
                    // Only send if this version is different and we have messages
                    if new_version != last_version && !new_messages.is_empty() {
                        tracing::info!("[Server] Received broadcast: {} new messages with version: {}", new_messages.len(), new_version);
                        
                        // add_trailing_newlines=true because this is part of a subscription stream
                        let update = format_braid_update(Some(&new_version), &new_messages, true);
                        match update {
                            Ok(bytes) => {
                                if tx.send(Ok(bytes)).is_err() {
                                    tracing::warn!("[Server] Failed to send update (receiver dropped)");
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("[Server] Failed to format update: {:?}", e);
                                // Continue on error - don't send error to client, just log it
                            }
                        }
                        last_version = new_version;
                    } else {
                        tracing::debug!("[Server] Broadcast received but version unchanged or empty, continuing to listen");
                        last_version = new_version;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("[Server] Broadcast receiver lagged, skipped {} messages. Client may need to reconnect.", skipped);
                    // Continue looping - we'll catch up on next message
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::warn!("[Server] Broadcast channel closed, ending stream");
                    break;
                }
            }
        }
    });
    
    // Spawn keep-alive heartbeat task
    // Heartbeats use CRLF (\r\n) per HTTP spec and Braid protocol
    // Blank lines help keep connections alive and signal to intermediaries
    // Reference: draft-toomim-httpbis-braid-http-04.txt Section 4.2 (Sending multiple updates per GET)
    // Reference: braid-http-server.js lines 567-583 (heartbeat implementation)
    let connected_heartbeat = connected.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            // Check if still connected
            {
                let connected_guard = connected_heartbeat.read().await;
                if !*connected_guard {
                    break;
                }
            }
            
            // Send heartbeat using CRLF (\r\n) per spec
            // Blank lines help keep connections alive and signal to intermediaries
            if tx_heartbeat.send(Ok(Bytes::from("\r\n"))).is_err() {
                break;
            }
        }
    });
    
    // Create stream from receiver
    let body_stream = stream::unfold(rx, |mut receiver| async move {
        match receiver.recv().await {
            Some(item) => Some((item, receiver)),
            None => None,
        }
    });
    
    // Create custom status code 209 Subscription
    // Note: Axum doesn't have 209 built-in, so we'll use from_u16
    let status = StatusCode::from_u16(209)
        .unwrap_or_else(|_| StatusCode::OK); // Fallback to 200 if 209 not supported
    
    tracing::info!("[Server] Creating pure Braid stream response with status {}", status);
    
    // Create response with streaming body
    let body = Body::from_stream(body_stream);
    
    // Handle connection close to update connected state
    // Note: We can't easily detect when the client disconnects in Axum,
    // but the stream will naturally end when the receiver is dropped
    let _connected_for_cleanup = connected;
    
    // Get Subscribe header value from request (or default to empty string)
    // Per spec section 4.1: "A server implementing Subscribe MUST include a Subscribe header in its response"
    // Reference: draft-toomim-httpbis-braid-http-04.txt Section 4.1, line 783
    let subscribe_value = headers.get("subscribe")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    Ok(Response::builder()
        .status(status)
        // MUST include Subscribe header in response per spec section 4.1
        // Reference: draft-toomim-httpbis-braid-http-04.txt Section 4.1, line 783
        // The blank line after Subscribe header is automatically handled by HTTP
        .header("Subscribe", subscribe_value)
        // Cache-Control per braid-http reference implementation and spec requirements
        // Reference: braid-http-server.js line 525
        .header(axum::http::header::CACHE_CONTROL, "no-cache, no-transform, no-store")
        .header(axum::http::header::CONNECTION, "keep-alive")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        // X-Accel-Buffering: no prevents nginx from buffering the subscription stream
        // This ensures real-time updates are sent immediately
        // Reference: braid-http-server.js line 544
        .header("X-Accel-Buffering", "no")
        .body(body)
        .map_err(|e| {
            tracing::error!("[Server] Failed to build response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
}

