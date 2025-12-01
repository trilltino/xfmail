//! Main Messaging Layout
//!
//! This module contains the main layout for the messaging view,
//! with a sidebar on the left and chat area on the right.

use eframe::egui;
use std::sync::mpsc::channel;
use super::state::MessagingState;
use super::sidebar::render_sidebar;
use super::chat_area::render_chat_area;
use super::friend_api::FriendApiClient;
use super::braid_sync::MessageSyncClient;
use crate::egui_app::config::Config;
use crate::egui_app::theme::styles;

/// Sidebar width in pixels
const SIDEBAR_WIDTH: f32 = 320.0;

/// Render the main messaging view
pub fn render_messaging_view(ui: &mut egui::Ui, state: &mut MessagingState, config: &Config) {
    // Check for pending async operation results
    state.check_pending_operations();

    // Initialize data on first render
    if !state.initialized {
        state.initialized = true;
        load_initial_data(state, config);
    }

    // Subscribe to selected conversation (only when selection changes) and poll for messages
    if let Some(conv_id) = state.selected_conversation_id {
        if state.last_subscribed_conversation_id != Some(conv_id) {
            tracing::info!(
                "[BRAID] Selected conversation changed to {} – subscribing once",
                conv_id
            );
            if let Some(ref mut client) = state.message_sync_client {
                client.subscribe_to_conversation(conv_id);
                state.last_subscribed_conversation_id = Some(conv_id);
            } else {
                tracing::error!("[BRAID] No message sync client available!");
            }
        }

        // Poll for incoming messages
        if let Some(ref mut client) = state.message_sync_client {
            let incoming = client.poll_messages();
            if !incoming.is_empty() {
                tracing::info!("[BRAID] Received {} messages via polling", incoming.len());
                for msg in incoming {
                    tracing::info!("[BRAID] UI updating with received message: id={}, sender={}, content='{}...', version={}, parents={:?}",
                                  msg.id, msg.sender_id, &msg.content[..msg.content.len().min(30)], msg.braid_version, msg.braid_parents);
                    state.messages.entry(conv_id)
                        .or_insert_with(Vec::new)
                        .push(msg);
                    tracing::debug!(
                        "[BRAID] Message added to UI state, total messages in conversation: {}",
                        state.messages.get(&conv_id).map(|v| v.len()).unwrap_or(0)
                    );
                }
            }

            // Poll subscription status updates and reflect in UI state + log
            if let Some(status) = client.poll_status() {
                if state.last_subscription_status.as_ref() != Some(&status) {
                    let ts = chrono::Utc::now().to_rfc3339();
                    state.subscription_log.push(format!("{} - {:?}", ts, status));
                    if state.subscription_log.len() > 200 { state.subscription_log.remove(0); }
                    state.last_subscription_status = Some(status.clone());
                }
                state.subscription_status = Some(status);
            }
        }
    } else {
        tracing::debug!("[BRAID] No conversation selected");
    }

    // Sync offline messages when online
    state.sync_offline_messages();

    // Refresh friend requests when panel is shown
    if state.show_friend_requests_panel && state.pending_load_requests.is_none() {
        refresh_friend_requests(state, config);
    }

    // Reload contacts and conversations if requested (e.g., after accepting a friend request)
    if state.should_reload_contacts && state.pending_load_contacts.is_none() {
        state.should_reload_contacts = false;
        state.contact_reload_frames = 0;
        load_contacts(state, config);
        load_conversations(state, config);
    }
    
    // Reload contacts and conversations periodically while friend requests panel is open to catch new acceptances
    if state.show_friend_requests_panel {
        state.contact_reload_frames += 1;
        if state.contact_reload_frames > 20 && state.pending_load_contacts.is_none() && state.pending_load_conversations.is_none() {
            state.contact_reload_frames = 0;
            load_contacts(state, config);
            load_conversations(state, config);
        }
    } else {
        state.contact_reload_frames = 0;
    }

    let available_size = ui.available_size();

    ui.horizontal(|ui| {
        // Left sidebar
        ui.allocate_ui_with_layout(
            egui::vec2(SIDEBAR_WIDTH, available_size.y),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                styles::sidebar_frame().show(ui, |ui| {
                    render_sidebar(ui, state, config);
                });
            },
        );

        // Separator line
        ui.add(egui::Separator::default().vertical());

        // Right chat area
        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x - SIDEBAR_WIDTH - 1.0, available_size.y),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                styles::chat_area_frame().show(ui, |ui| {
                    render_chat_area(ui, state);
                });
            },
        );
    });

    // Render modals on top
    render_modals(ui, state, config);
}

/// Load initial data (friend requests, contacts, conversations)
fn load_initial_data(state: &mut MessagingState, config: &Config) {
    // Initialize message sync client
    if state.message_sync_client.is_none() {
        tracing::info!("[BRAID] Initializing message sync client");
        state.message_sync_client = Some(MessageSyncClient::new(config.clone()));
        tracing::info!("[BRAID] Message sync client initialized successfully");
    } else {
        tracing::info!("[BRAID] Message sync client already exists");
    }

    // Load friend requests
    let config_clone = config.clone();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let client = FriendApiClient::new(config_clone);
        let result = client.get_pending_requests().map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    state.pending_load_requests = Some(rx);

    // Load contacts
    load_contacts(state, config);
    
    // Load conversations
    load_conversations(state, config);
}

/// Load or reload contacts
fn load_contacts(state: &mut MessagingState, config: &Config) {
    let config_clone = config.clone();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let client = FriendApiClient::new(config_clone);
        let result = client.get_contacts().map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    state.pending_load_contacts = Some(rx);
    state.is_loading_contacts = true;
}

/// Load or reload conversations
fn load_conversations(state: &mut MessagingState, config: &Config) {
    let config_clone = config.clone();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let client = FriendApiClient::new(config_clone);
        let result = client.get_conversations().map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    state.pending_load_conversations = Some(rx);
    state.is_loading_conversations = true;
}

/// Render modal dialogs
fn render_modals(ui: &mut egui::Ui, state: &mut MessagingState, config: &Config) {
    // Add friend modal
    if state.show_add_friend_modal {
        render_add_friend_modal(ui, state, config);
    }
}

/// Refresh friend requests
fn refresh_friend_requests(state: &mut MessagingState, config: &Config) {
    let config_clone = config.clone();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let client = FriendApiClient::new(config_clone);
        let result = client.get_pending_requests().map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    state.pending_load_requests = Some(rx);
}

/// Render the add friend modal
fn render_add_friend_modal(ui: &mut egui::Ui, state: &mut MessagingState, config: &Config) {
    egui::Window::new("Add Friend")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(300.0);

            ui.vertical(|ui| {
                ui.label("Enter your friend's email address:");
                ui.add_space(8.0);

                let enabled = !state.is_sending_friend_request;
                ui.add_enabled(enabled, egui::TextEdit::singleline(&mut state.add_friend_email));
                ui.add_space(8.0);

                ui.label("Add a message (optional):");
                ui.add_space(4.0);
                ui.add_enabled(enabled, egui::TextEdit::multiline(&mut state.add_friend_message));
                ui.add_space(8.0);

                if let Some(ref error) = state.add_friend_error {
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(8.0);
                }

                if let Some(ref success) = state.add_friend_success {
                    ui.colored_label(egui::Color32::GREEN, success);
                    ui.add_space(8.0);
                }

                if state.is_sending_friend_request {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Sending request...");
                    });
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            state.close_add_friend_modal();
                        }

                        if ui.button("Send Request").clicked() {
                            if state.add_friend_email.is_empty() {
                                state.add_friend_error = Some("Please enter an email address".to_string());
                            } else if !state.add_friend_email.contains('@') {
                                state.add_friend_error = Some("Please enter a valid email address".to_string());
                            } else {
                                // Send the friend request
                                state.is_sending_friend_request = true;
                                state.add_friend_error = None;
                                state.add_friend_success = None;

                                let email = state.add_friend_email.clone();
                                let config_clone = config.clone();
                                let (tx, rx) = channel();

                                std::thread::spawn(move || {
                                    let client = FriendApiClient::new(config_clone);
                                    let result = client.send_friend_request(&email);
                                    let mapped_result = match result {
                                        Ok(resp) if resp.success => Ok(()),
                                        Ok(resp) => Err(resp.error.unwrap_or("Unknown error".to_string())),
                                        Err(e) => Err(e),
                                    };
                                    let _ = tx.send(mapped_result);
                                });

                                state.pending_send_request = Some(rx);
                            }
                        }
                    });
                }
            });
        });
}

