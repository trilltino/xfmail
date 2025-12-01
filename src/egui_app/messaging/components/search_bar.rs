//! Search Bar Component
//!
//! A search bar for filtering contacts by email or username.

use eframe::egui;
use crate::egui_app::messaging::state::MessagingState;

/// Render the search bar
pub fn render(ui: &mut egui::Ui, state: &mut MessagingState) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // Search icon
        ui.label("🔍");

        // Search input
        let _response = ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text("Search contacts...")
                .desired_width(ui.available_width() - 40.0)
        );

        // Clear button
        if !state.search_query.is_empty() {
            if ui.button("✕").clicked() {
                state.search_query.clear();
            }
        }

        ui.add_space(8.0);
    });
}

