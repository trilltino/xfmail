use eframe::egui;

use crate::egui_app::AppView;
use crate::egui_app::state::AppState;
use crate::egui_app::theme::colors;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let frame = egui::Frame::default()
        .fill(colors::BG_DARK);

    frame.show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);

            // App logo/title
            ui.colored_label(colors::TEXT_LIGHT, egui::RichText::new("💬 XFChat").size(48.0).strong());
            ui.add_space(10.0);

            // Welcome message
            ui.colored_label(colors::TEXT_LIGHT, egui::RichText::new("Welcome!").size(28.0));
            if let Some(ref user) = state.auth_state.user {
                ui.colored_label(colors::ICONS, egui::RichText::new(format!("@{}", user.username)).size(18.0));
            }
            ui.add_space(40.0);

            ui.colored_label(colors::TEXT_LIGHT, egui::RichText::new("Select an Application:").size(18.0));
            ui.add_space(20.0);

            // Messaging button - larger and styled
            let messaging_btn = egui::Button::new(
                egui::RichText::new("💬 Messaging").size(20.0)
            )
            .min_size(egui::vec2(200.0, 50.0))
            .fill(colors::BUTTON_PRIMARY);

            if ui.add(messaging_btn).clicked() {
                state.current_view = AppView::Messaging;
            }
            ui.add_space(15.0);

            // XFCollab button
            let collab_btn = egui::Button::new(
                egui::RichText::new("📝 XFCollab").size(20.0)
            )
            .min_size(egui::vec2(200.0, 50.0))
            .fill(colors::BUTTON_SECONDARY);

            if ui.add(collab_btn).clicked() {
                state.current_view = AppView::XFCollab;
            }
        });
    });
}

