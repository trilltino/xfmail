use eframe::egui;

use crate::egui_app::AppView;
use crate::egui_app::state::AppState;
use crate::egui_app::messaging;

/// Render the Telegram-style messaging view
pub fn render_messaging(ui: &mut egui::Ui, state: &mut AppState) {
    // Render the main messaging layout with config for API access
    messaging::main_layout::render_messaging_view(ui, &mut state.messaging_state, &state.config);
}

/// Render the XFCollab collaborative editing view
pub fn render_xfcollab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        if ui.button("← Back to Home").clicked() {
            state.current_view = AppView::Landing;
        }
        ui.separator();
        ui.add_space(50.0);

        ui.heading("XFCollab");
        ui.add_space(20.0);
        ui.label("Collaborative editing application coming soon...");
    });
}

