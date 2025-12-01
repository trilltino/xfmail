//! Braid Message Sync Client
//!
//! This module implements the Braid-HTTP client for real-time message synchronization.

use crate::egui_app::config::Config;
use crate::shared::messaging::ChatMessage;
use reqwest::Client;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use uuid::Uuid;

/// Message sync client for Braid-HTTP
#[derive(Debug)]
pub struct MessageSyncClient {
    config: Config,
    client: Client,
    current_version: Option<String>,
    subscription_thread: Option<thread::JoinHandle<()>>,
    message_sender: Sender<ChatMessage>,
    message_receiver: Receiver<ChatMessage>,
    status_sender: Sender<SubscriptionStatus>,
    status_receiver: Receiver<SubscriptionStatus>,
}

impl Default for MessageSyncClient {
    fn default() -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel();
        Self {
            config: Config::default(),
            client: Client::new(),
            current_version: None,
            subscription_thread: None,
            message_sender: message_tx,
            message_receiver: message_rx,
            status_sender: status_tx,
            status_receiver: status_rx,
        }
    }
}

impl MessageSyncClient {
    pub fn new(config: Config) -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel();
        Self {
            config,
            client: Client::new(),
            current_version: None,
            subscription_thread: None,
            message_sender: message_tx,
            message_receiver: message_rx,
            status_sender: status_tx,
            status_receiver: status_rx,
        }
    }

    /// Subscribe to a conversation's message stream
    pub fn subscribe_to_conversation(&mut self, conversation_id: Uuid) {
        // Stop existing subscription if any
        if self.subscription_thread.is_some() {
            // Join the old thread (it will terminate when we create a new one)
            self.subscription_thread = None;
        }

        let config = self.config.clone();
        let message_sender = self.message_sender.clone();
        let status_sender = self.status_sender.clone();

        let thread = std::thread::spawn(move || {
            subscribe_to_stream(config, conversation_id, message_sender, status_sender);
        });

        self.subscription_thread = Some(thread);
    }

    /// Send a message via PUT
    pub fn send_message(
        &mut self,
        conversation_id: Uuid,
        content: String,
        parents: Option<Vec<String>>,
    ) -> Result<(Uuid, String), String> {
        tracing::info!("[BRAID] Client sending message: conversation={}, content_preview='{}...'",
                      conversation_id, &content[..content.len().min(50)]);
        let message_id = Uuid::new_v4();
        let url = self.config.api_url(&format!(
            "/sync/conversations/{}/messages/{}",
            conversation_id, message_id
        ));
        // Prefer real JWT when available; otherwise, allow dev bypass if configured
        let token_opt = self.config.get_token().cloned();
        let dev_bypass = self.config.dev_auth_bypass();
        let dev_user_id = self.config.dev_user_id().map(|s| s.to_string());
        if token_opt.is_none() && !dev_bypass {
            return Err("Not authenticated".to_string());
        }

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let mut request = self
                .client
                .put(&url)
                .header("Content-Type", "application/json");

            if let Some(token) = token_opt.as_ref() {
                request = request.header("Authorization", format!("Bearer {}", token));
            } else if dev_bypass {
                if let Some(uid) = dev_user_id.as_ref() {
                    request = request.header("X-Dev-User-Id", uid);
                }
            }

            // Add Parents header if provided (Structured Headers format)
            if let Some(ref parents_vec) = parents {
                let parents_header = parents_vec
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                request = request.header("Parents", parents_header);
            }

            let body = serde_json::json!({
                "content": content,
                "message_type": "text"
            });

            let response = request
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("PUT failed: {} - {}", status, error_text));
            }

            // Extract version from Version header
            let version_header = response
                .headers()
                .get("Version")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            let version = version_header
                .trim_matches('"')
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();

            self.current_version = Some(version.clone());
            Ok((message_id, version))
        })
    }

    /// Get current version
    pub fn get_current_version(&self) -> Option<&String> {
        self.current_version.as_ref()
    }

    /// Check for new messages (non-blocking)
    pub fn poll_messages(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.message_receiver.try_recv() {
            messages.push(msg);
        }
        messages
    }

    /// Poll latest subscription status update (non-blocking)
    pub fn poll_status(&self) -> Option<SubscriptionStatus> {
        self.status_receiver.try_recv().ok()
    }
}

/// Subscription status reported by the client
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Connecting,
    Connected,
    Retrying,
    Error(String),
    Disconnected,
}

/// Subscribe to SSE stream for a conversation
fn subscribe_to_stream(
    config: crate::egui_app::config::Config,
    conversation_id: Uuid,
    message_sender: Sender<ChatMessage>,
    status_sender: Sender<SubscriptionStatus>,
) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!("Failed to create runtime for message subscription: {}", e);
            return;
        }
    };

    rt.block_on(async {
        let mut reconnect_delay = std::time::Duration::from_millis(1000);
        const MAX_RECONNECT_DELAY: std::time::Duration = std::time::Duration::from_secs(30);

        loop {
            let url = config.api_url(&format!(
                "/sync/conversations/{}/messages",
                conversation_id
            ));

            let token_opt = config.get_token().cloned();
            let dev_bypass = config.dev_auth_bypass();
            let dev_user_id = config.dev_user_id().map(|s| s.to_string());
            
            println!("[CLIENT-SUB] Attempting subscription: {}", url);
            println!("[CLIENT-SUB] Token: {}, Dev bypass: {}, Dev user: {:?}", token_opt.is_some(), dev_bypass, dev_user_id);
            
            if token_opt.is_none() && !dev_bypass {
                println!("[CLIENT-SUB] ERROR: No token and no dev bypass!");
                tracing::error!("No authentication token available and DEV_AUTH_BYPASS disabled");
                break;
            }

            let client = Client::new();

            let mut req = client.get(&url).header("Subscribe", "true");
            if let Some(token) = token_opt.as_ref() {
                req = req.header("Authorization", format!("Bearer {}", token));
            } else if dev_bypass {
                if let Some(uid) = dev_user_id.as_ref() {
                    req = req.header("X-Dev-User-Id", uid);
                    println!("[CLIENT-SUB] Using X-Dev-User-Id: {}", uid);
                }
            }

            println!("[CLIENT-SUB] Sending request to: {}", url);
            tracing::info!("[BRAID] Subscribing to SSE: {}", url);
            // Report connecting
            let _ = status_sender.send(SubscriptionStatus::Connecting);
            let response = match req.send()
                .await
            {
                Ok(resp) => {
                    println!("[CLIENT-SUB] Response received: {}", resp.status());
                    resp
                }
                Err(e) => {
                    println!("[CLIENT-SUB] Request failed: {}", e);
                    tracing::warn!("Failed to subscribe to message stream (will retry): {}", e);
                    let _ = status_sender.send(SubscriptionStatus::Error(format!("network: {}", e)));
                    let _ = status_sender.send(SubscriptionStatus::Retrying);
                    tokio::time::sleep(reconnect_delay).await;
                    reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
                    continue;
                }
            };

            if !response.status().is_success() {
                println!("[CLIENT-SUB] ERROR: Subscription failed with status: {}", response.status());
                tracing::error!(
                    "Subscription failed with status: {} (will retry)",
                    response.status()
                );
                let _ = status_sender.send(SubscriptionStatus::Error(format!("http: {}", response.status())));
                let _ = status_sender.send(SubscriptionStatus::Retrying);
                tokio::time::sleep(reconnect_delay).await;
                reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
                continue;
            }
            
            println!("[CLIENT-SUB] Connected successfully!");

            tracing::info!("[BRAID] SSE subscription established for conversation {}", conversation_id);
            let _ = status_sender.send(SubscriptionStatus::Connected);

            // Reset reconnect delay on successful connection
            reconnect_delay = std::time::Duration::from_millis(1000);

            // Read SSE stream as bytes stream
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut connection_active = true;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Convert bytes to string
                        let chunk_str = match std::str::from_utf8(&chunk) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("Invalid UTF-8 in SSE stream: {}", e);
                                connection_active = false;
                                break;
                            }
                        };

                        // Append to buffer
                        buffer.push_str(chunk_str);

                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            // Skip empty lines and comments
                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }

                            // Parse SSE event data lines (data: ...)
                            if let Some(data_content) = line.strip_prefix("data: ") {
                                // Try to parse as JSON
                                if let Ok(msg) = serde_json::from_str::<ChatMessage>(data_content) {
                                    tracing::debug!("Received message via SSE: {:?}", msg.id);
                                    if let Err(e) = message_sender.send(msg) {
                                        tracing::error!("Failed to send message to channel: {}", e);
                                        return;
                                    }
                                } else {
                                    tracing::warn!("Failed to parse SSE data as ChatMessage: {}", data_content);
                                }
                                continue;
                            }

                            // Fallback: If the line looks like raw JSON (some proxies or servers may send JSON lines)
                            if line.starts_with('{') && line.ends_with('}') {
                                match serde_json::from_str::<ChatMessage>(&line) {
                                    Ok(msg) => {
                                        tracing::debug!("Received JSON line message: {:?}", msg.id);
                                        if let Err(e) = message_sender.send(msg) {
                                            tracing::error!("Failed to send message to channel: {}", e);
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse JSON line as ChatMessage: {} | line: {}", e, line);
                                    }
                                }
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading from SSE stream: {}", e);
                        connection_active = false;
                        let _ = status_sender.send(SubscriptionStatus::Error(format!("stream: {}", e)));
                        break;
                    }
                }
            }

            if connection_active {
                tracing::info!("Message stream closed normally for conversation {}", conversation_id);
                let _ = status_sender.send(SubscriptionStatus::Disconnected);
                break; // Normal closure, don't reconnect
            } else {
                tracing::warn!("Message stream connection lost for conversation {}, will reconnect", conversation_id);
                let _ = status_sender.send(SubscriptionStatus::Retrying);
                tokio::time::sleep(reconnect_delay).await;
                reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
            }
        }
    });
}

