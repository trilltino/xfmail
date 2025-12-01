#![allow(unused_imports, unused_variables, dead_code, unused_mut)]

/**
 * Collaborative Editing Handlers
 * 
 * This module implements Braid protocol handlers for collaborative text editing
 * using diamond-types CRDT. It provides:
 * - GET /collab/:doc_id - Subscription endpoint for real-time updates
 * - PUT /collab/:doc_id - Update endpoint for applying CRDT operations
 */

use crate::backend::server::state::AppState;
use crate::backend::collab::state::{CollabState, generate_agent_id};
use crate::shared::{DocumentState, CRDTPatch, ApplyOperationsRequest};
use crate::shared::version_bridge::VersionMap;
use diamond_types::Frontier;
use diamond_types::list::ListBranch;
use axum::{
    body::Body,
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    response::Response,
};
use bytes::Bytes;
use futures_util::stream;

/// Handle Braid subscription for collaborative editing (GET /collab/:doc_id)
/// 
/// This handler implements the Braid subscription protocol for CRDT-based
/// collaborative editing. It streams CRDT patches as they arrive.
#[cfg(feature = "ssr")]
pub async fn handle_collab_subscription(
    State(app_state): State<AppState>,
    Path(doc_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    tracing::info!("[Collab] Subscription request for document: {}", doc_id);
    
    // Note: Subscribe header check is optional for now - client may not send it
    if !headers.contains_key("subscribe") {
        tracing::debug!("[Collab] Subscribe header missing - this is ok for now");
    }
    
    // Get or create document from app state
    let collab_state = app_state.collab_state.clone();
    let doc_state = {
        // Need write lock because get_or_create_document may create a new document
        let mut collab = collab_state.write().await;
        collab.get_or_create_document(doc_id.clone(), None).await
    };
    
    // Get Parents header for reconnection
    let parents_header = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Parse Structured Headers format
            s.split(',')
                .map(|s| s.trim().trim_matches('"'))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    
    // Get current document state
    let doc = doc_state.read().await;
    let current_state = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        doc.get_state()
    })) {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("[Collab] Failed to get document state: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    tracing::debug!("[Collab] Document state retrieved, content length: {}", current_state.content.len());
    
    // Create initial update
    let initial_update = match format_braid_collab_update(
        current_state.version.as_ref(),
        &current_state.content,
        true, // add_trailing_newlines
    ) {
        Ok(update) => update,
        Err(e) => {
            tracing::error!("[Collab] Failed to format update: {:?}", e);
            return Err(e);
        }
    };
    
    // Create stream that sends initial state and then listens for updates
    // For now, we'll just send the initial state
    // In a full implementation, you'd subscribe to a broadcast channel
    let stream = stream::once(async move { 
        Ok::<Bytes, Box<dyn std::error::Error + Send + Sync>>(initial_update) 
    });
    
    Ok(Response::builder()
        .status(209) // 209 Subscription
        .header("Subscribe", "")
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-cache, no-transform, no-store")
        .header("Connection", "keep-alive")
        .header("X-Accel-Buffering", "no")
        .body(Body::from_stream(stream))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

/// Handle Braid PUT for collaborative editing (PUT /collab/:doc_id)
/// 
/// This handler accepts CRDT operations and merges them into the document's OpLog.
#[cfg(feature = "ssr")]
pub async fn handle_collab_put(
    State(app_state): State<AppState>,
    Path(doc_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, StatusCode> {
    tracing::info!("[Collab] PUT request for document: {}", doc_id);
    
    // TODO: Re-enable authentication after debugging
    // For now, bypass JWT for debugging connection issues
    use axum::http::header::AUTHORIZATION;
    
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            tracing::debug!("[Collab] Auth header present: {}", header_str);
        }
    } else {
        tracing::debug!("[Collab] No auth header - bypassing JWT for debugging");
    }
    
    tracing::debug!("[Collab] Processing PUT (JWT bypass for debugging)");
    
    // Get or create document from app state
    let collab_state = app_state.collab_state.clone();
    let doc_state = {
        // Need write lock because get_or_create_document may create a new document
        let mut collab = collab_state.write().await;
        collab.get_or_create_document(doc_id.clone(), None).await
    };
    
    // Parse request body - can be either CRDT patch (binary) or operations (JSON)
    // For now, we'll support JSON operations
    let request: ApplyOperationsRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("[Collab] Failed to parse request: {:?}, body length: {}", e, body.len());
            if body.len() < 500 {
                if let Ok(body_str) = std::str::from_utf8(&body) {
                    tracing::error!("[Collab] Request body: {}", body_str);
                }
            }
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Get Parents header
    let parents_header = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().trim_matches('"'))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    
    // Apply operations to document
    let mut doc = doc_state.write().await;
    let agent_id_str = generate_agent_id();
    let agent_id = doc.oplog.get_or_create_agent_id(&agent_id_str);
    
    tracing::debug!("[Collab] Created agent ID: {}", agent_id);
    
    // Convert parent versions to frontier
    let mut version_map = doc.version_map.clone();
    let _parent_frontiers: Vec<Frontier> = if parents_header.is_empty() {
        // Use current frontier as parent
        vec![doc.oplog.local_frontier()]
    } else {
        parents_header.iter()
            .filter_map(|pid| version_map.braid_to_frontier(pid))
            .collect()
    };
    
    // For now, we'll apply operations directly
    // In a full implementation, you'd merge remote operations properly
    let mut branch = ListBranch::new_at_tip(&doc.oplog);
    
    tracing::debug!("[Collab] Applying {} operations", request.operations.len());
    
    // Apply insert operations
    for (idx, op) in request.operations.iter().enumerate() {
        match op {
            crate::shared::CRDTOperation::Insert { position, text } => {
                tracing::debug!("[Collab] Op {}: Insert at {} with text length {}", idx, position, text.len());
                branch.insert(&mut doc.oplog, agent_id, *position, text.as_str());
            }
            crate::shared::CRDTOperation::Delete { start, end } => {
                tracing::debug!("[Collab] Op {}: Delete from {} to {}", idx, start, end);
                branch.delete_without_content(&mut doc.oplog, agent_id, *start..*end);
            }
        }
    }
    
    // Get new frontier and convert to Braid version
    let new_frontier = doc.oplog.local_frontier();
    let new_version = version_map.frontier_to_braid(&new_frontier);
    doc.version_map = version_map;
    
    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        doc.update_metadata();
    })) {
        tracing::error!("[Collab] Error updating metadata: {:?}", e);
    }
    
    tracing::debug!("[Collab] New version: {}", new_version);
    
    // Broadcast update to subscribers (would be implemented with broadcast channel)
    
    // Return response with Version header
    let version_header_value = format!("\"{}\"", new_version);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Version", version_header_value)
        .body(Body::empty())
        .map_err(|e| {
            tracing::error!("[Collab] Failed to build response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
}

/// Format a Braid update for collaborative editing
/// 
/// Formats an update in pure Braid format with document content.
fn format_braid_collab_update(
    version: Option<&String>,
    content: &str,
    add_trailing_newlines: bool,
) -> Result<Bytes, StatusCode> {
    // Create JSON body with document state
    let json_body = serde_json::json!({
        "content": content,
        "version": version
    });
    
    let json_str = serde_json::to_string(&json_body)
        .map_err(|e| {
            tracing::error!("[Collab] Failed to serialize update: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let body_bytes = json_str.as_bytes();
    let content_length = body_bytes.len();
    
    // Build headers using CRLF line endings
    let mut header_lines = Vec::new();
    
    if let Some(ver) = version {
        let version_header = format!("\"{}\"", ver);
        header_lines.push(format!("Version: {}\r\n", version_header));
    }
    
    header_lines.push(format!("Content-Length: {}\r\n", content_length));
    header_lines.push("\r\n".to_string());
    
    let headers_str = header_lines.join("");
    let headers_bytes = headers_str.as_bytes();
    
    let trailing_newlines_size = if add_trailing_newlines { 4 } else { 0 };
    
    let mut result = Vec::with_capacity(headers_bytes.len() + body_bytes.len() + trailing_newlines_size);
    result.extend_from_slice(headers_bytes);
    result.extend_from_slice(body_bytes);
    
    if add_trailing_newlines {
        result.extend_from_slice(b"\r\n\r\n");
    }
    
    Ok(Bytes::from(result))
}