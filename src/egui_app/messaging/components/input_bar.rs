//! Input Bar Component
//!
//! The message input bar at the bottom of the chat area.

use eframe::egui;
use crate::egui_app::messaging::state::MessagingState;
use crate::egui_app::theme::colors;

/// Render the input bar
pub fn render(ui: &mut egui::Ui, state: &mut MessagingState, is_online: bool) {
    egui::Frame::new()
        .fill(colors::INPUT_BG)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            
            ui.horizontal(|ui| {
                // Attachment button
                if ui.button("📎").clicked() {
                    // TODO: Open file picker
                }
                
                // Message input
                let hint_text = if is_online {
                    "Type a message..."
                } else {
                    "You're offline - message will be sent when online"
                };

                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.message_input)
                        .hint_text(hint_text)
                        .desired_width(ui.available_width() - 80.0)
                );
                
                // Send on Enter
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if response.lost_focus() && enter_pressed {
                    tracing::info!("[BRAID] Enter key pressed, calling send_message");
                    send_message(state, is_online);
                }

                // Send button
                let send_enabled = !state.message_input.trim().is_empty() && !state.is_sending_message;

                ui.add_enabled_ui(send_enabled, |ui| {
                    if ui.button("➤").clicked() {
                        tracing::info!("[BRAID] Send button clicked, calling send_message");
                        send_message(state, is_online);
                    }
                });
            });
        });
}

/// Send the current message
fn send_message(state: &mut MessagingState, is_online: bool) {
    tracing::info!("[BRAID] send_message called with content length: {}, is_online: {}", state.message_input.len(), is_online);
    let content = state.message_input.trim().to_string();
    if content.is_empty() {
        tracing::info!("[BRAID] Message content is empty, not sending");
        return;
    }
    tracing::info!("[BRAID] Sending message: '{}'", content);

    // Get the selected conversation
    let conversation_id = match state.selected_conversation_id {
        Some(id) => {
            tracing::info!("[BRAID] Selected conversation: {}", id);
            id
        },
        None => {
            tracing::warn!("[BRAID] No conversation selected, cannot send message");
            return;
        }
    };

    // Set sending state
    state.is_sending_message = true;

    if is_online {
        // Online: Send immediately via Braid-HTTP
        tracing::info!("[BRAID] Attempting to send message online");
        let message_sync_client = match state.message_sync_client.as_mut() {
            Some(client) => {
                tracing::info!("[BRAID] Message sync client available, sending message");
                client
            },
            None => {
                tracing::error!("[BRAID] No message sync client available!");
                state.is_sending_message = false;
                return;
            }
        };

        // Ensure we are subscribed before sending so the other client receives updates and
        // this client also processes the server-echoed message when it arrives.
        if state.last_subscribed_conversation_id != Some(conversation_id) {
            tracing::info!("[BRAID] Not subscribed to {}, subscribing now before send", conversation_id);
            message_sync_client.subscribe_to_conversation(conversation_id);
            state.last_subscribed_conversation_id = Some(conversation_id);
        }

        match message_sync_client.send_message(conversation_id, content.clone(), None) {
            Ok((message_id, version)) => {
                tracing::info!("[BRAID] Message sent successfully: id={}, version={}", message_id, version);
                // Create a new message and add it to the conversation
                let new_message = crate::shared::messaging::ChatMessage {
                    id: message_id,
                    conversation_id,
                    sender_id: state.current_user_id.unwrap_or_else(|| uuid::Uuid::nil()),
                    content,
                    message_type: crate::shared::messaging::MessageType::Text,
                    #[cfg(feature = "ssr")]
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    #[cfg(not(feature = "ssr"))]
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    is_read: false,
                    is_delivered: true,
                    crdt_timestamp: 0, // TODO: Implement proper CRDT timestamp
                    braid_version: version,
                    braid_parents: Vec::new(),
                    version_vector: crate::shared::messaging::message::VersionVector::default(),
                };

                // Add to messages map
                state.messages.entry(conversation_id)
                    .or_insert_with(Vec::new)
                    .push(new_message);

                // Clear input
                state.message_input.clear();
            }
            Err(e) => {
                // Network error - queue for later
                tracing::warn!("[BRAID] Failed to send message online, queuing for offline: {}", e);
                // Surface common auth/network issues to the UI so users see why nothing happens
                if e.contains("Not authenticated") || e.contains("401") || e.contains("UNAUTHORIZED") {
                    state.ui_error = Some("You are not authenticated. Please login in this window.".to_string());
                } else if e.contains("FORBIDDEN") || e.contains("403") {
                    state.ui_error = Some("You are not a participant in this conversation.".to_string());
                } else {
                    state.ui_error = Some(format!("Failed to send message: {}", e));
                }
                queue_message_offline(state, conversation_id, content);
            }
        }
    } else {
        // Offline: Queue for later sending
        queue_message_offline(state, conversation_id, content);
    }

    // Reset sending state
    state.is_sending_message = false;
}

/// Queue a message for offline sending
fn queue_message_offline(state: &mut MessagingState, conversation_id: uuid::Uuid, content: String) {
    // Create a message for offline queuing
    let offline_message = crate::shared::messaging::ChatMessage {
        id: uuid::Uuid::new_v4(),
        conversation_id,
        sender_id: state.current_user_id.unwrap_or_else(|| uuid::Uuid::nil()),
        content: content.clone(),
        message_type: crate::shared::messaging::MessageType::Text,
        #[cfg(feature = "ssr")]
        timestamp: chrono::Utc::now().to_rfc3339(),
        #[cfg(not(feature = "ssr"))]
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_read: false,
        is_delivered: false, // Mark as not delivered yet
        crdt_timestamp: 0,
        braid_version: "pending".to_string(),
        braid_parents: Vec::new(),
        version_vector: crate::shared::messaging::message::VersionVector::default(),
    };

    // Add to offline queue
    state.queue_message_offline(offline_message.clone());

    // Also add to UI with pending status for immediate feedback
    state.messages.entry(conversation_id)
        .or_insert_with(Vec::new)
        .push(offline_message);

    // Clear input
    state.message_input.clear();

    tracing::info!("[BRAID] Message queued for offline sending: {}", content);
}

