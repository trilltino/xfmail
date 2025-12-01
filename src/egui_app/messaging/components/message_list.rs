//! Message List Component
//!
//! Displays the list of messages in a conversation.

use eframe::egui;
use crate::egui_app::messaging::state::MessagingState;
use crate::egui_app::theme::colors;
use super::message_bubble;

/// Render the message list
pub fn render(ui: &mut egui::Ui, state: &MessagingState) {
    let messages = match state.selected_messages() {
        Some(msgs) => msgs,
        None => return,
    };

    let current_user_id = state.current_user_id;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            if messages.is_empty() {
                render_empty_state(ui);
            } else {
                let mut last_date: Option<String> = None;

                for message in messages {
                    // Date separator - extract date from timestamp string
                    let message_date = extract_date(&message.timestamp);
                    if last_date.as_ref().map(|d| d != &message_date).unwrap_or(true) {
                        render_date_separator(ui, &message_date);
                        last_date = Some(message_date);
                    }

                    // Message bubble
                    let is_own_message = current_user_id
                        .map(|id| id == message.sender_id)
                        .unwrap_or(false);

                    message_bubble::render(ui, message, is_own_message);
                }
            }

            ui.add_space(8.0);
        });
}

/// Render empty state when no messages
fn render_empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.colored_label(colors::TEXT_SECONDARY, "No messages yet");
        ui.add_space(8.0);
        ui.colored_label(colors::TEXT_SECONDARY, "Send a message to start the conversation");
    });
}

/// Extract date portion from ISO 8601 timestamp string
fn extract_date(timestamp: &str) -> String {
    // Timestamp format: "2024-01-15T10:30:00Z"
    if let Some(t_pos) = timestamp.find('T') {
        timestamp[..t_pos].to_string()
    } else {
        timestamp.to_string()
    }
}

/// Render a date separator
fn render_date_separator(ui: &mut egui::Ui, date_str: &str) {
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.add(egui::Separator::default().horizontal());

        // Format the date nicely if possible
        let display_str = format_date_display(date_str);

        ui.colored_label(colors::TEXT_SECONDARY, display_str);

        ui.add(egui::Separator::default().horizontal());
    });

    ui.add_space(16.0);
}

/// Format date string for display
fn format_date_display(date_str: &str) -> String {
    // Simple date formatting - could be enhanced with proper date parsing
    // For now, just return the date string as-is or a friendly format
    date_str.to_string()
}

