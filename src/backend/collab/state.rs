/**
 * Collaborative Editing State Management
 * 
 * This module manages server-side CRDT state for collaborative text editing.
 * It stores diamond-types OpLogs per document and maintains version mappings
 * between diamond-types Frontiers and Braid version IDs.
 */

use diamond_types::list::{ListOpLog, ListBranch};
use diamond_types::Frontier;
use crate::shared::version_bridge::VersionMap;
use crate::shared::{DocumentState, DocumentMetadata};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Per-document CRDT state
#[derive(Debug, Clone)]
pub struct DocumentCRDTState {
    /// The diamond-types OpLog for this document
    pub oplog: ListOpLog,
    /// Version mapping between diamond-types and Braid
    pub version_map: VersionMap,
    /// Document metadata
    pub metadata: DocumentMetadata,
}

impl DocumentCRDTState {
    /// Create a new document state
    pub fn new(doc_id: String, title: String) -> Self {
        let oplog = ListOpLog::new();
        // Note: doc_id is stored in metadata, not directly in oplog
        // The oplog's doc_id field is optional and used for document identification
        
        Self {
            oplog,
            version_map: VersionMap::new(),
            metadata: DocumentMetadata {
                id: doc_id,
                title,
                version: None,
                operation_count: 0,
            },
        }
    }

    /// Get current document state
    pub fn get_state(&self) -> DocumentState {
        let branch = ListBranch::new_at_tip(&self.oplog);
        let content = branch.content().to_string();
        let frontier = self.oplog.local_frontier();
        
        // Convert frontier to Braid version ID
        let mut version_map = self.version_map.clone();
        let version = if frontier.is_root() {
            None
        } else {
            Some(version_map.frontier_to_braid(&frontier))
        };
        
        // Get parent versions (for now, use empty - could be enhanced)
        let parents = Vec::new();
        
        DocumentState {
            content,
            version,
            parents,
        }
    }

    /// Get current frontier
    pub fn local_frontier(&self) -> Frontier {
        self.oplog.local_frontier()
    }

    /// Update metadata
    pub fn update_metadata(&mut self) {
        self.metadata.operation_count = self.oplog.len();
        let frontier = self.oplog.local_frontier();
        let mut version_map = self.version_map.clone();
        self.metadata.version = if frontier.is_root() {
            None
        } else {
            Some(version_map.frontier_to_braid(&frontier))
        };
    }
}

/// Collaborative editing state managed by the server
/// 
/// This structure stores all document OpLogs and their version mappings.
/// It implements thread-safe concurrent access using Arc<RwLock<>>.
#[derive(Debug, Clone)]
pub struct CollabState {
    /// Map of document ID to document CRDT state
    documents: Arc<RwLock<HashMap<String, DocumentCRDTState>>>,
}

impl CollabState {
    /// Create a new empty collaborative state
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new document
    /// 
    /// Returns true if the document was created, false if it already exists.
    pub async fn create_document(&self, doc_id: String, title: Option<String>) -> bool {
        let mut docs = self.documents.write().await;
        if docs.contains_key(&doc_id) {
            return false;
        }
        
        let title = title.unwrap_or_else(|| format!("Document {}", doc_id));
        let doc_state = DocumentCRDTState::new(doc_id.clone(), title);
        docs.insert(doc_id, doc_state);
        true
    }

    /// Get or create a document
    /// 
    /// Returns the document state, creating it if it doesn't exist.
    pub async fn get_or_create_document(&self, doc_id: String, title: Option<String>) -> Arc<RwLock<DocumentCRDTState>> {
        // Check if document exists
        {
            let docs = self.documents.read().await;
            if let Some(doc_state) = docs.get(&doc_id) {
                // Return a clone wrapped in Arc<RwLock>
                // Note: This is a simplified approach. In production, you might want
                // to store Arc<RwLock<DocumentCRDTState>> directly in the HashMap
                // to avoid cloning the entire state.
                return Arc::new(RwLock::new(doc_state.clone()));
            }
        }
        
        // Create new document
        let title = title.unwrap_or_else(|| format!("Document {}", doc_id));
        let doc_state = DocumentCRDTState::new(doc_id.clone(), title);
        
        {
            let mut docs = self.documents.write().await;
            docs.insert(doc_id.clone(), doc_state.clone());
        }
        
        Arc::new(RwLock::new(doc_state))
    }

    /// Get a document by ID
    /// 
    /// Returns None if the document doesn't exist.
    pub async fn get_document(&self, doc_id: &str) -> Option<Arc<RwLock<DocumentCRDTState>>> {
        let docs = self.documents.read().await;
        docs.get(doc_id).map(|doc_state| {
            Arc::new(RwLock::new(doc_state.clone()))
        })
    }

    /// Get document metadata
    pub async fn get_metadata(&self, doc_id: &str) -> Option<DocumentMetadata> {
        let docs = self.documents.read().await;
        docs.get(doc_id).map(|doc| {
            let mut metadata = doc.metadata.clone();
            // Update operation count
            metadata.operation_count = doc.oplog.len();
            metadata
        })
    }

    /// List all document IDs
    pub async fn list_documents(&self) -> Vec<String> {
        let docs = self.documents.read().await;
        docs.keys().cloned().collect()
    }

    /// Delete a document
    pub async fn delete_document(&self, doc_id: &str) -> bool {
        let mut docs = self.documents.write().await;
        docs.remove(doc_id).is_some()
    }
}

impl Default for CollabState {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to generate unique agent ID per session
// This ensures we never reuse agent IDs (critical for CRDT correctness)
pub fn generate_agent_id() -> String {
    // Use UUID to ensure uniqueness
    format!("agent-{}", Uuid::new_v4().to_string())
}

