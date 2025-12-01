/**
 * Editing Demo Module
 * 
 * Collaborative text editing demo with diamond-types CRDT integration.
 * This demo shows real-time collaborative editing using the Braid protocol.
 */

use eframe::egui;
use diamond_types::list::ListCRDT;
use crate::egui_app::config::Config;
use crate::egui_app::braid_client::BraidClient;
use crate::shared::{CRDTOperation, ApplyOperationsRequest};
use std::sync::{Arc, Mutex};

/// Editing demo state
pub struct EditingDemo {
    /// Local CRDT instance
    crdt: Arc<Mutex<ListCRDT>>,
    /// Document ID
    doc_id: String,
    /// Current document content (cached from CRDT)
    content: String,
    /// Text input buffer
    text_input: String,
    /// Connection status
    is_connected: bool,
    /// Current version
    current_version: Option<String>,
    /// Braid client for syncing
    braid_client: Option<BraidClient>,
    /// Config for server URL
    config: Config,
}


impl Default for EditingDemo {
    fn default() -> Self {
        let mut crdt = ListCRDT::new();
        let _agent_id = crdt.oplog.get_or_create_agent_id(&format!("egui-{}", uuid::Uuid::new_v4().to_string()));
        
        Self {
            crdt: Arc::new(Mutex::new(crdt)),
            doc_id: "default".to_string(),
            content: String::new(),
            text_input: String::new(),
            is_connected: false,
            current_version: None,
            braid_client: None,
            config: Config::new(),
        }
    }
}

impl EditingDemo {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize connection to server
    pub fn connect(&mut self) {
        if self.braid_client.is_none() {
            self.braid_client = Some(BraidClient::new(self.config.clone()));
            self.is_connected = true;
        }
    }
    
    /// Send local operations to server
    fn send_operations(&mut self, operations: Vec<CRDTOperation>) {
        if let Some(ref mut client) = self.braid_client {
            let request = ApplyOperationsRequest {
                operations,
                parents: self.current_version.clone().map(|v| vec![v]).unwrap_or_default(),
                version: None,
            };

            // Send via PUT /collab/:doc_id
            match client.put_collab(&self.doc_id, serde_json::to_value(request).unwrap(), None) {
                Ok(version) => {
                    self.current_version = Some(version.clone());
                    eprintln!("[EditingDemo] Successfully sent operations, new version: {}", version);
                }
                Err(e) => {
                    eprintln!("[EditingDemo] Failed to send operations: {}", e);
                }
            }
        }
    }
    
    /// Update content from CRDT
    fn update_content(&mut self) {
        let crdt = self.crdt.lock().unwrap();
        self.content = crdt.branch.content().to_string();
    }
    
    /// Render the editing demo UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Collaborative Text Editor");
            ui.separator();
            // Connection status
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.connect();
                }
                if self.is_connected {
                    ui.label(egui::RichText::new("● Connected").color(egui::Color32::from_rgb(0, 255, 0)));
                } else {
                    ui.label(egui::RichText::new("○ Disconnected").color(egui::Color32::from_rgb(255, 0, 0)));
                }
                if let Some(ref version) = self.current_version {
                    ui.label(format!("Version: {}", version));
                }
            });
            
            ui.separator();
            
            // Document ID input
            ui.horizontal(|ui| {
                ui.label("Document ID:");
                ui.text_edit_singleline(&mut self.doc_id);
            });
            
            ui.separator();
            
            // Text editor
            ui.label("Document Content:");
            let text_edit = egui::TextEdit::multiline(&mut self.text_input)
                .desired_width(f32::INFINITY)
                .desired_rows(20);
            
            let response = ui.add(text_edit);
            
            // Detect text changes and convert to CRDT operations
            if response.changed() {
                // Simple diff: compare old content with new
                let old_content = &self.content;
                let new_content = &self.text_input;
                
                // For now, we'll do a simple approach:
                // If content changed, update the CRDT
                if old_content != new_content {
                    // Calculate diff and create operations
                    // This is simplified - in production, you'd want proper diffing
                    let mut operations = Vec::new();
                    
                    // If new content is longer, it's an insert
                    if new_content.len() > old_content.len() {
                        let insert_pos = old_content.len();
                        let insert_text = &new_content[old_content.len()..];
                        operations.push(CRDTOperation::insert(insert_pos, insert_text.to_string()));
                    }
                    // If new content is shorter, it's a delete
                    else if new_content.len() < old_content.len() {
                        let delete_start = new_content.len();
                        let delete_end = old_content.len();
                        operations.push(CRDTOperation::delete(delete_start, delete_end));
                    }
                    
                    // Apply operations to local CRDT
                    if !operations.is_empty() {
                        let agent_id = {
                            let mut crdt = self.crdt.lock().unwrap();
                            crdt.oplog.get_or_create_agent_id(&format!("egui-{}", uuid::Uuid::new_v4().to_string()))
                        };
                        
                        {
                            let mut crdt = self.crdt.lock().unwrap();
                            for op in &operations {
                                match op {
                                    CRDTOperation::Insert { position, text } => {
                                        crdt.insert(agent_id, *position, text.as_str());
                                    }
                                    CRDTOperation::Delete { start, end } => {
                                        crdt.delete_without_content(agent_id, *start..*end);
                                    }
                                }
                            }
                        }
                        
                        // Update cached content
                        self.update_content();
                        
                        // Send to server
                        self.send_operations(operations);
                    }
                }
            }
            ui.separator();            
            
            // CRDT info
            ui.collapsing("CRDT Info", |ui| {
                let crdt = self.crdt.lock().unwrap();
                ui.label(format!("Operations: {}", crdt.oplog.len()));
                ui.label(format!("Content length: {} chars", self.content.len()));
            });
        });
    }
}
