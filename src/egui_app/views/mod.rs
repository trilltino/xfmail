use eframe::egui;

use crate::egui_app::AppView;
use crate::egui_app::state::AppState;
use crate::egui_app::theme::colors;

pub mod auth_view;
pub mod landing_view;
pub mod xfmail_view;
pub mod debug_view;

pub fn render_top_bar(ctx: &egui::Context, state: &mut AppState, frame: &mut eframe::Frame) {
    let frame_style = egui::Frame::default()
        .fill(colors::TOP_BAR_BG)
        .inner_margin(egui::Margin::symmetric(12, 8));

    egui::TopBottomPanel::top("top_panel")
        .frame(frame_style)
        .show(ctx, |ui| {
            let _frame = frame;

            ui.horizontal(|ui| {
                ui.colored_label(colors::TEXT_LIGHT, egui::RichText::new("💬 XFChat").size(18.0).strong());

                // Connectivity status indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);

                    // Show sync status if there are pending operations
                    if state.pending_sync_operations > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 193, 7), // Orange for syncing
                            format!("🔄 {} pending", state.pending_sync_operations)
                        );
                    } else if !state.is_online {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 53, 69), // Red for offline
                            "🔴 Offline"
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(40, 167, 69), // Green for online
                            "🟢 Online"
                        );
                    }

                    ui.add_space(16.0);

                    if state.auth_state.authenticated {
                        if ui.button("Logout").clicked() {
                            state.logout();
                        }
                        if let Some(ref user) = state.auth_state.user {
                            ui.colored_label(colors::TEXT_LIGHT, format!("@{}", user.username));
                        }
                    }
                });
            });
        });
}

pub fn render_main_panel(ctx: &egui::Context, state: &mut AppState) {
    let frame = egui::Frame::default()
        .fill(colors::BG_DARK)
        .inner_margin(egui::Margin::same(0));

    egui::CentralPanel::default()
        .frame(frame)
        .show(ctx, |ui| match state.current_view {
            AppView::Auth => auth_view::render(ui, state),
            AppView::Landing => landing_view::render(ui, state),
            AppView::Messaging => xfmail_view::render_messaging(ui, state),
            AppView::XFCollab => xfmail_view::render_xfcollab(ui, state),
        });
}


