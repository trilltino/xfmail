use eframe::egui;

use crate::egui_app::state::AppState;
use crate::egui_app::theme::colors;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // Fill the entire background first
    let available_rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(available_rect, 0.0, colors::BG_DARK);

    // Center the content vertically and horizontally
    ui.scope_builder(egui::UiBuilder::new().max_rect(available_rect), |ui| {
        ui.vertical_centered(|ui| {
            // Calculate vertical centering
            let total_height = if state.is_signup_mode { 350.0 } else { 280.0 };
            let top_space = (available_rect.height() - total_height).max(0.0) / 2.0;
            ui.add_space(top_space);

            // App title
            ui.label(egui::RichText::new("💬 XFChat").size(32.0).strong().color(colors::TEXT_LIGHT));
            ui.add_space(20.0);

            ui.label(
                egui::RichText::new(if state.is_signup_mode { "Create Account" } else { "Welcome Back" })
                    .size(24.0)
                    .color(colors::TEXT_LIGHT)
            );
            ui.add_space(20.0);

            if let Some(ref error) = state.auth_state.error {
                ui.label(egui::RichText::new(error).color(colors::ERROR));
                ui.add_space(10.0);
            }

            // Styled text inputs
            let input_width = 280.0;
            let label_width = 80.0;

            // Username field
            ui.horizontal(|ui| {
                ui.add_space((available_rect.width() - input_width - label_width - 20.0) / 2.0);
                ui.add_sized([label_width, 24.0], egui::Label::new(
                    egui::RichText::new("Username:").color(colors::TEXT_SECONDARY)
                ));
                ui.add_sized([input_width, 28.0], egui::TextEdit::singleline(&mut state.username_input)
                    .text_color(colors::TEXT_LIGHT));
            });
            ui.add_space(8.0);

            // Email field only for signup
            if state.is_signup_mode {
                ui.horizontal(|ui| {
                    ui.add_space((available_rect.width() - input_width - label_width - 20.0) / 2.0);
                    ui.add_sized([label_width, 24.0], egui::Label::new(
                        egui::RichText::new("Email:").color(colors::TEXT_SECONDARY)
                    ));
                    ui.add_sized([input_width, 28.0], egui::TextEdit::singleline(&mut state.email_input)
                        .text_color(colors::TEXT_LIGHT));
                });
                ui.add_space(8.0);
            }

            // Password field
            ui.horizontal(|ui| {
                ui.add_space((available_rect.width() - input_width - label_width - 20.0) / 2.0);
                ui.add_sized([label_width, 24.0], egui::Label::new(
                    egui::RichText::new("Password:").color(colors::TEXT_SECONDARY)
                ));
                ui.add_sized([input_width, 28.0], egui::TextEdit::singleline(&mut state.password_input)
                    .password(true)
                    .text_color(colors::TEXT_LIGHT));
            });
            ui.add_space(8.0);

            if state.is_signup_mode {
                ui.horizontal(|ui| {
                    ui.add_space((available_rect.width() - input_width - label_width - 20.0) / 2.0);
                    ui.add_sized([label_width, 24.0], egui::Label::new(
                        egui::RichText::new("Confirm:").color(colors::TEXT_SECONDARY)
                    ));
                    ui.add_sized([input_width, 28.0], egui::TextEdit::singleline(&mut state.confirm_password_input)
                        .password(true)
                        .text_color(colors::TEXT_LIGHT));
                });
                ui.add_space(8.0);
            }

            ui.add_space(20.0);

            // Buttons centered
            ui.horizontal(|ui| {
                let button_width = 120.0;
                let total_buttons_width = button_width * 2.0 + 10.0;
                ui.add_space((available_rect.width() - total_buttons_width) / 2.0);

                if ui.add_sized([button_width, 32.0], egui::Button::new(
                    egui::RichText::new(if state.is_signup_mode { "Sign Up" } else { "Login" }).color(colors::TEXT_LIGHT)
                ).fill(colors::ACCENT)).clicked() {
                    state.auth_state.clear_error();
                    if state.is_signup_mode {
                        state.handle_signup();
                    } else {
                        state.handle_login();
                    }
                }

                ui.add_space(10.0);

                if ui.add_sized([button_width, 32.0], egui::Button::new(
                    egui::RichText::new(if state.is_signup_mode { "Back to Login" } else { "Create Account" }).color(colors::TEXT_SECONDARY)
                )).clicked() {
                    state.toggle_auth_mode();
                }
            });

            if state.auth_state.loading {
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.add_space((available_rect.width() - 100.0) / 2.0);
                    ui.label(egui::RichText::new("Loading...").color(colors::TEXT_LIGHT));
                    ui.spinner();
                });
            }
        });
    });
}

