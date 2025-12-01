use crate::egui_app::state::AppState;
use crate::egui_app::debug::DebugCategory;

pub fn render_debug_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.heading("🐛 Deep Debug Console");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(if state.debug_view_expanded {
                "⬇ Collapse"
            } else {
                "⬆ Expand"
            }).clicked() {
                state.debug_view_expanded = !state.debug_view_expanded;
            }

            ui.label(format!("Entries: {}", state.debug_logger.count()));

            if ui.button("🗑 Clear Logs").clicked() {
                state.debug_logger.clear();
            }

            ui.separator();

            ui.label("Filter:");
            let categories = vec![
                ("All", None),
                ("Network", Some(DebugCategory::Network)),
                ("Sync", Some(DebugCategory::Sync)),
                ("State", Some(DebugCategory::State)),
                ("Auth", Some(DebugCategory::Auth)),
                ("Peer", Some(DebugCategory::Peer)),
                ("Email", Some(DebugCategory::Email)),
                ("Thread", Some(DebugCategory::Thread)),
            ];

            for (label, category) in categories {
                if ui.selectable_label(state.debug_filter_category == category, label).clicked() {
                    state.debug_filter_category = category;
                }
            }
        });

        ui.separator();

        let entries = if let Some(ref cat) = state.debug_filter_category {
            state.debug_logger.get_entries_by_category(cat.clone())
        } else {
            state.debug_logger.get_entries()
        };

        let show_height = if state.debug_view_expanded { 400.0 } else { 150.0 };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(show_height)
            .show(ui, |ui| {
                for entry in entries.iter().rev().take(if state.debug_view_expanded { 500 } else { 50 }) {
                    let color = match entry.level {
                        crate::egui_app::debug::DebugLevel::Error => egui::Color32::RED,
                        crate::egui_app::debug::DebugLevel::Warn => egui::Color32::YELLOW,
                        crate::egui_app::debug::DebugLevel::Info => egui::Color32::GREEN,
                        crate::egui_app::debug::DebugLevel::Debug => egui::Color32::LIGHT_GRAY,
                        crate::egui_app::debug::DebugLevel::Trace => egui::Color32::DARK_GRAY,
                    };

                    ui.colored_label(color, entry.to_string());
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Stats:");

            let entries = state.debug_logger.get_entries();
            let errors = state
                .debug_logger
                .get_entries_by_level(crate::egui_app::debug::DebugLevel::Error)
                .len();
            let warns = state
                .debug_logger
                .get_entries_by_level(crate::egui_app::debug::DebugLevel::Warn)
                .len();
            let infos = state
                .debug_logger
                .get_entries_by_level(crate::egui_app::debug::DebugLevel::Info)
                .len();

            ui.colored_label(egui::Color32::RED, format!("❌ Errors: {}", errors));
            ui.colored_label(egui::Color32::YELLOW, format!("⚠️ Warnings: {}", warns));
            ui.colored_label(egui::Color32::GREEN, format!("ℹ️ Info: {}", infos));
            ui.label(format!("Total: {}", entries.len()));
        });
    });
}
