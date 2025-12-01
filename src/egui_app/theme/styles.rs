//! Theme Styling Functions
//!
//! This module provides helper functions for applying the brown color scheme
//! consistently across all UI components.

use eframe::egui::{self, Color32, CornerRadius, Stroke};
use super::colors;

/// Apply the global theme to the egui context
pub fn apply_global_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Window styling
    style.visuals.window_fill = colors::MAIN_CHAT_BG;
    style.visuals.window_stroke = Stroke::new(1.0, colors::BUBBLE_BORDER);

    // Panel styling
    style.visuals.panel_fill = colors::SIDEBAR_BG;

    // Widget styling
    style.visuals.widgets.noninteractive.bg_fill = colors::INPUT_BAR_BG;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_DARK);

    style.visuals.widgets.inactive.bg_fill = colors::INPUT_BAR_BG;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_DARK);

    style.visuals.widgets.hovered.bg_fill = colors::CHAT_LIST_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::TEXT_LIGHT);

    style.visuals.widgets.active.bg_fill = colors::BUTTON_PRIMARY;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::TEXT_LIGHT);

    // Selection color
    style.visuals.selection.bg_fill = colors::ACTIVE_CHAT_STRIP;
    style.visuals.selection.stroke = Stroke::new(1.0, colors::TEXT_LIGHT);

    ctx.set_style(style);
}

/// Create a frame style for the sidebar
pub fn sidebar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::SIDEBAR_BG)
        .inner_margin(egui::Margin::same(0))
}

/// Create a frame style for the chat list panel
pub fn chat_list_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::CHAT_LIST_BG)
        .inner_margin(egui::Margin::same(0))
}

/// Create a frame style for the main chat area
pub fn chat_area_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::BG_DARK)
        .inner_margin(egui::Margin::same(0))
}

/// Create a frame style for the top bar
pub fn top_bar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::TOP_BAR_BG)
        .inner_margin(egui::Margin::symmetric(12, 8))
}

/// Create a frame style for the input bar
pub fn input_bar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::INPUT_BAR_BG)
        .stroke(Stroke::new(1.0, colors::INPUT_BAR_BORDER))
        .inner_margin(egui::Margin::symmetric(12, 8))
}

/// Create a frame style for outgoing message bubbles
pub fn outgoing_bubble_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::BUBBLE_OUTGOING)
        .stroke(Stroke::new(1.0, colors::BUBBLE_BORDER))
        .corner_radius(CornerRadius {
            nw: 12,
            ne: 12,
            sw: 12,
            se: 4, // Tail side
        })
        .inner_margin(egui::Margin::symmetric(12, 8))
}

/// Create a frame style for incoming message bubbles
pub fn incoming_bubble_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::BUBBLE_INCOMING)
        .stroke(Stroke::new(1.0, colors::BUBBLE_BORDER))
        .corner_radius(CornerRadius {
            nw: 12,
            ne: 12,
            sw: 4, // Tail side
            se: 12,
        })
        .inner_margin(egui::Margin::symmetric(12, 8))
}

/// Create a frame for contact list items
pub fn contact_item_frame(is_selected: bool, is_hovered: bool) -> egui::Frame {
    let bg_color = if is_selected {
        colors::CHAT_LIST_HOVER
    } else if is_hovered {
        Color32::from_rgba_unmultiplied(
            colors::CHAT_LIST_HOVER.r(),
            colors::CHAT_LIST_HOVER.g(),
            colors::CHAT_LIST_HOVER.b(),
            128,
        )
    } else {
        colors::CHAT_LIST_BG
    };

    egui::Frame::new()
        .fill(bg_color)
        .inner_margin(egui::Margin::symmetric(12, 10))
}

/// Create a frame for modal dialogs
pub fn modal_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(colors::MAIN_CHAT_BG)
        .stroke(Stroke::new(2.0, colors::BUBBLE_BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::same(20))
        .shadow(egui::epaint::Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(60),
        })
}

/// Style a primary button
#[allow(dead_code)]
pub fn style_primary_button(_ui: &mut egui::Ui) -> egui::Button<'static> {
    egui::Button::new("")
        .fill(colors::BUTTON_PRIMARY)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(6))
}

/// Get the text color for dark backgrounds
#[allow(dead_code)]
pub fn text_on_dark() -> Color32 {
    colors::TEXT_LIGHT
}

/// Get the text color for light backgrounds
#[allow(dead_code)]
pub fn text_on_light() -> Color32 {
    colors::TEXT_DARK
}

