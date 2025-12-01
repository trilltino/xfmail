//! Messaging State
//!
//! This module contains the state management for the messaging UI.

use crate::shared::messaging::{Contact, ChatMessage, Conversation, FriendRequest};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Receiver;
use uuid::Uuid;
use super::braid_sync::{MessageSyncClient, SubscriptionStatus};
// use crate::egui_app::config::Config; // Currently unused

/// Pending API operation result types
pub type FriendRequestResult = Result<(), String>;
pub type LoadRequestsResult = Result<Vec<FriendRequest>, String>;
pub type LoadContactsResult = Result<Vec<Contact>, String>;
pub type LoadConversationsResult = Result<Vec<Conversation>, String>;

/// The main state for the messaging UI
pub struct MessagingState {
    /// Current user's ID
    pub current_user_id: Option<Uuid>,
    /// Current user's username
    pub current_username: Option<String>,

    /// Braid message sync client
    pub message_sync_client: Option<MessageSyncClient>,

    /// List of contacts (friends)
    pub contacts: Vec<Contact>,
    /// Map of conversation ID to conversation
    pub conversations: HashMap<Uuid, Conversation>,
    /// Map of conversation ID to messages
    pub messages: HashMap<Uuid, Vec<ChatMessage>>,

    /// Currently selected conversation ID
    pub selected_conversation_id: Option<Uuid>,

    /// Pending friend requests (received)
    pub incoming_friend_requests: Vec<FriendRequest>,
    /// Pending friend requests (sent)
    pub outgoing_friend_requests: Vec<FriendRequest>,

    /// Search query for contacts
    pub search_query: String,
    /// Message input text
    pub message_input: String,

    /// Add friend modal state
    pub show_add_friend_modal: bool,
    pub add_friend_email: String,
    pub add_friend_message: String,
    pub add_friend_error: Option<String>,
    pub add_friend_success: Option<String>,

    /// Friend requests panel state
    pub show_friend_requests_panel: bool,

    /// Chat header menu state
    pub show_chat_header_menu: bool,

    /// Loading states
    pub is_loading_contacts: bool,
    pub is_loading_conversations: bool,
    pub is_loading_messages: bool,
    pub is_sending_message: bool,
    pub is_sending_friend_request: bool,

    /// Pending async operation receivers
    pub pending_send_request: Option<Receiver<FriendRequestResult>>,
    pub pending_accept_request: Option<(Uuid, Receiver<FriendRequestResult>)>,
    pub pending_reject_request: Option<(Uuid, Receiver<FriendRequestResult>)>,
    pub pending_load_requests: Option<Receiver<LoadRequestsResult>>,
    pub pending_load_contacts: Option<Receiver<LoadContactsResult>>,
    pub pending_load_conversations: Option<Receiver<LoadConversationsResult>>,

    /// Flag to trigger contacts reload on next frame
    pub should_reload_contacts: bool,

    /// Frame counter for throttling contact reloads
    pub contact_reload_frames: u32,

    /// Whether initial data has been loaded
    pub initialized: bool,

    /// Offline message queue for when network is unavailable
    pub offline_queue: VecDeque<ChatMessage>,

    /// Network connectivity status
    pub is_online: bool,

    /// Last time we successfully synced with server
    pub last_sync_time: Option<std::time::Instant>,

    /// Transient UI error to show to the user (e.g., auth or network issues)
    pub ui_error: Option<String>,

    /// Tracks which conversation we last subscribed to (to avoid re-subscribing every frame)
    pub last_subscribed_conversation_id: Option<uuid::Uuid>,

    /// Latest subscription status for the selected conversation
    pub subscription_status: Option<SubscriptionStatus>,

    /// Whether to show the connection log panel
    pub show_connection_log: bool,
    /// Recent subscription status log lines
    pub subscription_log: Vec<String>,
    /// Remember last status to avoid duplicate log entries
    pub last_subscription_status: Option<SubscriptionStatus>,
}

impl Default for MessagingState {
    fn default() -> Self {
        Self::new()
    }
}

impl MessagingState {
    pub fn new() -> Self {
        Self {
            current_user_id: None,
            current_username: None,
            message_sync_client: None,
            contacts: Vec::new(),
            conversations: HashMap::new(),
            messages: HashMap::new(),
            selected_conversation_id: None,
            incoming_friend_requests: Vec::new(),
            outgoing_friend_requests: Vec::new(),
            search_query: String::new(),
            message_input: String::new(),
            show_add_friend_modal: false,
            add_friend_email: String::new(),
            add_friend_message: String::new(),
            add_friend_error: None,
            add_friend_success: None,
            show_friend_requests_panel: false,
            show_chat_header_menu: false,
            is_loading_contacts: false,
            is_loading_conversations: false,
            is_loading_messages: false,
            is_sending_message: false,
            is_sending_friend_request: false,
            pending_send_request: None,
            pending_accept_request: None,
            pending_reject_request: None,
            pending_load_requests: None,
            pending_load_contacts: None,
            pending_load_conversations: None,
            should_reload_contacts: false,
            contact_reload_frames: 0,
            initialized: false,
            offline_queue: VecDeque::new(),
            is_online: true,
            last_sync_time: Some(std::time::Instant::now()),
            ui_error: None,
            last_subscribed_conversation_id: None,
            subscription_status: None,
            show_connection_log: false,
            subscription_log: Vec::new(),
            last_subscription_status: None,
        }
    }
    
    /// Get the currently selected conversation
    pub fn selected_conversation(&self) -> Option<&Conversation> {
        self.selected_conversation_id
            .and_then(|id| self.conversations.get(&id))
    }
    
    /// Get messages for the currently selected conversation
    pub fn selected_messages(&self) -> Option<&Vec<ChatMessage>> {
        self.selected_conversation_id
            .and_then(|id| self.messages.get(&id))
    }
    
    /// Get filtered contacts based on search query
    pub fn filtered_contacts(&self) -> Vec<&Contact> {
        let query = self.search_query.to_lowercase().trim().to_string();
        
        if query.is_empty() {
            return self.contacts.iter().collect();
        }
        
        self.contacts
            .iter()
            .filter(|c| {
                c.username.to_lowercase().contains(query.as_str())
                    || c.email.to_lowercase().contains(query.as_str())
                    || c.display_name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(query.as_str()))
                        .unwrap_or(false)
            })
            .collect()
    }
    
    /// Select a conversation
    pub fn select_conversation(&mut self, conversation_id: Uuid) {
        self.selected_conversation_id = Some(conversation_id);
    }
    
    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selected_conversation_id = None;
    }
    
    /// Open the add friend modal
    pub fn open_add_friend_modal(&mut self) {
        self.show_add_friend_modal = true;
        self.add_friend_email.clear();
        self.add_friend_message.clear();
        self.add_friend_error = None;
        self.add_friend_success = None;
    }

    /// Close the add friend modal
    pub fn close_add_friend_modal(&mut self) {
        self.show_add_friend_modal = false;
        self.is_sending_friend_request = false;
    }

    /// Toggle friend requests panel
    pub fn toggle_friend_requests_panel(&mut self) {
        self.show_friend_requests_panel = !self.show_friend_requests_panel;
    }

    /// Get the count of pending friend requests
    pub fn pending_request_count(&self) -> usize {
        self.incoming_friend_requests.len()
    }

    /// Check for pending async operation results
    pub fn check_pending_operations(&mut self) {
        // Check send friend request result
        if let Some(ref rx) = self.pending_send_request {
            if let Ok(result) = rx.try_recv() {
                self.pending_send_request = None;
                self.is_sending_friend_request = false;
                match result {
                    Ok(()) => {
                        self.add_friend_success = Some("Friend request sent!".to_string());
                        self.add_friend_error = None;
                    }
                    Err(e) => {
                        self.add_friend_error = Some(e);
                        self.add_friend_success = None;
                    }
                }
            }
        }

        // Check accept friend request result
        if let Some((request_id, ref rx)) = self.pending_accept_request {
            if let Ok(result) = rx.try_recv() {
                self.pending_accept_request = None;
                match result {
                    Ok(()) => {
                        // Remove from incoming requests
                        self.incoming_friend_requests.retain(|r| r.id != request_id);
                        // Trigger reload of contacts list on next frame
                        self.should_reload_contacts = true;
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept friend request: {}", e);
                    }
                }
            }
        }

        // Check reject friend request result
        if let Some((request_id, ref rx)) = self.pending_reject_request {
            if let Ok(result) = rx.try_recv() {
                self.pending_reject_request = None;
                match result {
                    Ok(()) => {
                        // Remove from incoming requests
                        self.incoming_friend_requests.retain(|r| r.id != request_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to reject friend request: {}", e);
                    }
                }
            }
        }

        // Check load friend requests result
        if let Some(ref rx) = self.pending_load_requests {
            if let Ok(result) = rx.try_recv() {
                self.pending_load_requests = None;
                match result {
                    Ok(requests) => {
                        self.incoming_friend_requests = requests;
                    }
                    Err(e) => {
                        tracing::error!("Failed to load friend requests: {}", e);
                    }
                }
            }
        }

        // Check load contacts result
        if let Some(ref rx) = self.pending_load_contacts {
            if let Ok(result) = rx.try_recv() {
                self.pending_load_contacts = None;
                self.is_loading_contacts = false;
                match result {
                    Ok(contacts) => {
                        self.contacts = contacts;
                    }
                    Err(e) => {
                        tracing::error!("Failed to load contacts: {}", e);
                    }
                }
            }
        }

        // Check load conversations result
        if let Some(ref rx) = self.pending_load_conversations {
            if let Ok(result) = rx.try_recv() {
                self.pending_load_conversations = None;
                self.is_loading_conversations = false;
                match result {
                    Ok(conversations) => {
                        self.conversations = conversations.into_iter().map(|c| (c.id, c)).collect();
                        // Auto-select first conversation if none selected yet
                        if self.selected_conversation_id.is_none() {
                            if let Some((&first_id, _)) = self.conversations.iter().next() {
                                tracing::info!("[BRAID] Auto-selecting first conversation: {}", first_id);
                                self.selected_conversation_id = Some(first_id);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load conversations: {}", e);
                        self.ui_error = Some(format!("Failed to load conversations: {}", e));
                    }
                }
            }
        }
    }

    /// Queue a message for offline sending
    pub fn queue_message_offline(&mut self, message: ChatMessage) {
        tracing::info!("[BRAID] Queuing message offline: id={}, content={}", message.id, message.content);
        self.offline_queue.push_back(message);
    }

    /// Sync offline messages when back online
    pub fn sync_offline_messages(&mut self) {
        if !self.is_online || self.offline_queue.is_empty() {
            return;
        }

        tracing::info!("[BRAID] Syncing {} offline messages", self.offline_queue.len());

        while let Some(message) = self.offline_queue.pop_front() {
            // Try to send the message
            if let Some(ref mut client) = self.message_sync_client {
                match client.send_message(message.conversation_id, message.content.clone(), None) {
                    Ok((msg_id, version)) => {
                        tracing::info!("[BRAID] Successfully synced offline message: id={}, version={}", msg_id, version);
                        self.last_sync_time = Some(std::time::Instant::now());
                    }
                    Err(e) => {
                        tracing::warn!("[BRAID] Failed to sync offline message: {}, re-queuing", e);
                        self.offline_queue.push_front(message);
                        break; // Stop trying to sync more messages
                    }
                }
            }
        }
    }

    /// Update network status
    pub fn set_online_status(&mut self, online: bool) {
        let was_online = self.is_online;
        self.is_online = online;

        if !was_online && online {
            tracing::info!("[BRAID] Network connection restored, syncing offline messages");
            self.sync_offline_messages();
        } else if was_online && !online {
            tracing::warn!("[BRAID] Network connection lost, queuing messages offline");
        }
    }

    /// Get the count of queued offline messages
    pub fn offline_message_count(&self) -> usize {
        self.offline_queue.len()
    }
}

