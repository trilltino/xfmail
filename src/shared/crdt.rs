/**
 * Shared CRDT Types
 * 
 * This module defines types and data structures for collaborative editing
 * using diamond-types CRDT. These types are shared between frontend and
 * backend for serialization and network transport.
 */

use serde::{Deserialize, Serialize};

/// CRDT operation types for collaborative editing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum CRDTOperation {
    /// Insert text at a specific position
    Insert {
        /// Position in the document (character index)
        position: usize,
        /// Text to insert
        text: String,
    },
    /// Delete a range of text
    Delete {
        /// Start position (inclusive)
        start: usize,
        /// End position (exclusive)
        end: usize,
    },
}

/// Document state snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentState {
    /// The document content
    pub content: String,
    /// Current version (Braid version ID)
    pub version: Option<String>,
    /// Parent versions (for Braid DAG)
    pub parents: Vec<String>,
}

/// CRDT patch for syncing operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CRDTPatch {
    /// The patch data (encoded from diamond-types)
    /// This is the binary encoding from diamond-types OpLog
    pub data: Vec<u8>,
    /// Version this patch applies to
    pub version: Option<String>,
    /// Parent versions
    pub parents: Vec<String>,
}

/// Request to apply CRDT operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyOperationsRequest {
    /// CRDT operations to apply
    pub operations: Vec<CRDTOperation>,
    /// Parent version(s) for Braid DAG
    pub parents: Vec<String>,
    /// Optional version ID (server will generate if not provided)
    pub version: Option<String>,
}

/// Response after applying operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyOperationsResponse {
    /// Assigned version ID
    pub version: String,
    /// Current document state
    pub state: DocumentState,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentMetadata {
    /// Document ID
    pub id: String,
    /// Document title/name
    pub title: String,
    /// Current version
    pub version: Option<String>,
    /// Number of operations in the OpLog
    pub operation_count: usize,
}

impl CRDTOperation {
    /// Create a new insert operation
    pub fn insert(position: usize, text: String) -> Self {
        Self::Insert { position, text }
    }

    /// Create a new delete operation
    pub fn delete(start: usize, end: usize) -> Self {
        Self::Delete { start, end }
    }
}

impl DocumentState {
    /// Create a new empty document state
    pub fn new() -> Self {
        Self {
            content: String::new(),
            version: None,
            parents: Vec::new(),
        }
    }

    /// Create document state with content
    pub fn with_content(content: String, version: Option<String>, parents: Vec<String>) -> Self {
        Self {
            content,
            version,
            parents,
        }
    }
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::new()
    }
}

