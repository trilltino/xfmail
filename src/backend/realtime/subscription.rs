/**
 * Real-time Subscription Handler
 * 
 * This module implements the Server-Sent Events (SSE) subscription handler
 * for the `/realtime` endpoint. It provides a generic real-time update stream
 * that can handle multiple types of events.
 * 
 * # Server-Sent Events (SSE)
 * 
 * This endpoint uses SSE to provide a one-way stream of events from server
 * to client. SSE is simpler than WebSockets for one-way communication and
 * works well with HTTP/2.
 * 
 * # Event Filtering
 * 
 * Clients can filter events by type using the `types` query parameter:
 * - `?types=message,notification` - Subscribe to messages and notifications
 * - `?types=typing` - Subscribe only to typing events
 * - No parameter - Subscribe to all event types
 * 
 * # Connection Management
 * 
 * - Connections are kept alive using SSE keep-alive mechanism
 * - Clients can reconnect using `Last-Event-ID` header
 * - Lagged events are logged but don't cause connection drops
 */

use crate::shared::EventType;
use crate::backend::realtime::broadcast::RealtimeEventBroadcast;
use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
};
use futures_util::stream;
use std::collections::HashMap;

/// Handle real-time subscription (GET /realtime)
/// 
/// This endpoint provides a generic real-time update stream using Server-Sent Events.
/// Clients can subscribe to receive real-time updates for different event types.
/// 
/// # Query Parameters
/// 
/// - `types` - Comma-separated list of event types to subscribe to (optional)
///   - If not provided, subscribes to all event types
///   - Examples: `?types=message,notification` or `?types=status`
/// 
/// # Headers
/// 
/// - `Subscribe:` - Required header to initiate subscription
/// - `Last-Event-ID:` - Optional header for reconnection (event ID)
/// 
/// # Returns
/// 
/// Server-Sent Events stream with real-time updates
/// 
/// # Errors
/// 
/// * `400 Bad Request` - If Subscribe header is missing
/// 
/// # Example Request
/// 
/// ```http
/// GET /realtime?types=message,notification HTTP/1.1
/// Subscribe:
/// ```
/// 
/// # Example Response
/// 
/// ```http
/// HTTP/1.1 200 OK
/// Content-Type: text/event-stream
/// Cache-Control: no-cache
/// Connection: keep-alive
/// 
/// event: message
/// data: {"event_type":"message","payload":{...},"timestamp":"..."}
/// 
/// event: notification
/// data: {"event_type":"notification","payload":{...},"timestamp":"..."}
/// ```
#[cfg(feature = "ssr")]
pub async fn handle_realtime_subscription(
    State(broadcast_tx): State<RealtimeEventBroadcast>,
    headers: axum::http::HeaderMap,
    query: axum::extract::Query<HashMap<String, String>>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    tracing::info!("[Realtime] Subscription request received");
    
    // Check if Subscribe header is present
    if !headers.contains_key("subscribe") {
        tracing::warn!("[Realtime] Subscribe header missing");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Parse event types filter from query parameters
    let event_types_filter: Option<Vec<EventType>> = query
        .get("types")
        .map(|types_str| {
            types_str
                .split(',')
                .map(|s| s.trim())
                .filter_map(|s| {
                    match s.to_lowercase().as_str() {
                        "message" => Some(EventType::Message),
                        "notification" => Some(EventType::Notification),
                        "status" => Some(EventType::Status),
                        "typing" => Some(EventType::Typing),
                        custom if !custom.is_empty() => Some(EventType::Custom(custom.to_string())),
                        _ => None,
                    }
                })
                .collect()
        })
        .filter(|v: &Vec<_>| !v.is_empty());
    
    if let Some(ref types) = event_types_filter {
        tracing::info!("[Realtime] Filtering events by types: {:?}", types);
    } else {
        tracing::info!("[Realtime] Subscribing to all event types");
    }
    
    // Subscribe to broadcast channel
    let broadcast_rx = broadcast_tx.subscribe();
    let filter = event_types_filter;
    
    tracing::info!("[Realtime] Subscription active, waiting for events...");
    
    // Create SSE stream that listens to broadcast channel
    // Loop until we get a meaningful event (matches the pattern from handlers.rs)
    // We only yield events when there's actual data to send
    // Axum's keep-alive mechanism will automatically inject comment lines (":")
    // to maintain the connection, so we don't need to send empty data events
    let stream = stream::unfold(
        (broadcast_rx, filter),
        move |(mut rx, filter)| async move {
            // Loop until we get a meaningful event that passes the filter
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        // Apply event type filter if specified
                        if let Some(ref filter_types) = filter {
                            if !filter_types.contains(&event.event_type) {
                                // Event type not in filter, continue looping for next event
                                tracing::debug!("[Realtime] Event type filtered out, continuing to listen");
                                continue;
                            }
                        }
                        
                        // Serialize event to JSON
                        let event_data = match serde_json::to_string(&event) {
                            Ok(data) => data,
                            Err(e) => {
                                tracing::error!("[Realtime] Failed to serialize event: {:?}", e);
                                // Continue looping on serialization error
                                continue;
                            }
                        };
                        
                        // Create SSE event with event type as the event name
                        let event_name = match &event.event_type {
                            EventType::Message => "message",
                            EventType::Notification => "notification",
                            EventType::Status => "status",
                            EventType::Typing => "typing",
                            EventType::Custom(name) => name.as_str(),
                        };
                        
                        tracing::info!("[Realtime] Broadcasting event: {} to subscriber", event_name);
                        
                        let sse_event = Event::default()
                            .event(event_name)
                            .data(event_data);
                        
                        return Some((Ok(sse_event), (rx, filter)));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("[Realtime] Receiver lagged, skipped {} events", skipped);
                        // Continue looping - we'll catch up on next event
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::warn!("[Realtime] Broadcast channel closed, ending stream");
                        return None;
                    }
                }
            }
        },
    );
    
    // Create SSE response with keep-alive
    let sse = Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default());
    
    Ok(sse)
}

