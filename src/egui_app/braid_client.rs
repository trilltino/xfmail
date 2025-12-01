/**
 * Braid HTTP Protocol Client
 * 
 * Implements the Braid HTTP protocol client for subscribing to updates
 * and sending PUT requests. Similar to public/braid_client.js but in Rust.
 */

use crate::egui_app::config::Config;
use crate::shared::Message;
use reqwest::Client;
use tokio::runtime::Runtime;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Braid client state
pub struct BraidClient {
    config: Config,
    client: Client,
    current_version: Option<String>,
    #[allow(dead_code)]
    subscription_thread: Option<thread::JoinHandle<()>>,
    #[allow(dead_code)]
    message_sender: Sender<Message>,
    #[allow(dead_code)]
    message_receiver: Receiver<Message>,
}

impl Default for BraidClient {
    fn default() -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        Self {
            config: Config::default(),
            client: Client::new(),
            current_version: None,
            subscription_thread: None,
            message_sender: message_tx,
            message_receiver: message_rx,
        }
    }
}

impl BraidClient {
    pub fn new(config: Config) -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        Self {
            config,
            client: Client::new(),
            current_version: None,
            subscription_thread: None,
            message_sender: message_tx,
            message_receiver: message_rx,
        }
    }
    
    /// Send a message via PUT /chat
    pub fn put_message(
        &mut self,
        message: Message,
        parents: Option<Vec<String>>,
    ) -> Result<String, String> {
        let url = self.config.api_url("/chat");
        let token = self.config.get_token().ok_or("Not authenticated")?;
        
        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        rt.block_on(async {
            let mut request = self.client
                .put(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json");
            
            // Add Parents header if provided (Structured Headers format)
            if let Some(ref parents_vec) = parents {
                let parents_header = parents_vec
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                request = request.header("Parents", parents_header);
            }
            
            let response = request
                .json(&message)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await
                .unwrap_or_else(|_| status.to_string());
                return Err(format!("PUT failed: {} - {}", status, error_text));
            }

            // Extract version from Version header (Structured Headers format)
            let version_header = response
                .headers()
                .get("Version")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");
            
            // Parse Structured Headers format: "version-id" or "version1", "version2"
            let version = version_header
                .trim_matches('"')
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
            
            self.current_version = Some(version.clone());
            Ok(version)
        })
    }
    
    /// Send collaborative edit via PUT /collab/:doc_id
    pub fn put_collab(
        &mut self,
        doc_id: &str,
        content: serde_json::Value,
        parents: Option<Vec<String>>,
    ) -> Result<String, String> {
        let url = self.config.api_url(&format!("/collab/{}", doc_id));
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let mut request = self.client
                .put(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json");

            // Add Parents header if provided (Structured Headers format)
            if let Some(ref parents_vec) = parents {
                let parents_header = parents_vec
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                request = request.header("Parents", parents_header);
            }

            let response = request
                .json(&content)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("PUT failed: {} - {}", status, error_text));
            }

            // Extract version from Version header (Structured Headers format)
            let version_header = response
                .headers()
                .get("Version")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            // Parse Structured Headers format: "version-id" or "version1", "version2"
            let version = version_header
                .trim_matches('"')
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();

            self.current_version = Some(version.clone());
            Ok(version)
        })
    }

    /// Get current version
    pub fn get_current_version(&self) -> Option<&String> {
        self.current_version.as_ref()
    }
}

