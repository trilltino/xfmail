//! Friend Requests Panel Component
//!
//! Displays incoming and outgoing friend requests.

use eframe::egui;
use std::sync::mpsc::channel;
use uuid::Uuid;
use crate::egui_app::config::Config;
use crate::egui_app::messaging::friend_api::FriendApiClient;
use crate::egui_app::messaging::state::MessagingState;
use crate::egui_app::theme::colors;

/// Actions from the friend requests panel
pub enum FriendRequestAction {
    Accept(Uuid),
    Reject(Uuid),
    Cancel(Uuid),
}

/// Render the friend requests panel
pub fn render(ui: &mut egui::Ui, state: &mut MessagingState, config: &Config) {
    egui::Frame::new()
        .fill(colors::CHAT_LIST_BG)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            
            // Header
            ui.horizontal(|ui| {
                ui.heading("Friend Requests");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        state.show_friend_requests_panel = false;
                    }
                });
            });
            
            ui.add_space(8.0);
            ui.add(egui::Separator::default().horizontal());
            ui.add_space(8.0);
            
            // Collect actions to process after UI rendering
            let mut action: Option<FriendRequestAction> = None;

            // Incoming requests
            if !state.incoming_friend_requests.is_empty() {
                ui.label(egui::RichText::new("Incoming").strong());
                ui.add_space(4.0);

                let requests = state.incoming_friend_requests.clone();
                for request in &requests {
                    if let Some(a) = render_incoming_request(ui, request, state) {
                        action = Some(a);
                    }
                }

                ui.add_space(8.0);
            }

            // Outgoing requests
            if !state.outgoing_friend_requests.is_empty() {
                ui.label(egui::RichText::new("Sent").strong());
                ui.add_space(4.0);

                let requests = state.outgoing_friend_requests.clone();
                for request in &requests {
                    if let Some(a) = render_outgoing_request(ui, request) {
                        action = Some(a);
                    }
                }
            }

            // Process actions
            if let Some(action) = action {
                process_action(action, state, config);
            }
            
            // Empty state
            if state.incoming_friend_requests.is_empty() && state.outgoing_friend_requests.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.colored_label(colors::TEXT_SECONDARY, "No pending requests");
                });
            }
        });
}

/// Render an incoming friend request - returns action if button clicked
fn render_incoming_request(
    ui: &mut egui::Ui,
    request: &crate::shared::messaging::FriendRequest,
    _state: &mut MessagingState,
) -> Option<FriendRequestAction> {
    let mut action = None;
    let request_id = request.id;

    egui::Frame::new()
        .fill(colors::SIDEBAR_BG)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&request.from_username).strong());
                    ui.colored_label(colors::TEXT_SECONDARY, &request.from_email);
                    if let Some(ref msg) = request.message {
                        ui.colored_label(colors::TEXT_SECONDARY, format!("\"{}\"", msg));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reject").clicked() {
                        action = Some(FriendRequestAction::Reject(request_id));
                    }
                    if ui.button("Accept").clicked() {
                        action = Some(FriendRequestAction::Accept(request_id));
                    }
                });
            });
        });

    ui.add_space(4.0);
    action
}

/// Render an outgoing friend request - returns action if button clicked
fn render_outgoing_request(
    ui: &mut egui::Ui,
    request: &crate::shared::messaging::FriendRequest,
) -> Option<FriendRequestAction> {
    let mut action = None;
    let request_id = request.id;

    egui::Frame::new()
        .fill(colors::SIDEBAR_BG)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&request.to_email).strong());
                    ui.colored_label(colors::TEXT_SECONDARY, "Pending...");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(FriendRequestAction::Cancel(request_id));
                    }
                });
            });
        });

    ui.add_space(4.0);
    action
}

/// Process a friend request action
fn process_action(action: FriendRequestAction, state: &mut MessagingState, config: &Config) {
    match action {
        FriendRequestAction::Accept(request_id) => {
            // Don't start another accept if one is pending
            if state.pending_accept_request.is_some() {
                return;
            }

            let config_clone = config.clone();
            let (tx, rx) = channel();

            std::thread::spawn(move || {
                let client = FriendApiClient::new(config_clone);
                let result = client.respond_to_request(request_id, true)
                    .map(|_| ())
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });

            state.pending_accept_request = Some((request_id, rx));
        }
        FriendRequestAction::Reject(request_id) => {
            // Don't start another reject if one is pending
            if state.pending_reject_request.is_some() {
                return;
            }

            let config_clone = config.clone();
            let (tx, rx) = channel();

            std::thread::spawn(move || {
                let client = FriendApiClient::new(config_clone);
                let result = client.respond_to_request(request_id, false)
                    .map(|_| ())
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });

            state.pending_reject_request = Some((request_id, rx));
        }
        FriendRequestAction::Cancel(request_id) => {
            // For cancel, we treat it as a reject from sender side
            // The API should handle this - for now just remove locally
            state.outgoing_friend_requests.retain(|r| r.id != request_id);
        }
    }
}

