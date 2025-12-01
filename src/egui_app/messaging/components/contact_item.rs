//! Contact Item Component
//!
//! A single contact item in the contact list showing username, last message preview, and time.

use eframe::egui;
use crate::shared::messaging::{Contact, ChatMessage};
use crate::egui_app::theme::colors;

/// Render a single contact item
/// Returns true if the item was clicked
pub fn render(
    ui: &mut egui::Ui,
    contact: &Contact,
    last_message: Option<&ChatMessage>,
    is_selected: bool,
) -> bool {
    let mut clicked = false;

    // Background color based on selection
    let bg_color = if is_selected {
        colors::SELECTED_ITEM
    } else {
        colors::CHAT_LIST_BG
    };

    let response = egui::Frame::new()
        .fill(bg_color)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                // Avatar placeholder (first letter of username)
                let avatar_text = contact.username.chars().next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_else(|| "?".to_string());

                egui::Frame::new()
                    .fill(colors::ACCENT)
                    .corner_radius(egui::CornerRadius::same(20))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(avatar_text).color(egui::Color32::WHITE).strong());
                    });

                ui.add_space(8.0);

                // Contact info
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Username
                        let display_name = contact.display_name.as_ref()
                            .unwrap_or(&contact.username);
                        ui.label(egui::RichText::new(display_name).strong());

                        // Time of last message
                        if let Some(msg) = last_message {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let time_str = format_time(&msg.timestamp);
                                ui.colored_label(colors::TEXT_SECONDARY, time_str);
                            });
                        }
                    });

                    // Last message preview
                    if let Some(msg) = last_message {
                        let preview = truncate_message(&msg.content, 40);
                        ui.colored_label(colors::TEXT_SECONDARY, preview);
                    } else {
                        ui.colored_label(colors::TEXT_SECONDARY, "No messages yet");
                    }
                });
            });
        });

    // Check if the entire frame was clicked
    if response.response.interact(egui::Sense::click()).clicked() {
        clicked = true;
    }

    // Add hover effect
    if response.response.hovered() && !is_selected {
        ui.painter().rect_filled(
            response.response.rect,
            egui::CornerRadius::ZERO,
            colors::HOVER_ITEM,
        );
    }

    clicked
}

/// Format timestamp for display (RFC3339 string -> HH:MM)
fn format_time(timestamp: &str) -> String {
    // Try to parse the timestamp, otherwise just return a shortened version
    // The timestamp is in ISO 8601 format
    if timestamp.len() >= 16 {
        // Extract time portion (HH:MM)
        if let Some(t_pos) = timestamp.find('T') {
            let time_part = &timestamp[t_pos + 1..];
            if time_part.len() >= 5 {
                return time_part[..5].to_string();
            }
        }
    }
    timestamp.to_string()
}

/// Truncate message for preview
fn truncate_message(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}

