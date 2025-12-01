//! Chat Area Component
//!
//! This module contains the main chat area with message list and input bar.

use eframe::egui;
use super::state::MessagingState;
use super::components::{chat_header, message_list, input_bar};
use crate::egui_app::theme::colors;

/// Render the chat area
pub fn render_chat_area(ui: &mut egui::Ui, state: &mut MessagingState) {
    // Show any transient UI error banner
    if let Some(err) = state.ui_error.take() {
        ui.add_space(6.0);
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 238, 238))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 80, 80)))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.colored_label(egui::Color32::from_rgb(160, 20, 20), format!("{}", err));
            });
        ui.add_space(6.0);
    }

    if state.selected_conversation_id.is_some() {
        render_active_chat(ui, state);
    } else {
        render_empty_state(ui);
    }
}

/// Render an active chat conversation
fn render_active_chat(ui: &mut egui::Ui, state: &mut MessagingState) {
    ui.vertical(|ui| {
        // Chat header
        chat_header::render(ui, state);
        
        ui.add(egui::Separator::default().horizontal());
        
        // Message list (takes remaining space)
        let available_height = ui.available_height() - 60.0; // Reserve space for input bar
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), available_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                message_list::render(ui, state);
            },
        );
        
        // Input bar at bottom
        input_bar::render(ui, state, true); // TODO: Pass actual online status
    });
}

/// Render the empty state when no conversation is selected
fn render_empty_state(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);

            // Text bubble
            egui::Frame::new()
                .fill(colors::BUBBLE_INCOMING)
                .stroke(egui::Stroke::new(1.0, colors::BUBBLE_BORDER))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Select a message to start typing").color(colors::TEXT_DARK));
                });
        });
    });
}

