//! Sidebar Component
//!
//! This module contains the sidebar with search bar, contact list, and add friend button.

use eframe::egui;
use super::state::MessagingState;
use super::components::{search_bar, contact_list, friend_requests_panel};
use crate::egui_app::config::Config;
use crate::egui_app::theme::styles;

/// Render the sidebar
pub fn render_sidebar(ui: &mut egui::Ui, state: &mut MessagingState, config: &Config) {
    ui.set_min_width(320.0);
    
    // Top section with search and add friend button
    styles::chat_list_frame().show(ui, |ui| {
        ui.add_space(8.0);
        
        // Header with title and add friend button
        ui.horizontal(|ui| {
            ui.heading("Messages");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Friend requests button with badge
                let request_count = state.pending_request_count();
                let requests_text = if request_count > 0 {
                    format!("📬 {}", request_count)
                } else {
                    "📬".to_string()
                };
                
                if ui.button(&requests_text).clicked() {
                    state.toggle_friend_requests_panel();
                }
                
                // Add friend button
                if ui.button("➕").clicked() {
                    state.open_add_friend_modal();
                }
            });
        });
        
        ui.add_space(8.0);
        
        // Search bar
        search_bar::render(ui, state);
        
        ui.add_space(8.0);
    });
    
    // Friend requests panel (if open)
    if state.show_friend_requests_panel {
        friend_requests_panel::render(ui, state, config);
    }
    
    // Contact list
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            contact_list::render(ui, state);
        });
}

