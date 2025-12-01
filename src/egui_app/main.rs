/**
 * egui Native Desktop App - Main Entry Point
 *
 * This is the main entry point for the egui native desktop application.
 * It implements eframe::App and provides the UI for authentication and demo selection.
 */
use eframe::egui;
use xfmail::egui_app::{AppState, views};

/// Configure custom font (Roboto Condensed Black)
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Load Roboto Condensed Black font
    fonts.font_data.insert(
        "RobotoCondensed".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "assets/Roboto_Condensed-Black.ttf"
        ))),
    );

    // Set as highest priority for Proportional (regular text)
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "RobotoCondensed".to_owned());

    // Also use for Monospace
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "RobotoCondensed".to_owned());

    ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "XFChat - Messaging",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(BraidApp::default()))
        }),
    )
}

/// Main application state
struct BraidApp {
    state: AppState,
}

impl Default for BraidApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.check_auth_result();

        views::render_top_bar(ctx, &mut self.state, frame);

        views::render_main_panel(ctx, &mut self.state);

        ctx.request_repaint();
    }
}
