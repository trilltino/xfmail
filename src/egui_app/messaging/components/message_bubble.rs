//! Message Bubble Component
//!
//! Displays a single message bubble with content and timestamp.

use eframe::egui;
use crate::shared::messaging::ChatMessage;
use crate::egui_app::theme::colors;

/// Render a message bubble
pub fn render(ui: &mut egui::Ui, message: &ChatMessage, is_own_message: bool) {
    let (bg_color, text_color, align) = if is_own_message {
        (colors::BUBBLE_OUTGOING, colors::TEXT_PRIMARY, egui::Align::RIGHT)
    } else {
        (colors::BUBBLE_INCOMING, colors::TEXT_PRIMARY, egui::Align::LEFT)
    };

    ui.with_layout(egui::Layout::top_down(align), |ui| {
        // Limit bubble width
        let max_width = ui.available_width() * 0.7;

        ui.allocate_ui_with_layout(
            egui::vec2(max_width, 0.0),
            egui::Layout::top_down(align),
            |ui| {
                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(egui::CornerRadius {
                        nw: if is_own_message { 12 } else { 4 },
                        ne: if is_own_message { 4 } else { 12 },
                        sw: 12,
                        se: 12,
                    })
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        // Message content
                        ui.label(egui::RichText::new(&message.content).color(text_color));

                        // Timestamp and status
                        ui.horizontal(|ui| {
                            let time_str = format_time(&message.timestamp);
                            ui.colored_label(colors::TEXT_SECONDARY, time_str);

                            if is_own_message {
                                // Delivery status
                                let status_icon = if message.is_delivered {
                                    "✓✓"
                                } else {
                                    "✓"
                                };
                                ui.colored_label(
                                    if message.is_read { colors::ACCENT } else { colors::TEXT_SECONDARY },
                                    status_icon,
                                );
                            }
                        });
                    });
            },
        );
    });

    ui.add_space(4.0);
}

/// Format timestamp string to display time (HH:MM)
fn format_time(timestamp: &str) -> String {
    // Timestamp format: "2024-01-15T10:30:00Z"
    if let Some(t_pos) = timestamp.find('T') {
        let time_part = &timestamp[t_pos + 1..];
        if time_part.len() >= 5 {
            return time_part[..5].to_string();
        }
    }
    timestamp.to_string()
}

