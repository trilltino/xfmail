//! Chat Header Component
//!
//! Displays the header of the chat area with contact info and actions.

use eframe::egui;
use crate::egui_app::messaging::state::MessagingState;
use crate::egui_app::theme::colors;
use crate::egui_app::messaging::braid_sync::SubscriptionStatus;

/// Render the chat header
pub fn render(ui: &mut egui::Ui, state: &mut MessagingState) {
    let conversation = match state.selected_conversation() {
        Some(conv) => conv,
        None => return,
    };
    
    // Find the other participant's contact info
    let other_contact = state.contacts.iter()
        .find(|c| conversation.participants.contains(&c.contact_user_id));
    
    egui::Frame::new()
        .fill(colors::CHAT_HEADER_BG)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            
            ui.horizontal(|ui| {
                // Avatar
                if let Some(contact) = other_contact {
                    let avatar_text = contact.username.chars().next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_else(|| "?".to_string());
                    
                    egui::Frame::new()
                        .fill(colors::ACCENT)
                        .corner_radius(egui::CornerRadius::same(18))
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(avatar_text).color(egui::Color32::WHITE).strong());
                        });
                    
                    ui.add_space(8.0);
                    
                    // Contact info
                    ui.vertical(|ui| {
                        let display_name = contact.display_name.as_ref()
                            .unwrap_or(&contact.username);
                        ui.label(egui::RichText::new(display_name).strong().size(16.0));
                        ui.colored_label(colors::TEXT_SECONDARY, &contact.email);
                    });
                } else {
                    ui.label(egui::RichText::new("Unknown Contact").strong());
                }
                
                // Right side actions
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let menu_button = ui.button("⋮");
                    if menu_button.clicked() {
                        state.show_chat_header_menu = !state.show_chat_header_menu;
                    }

                    ui.add_space(8.0);
                    // Toggle connection log window
                    if ui.button("Logs").on_hover_text("Show connection log").clicked() {
                        state.show_connection_log = !state.show_connection_log;
                    }
                    if state.show_connection_log {
                        let mut open = true;
                        egui::Window::new("Connection Log")
                            .open(&mut open)
                            .collapsible(true)
                            .resizable(true)
                            .default_size(egui::vec2(420.0, 260.0))
                            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                            .show(ui.ctx(), |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    for line in state.subscription_log.iter().rev() {
                                        ui.label(egui::RichText::new(line).color(colors::TEXT_SECONDARY));
                                    }
                                });
                            });
                        if !open { state.show_connection_log = false; }
                    }
                    ui.add_space(8.0);
                    // Subscription status pill
                    let (label, color) = match state.subscription_status.clone() {
                        Some(SubscriptionStatus::Connected) => ("Connected", egui::Color32::from_rgb(22, 163, 74)),
                        Some(SubscriptionStatus::Retrying) => ("Retrying", egui::Color32::from_rgb(234, 179, 8)),
                        Some(SubscriptionStatus::Connecting) => ("Connecting", egui::Color32::from_rgb(59, 130, 246)),
                        Some(SubscriptionStatus::Error(_)) => ("Error", egui::Color32::from_rgb(220, 38, 38)),
                        Some(SubscriptionStatus::Disconnected) => ("Disconnected", egui::Color32::from_rgb(107, 114, 128)),
                        None => ("—", egui::Color32::from_rgb(107, 114, 128)),
                    };
                    egui::Frame::new()
                        .fill(color.linear_multiply(0.15))
                        .stroke(egui::Stroke::new(1.0, color))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(label).color(color).strong());
                        });

                    // Show menu popup
                    if state.show_chat_header_menu {
                        let mut open = true;
                        egui::Window::new("Chat Menu")
                            .open(&mut open)
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 30.0])
                            .show(ui.ctx(), |ui| {
                                ui.set_min_width(150.0);
                                ui.vertical(|ui| {
                                    if ui.button("View Profile").clicked() {
                                        state.show_chat_header_menu = false;
                                        // TODO: Implement profile view
                                    }
                                    if ui.button("Block User").clicked() {
                                        state.show_chat_header_menu = false;
                                        // TODO: Implement block functionality
                                    }
                                    if ui.button("Clear Chat").clicked() {
                                        state.show_chat_header_menu = false;
                                        // TODO: Implement clear chat functionality
                                    }
                                });
                            });
                        if !open {
                            state.show_chat_header_menu = false;
                        }
                    }
                });
            });
        });
}

