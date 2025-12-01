//! Color Constants for Telegram-Style Messaging Theme
//!
//! This module defines all the color constants used throughout the messaging UI.
//! Colors are based on a warm brown/tan color scheme similar to classic Telegram themes.

use eframe::egui::Color32;

/// Main sidebar background - Deep brown
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(0x2F, 0x1E, 0x1A);

/// Active chat highlight strip - Medium brown
pub const ACTIVE_CHAT_STRIP: Color32 = Color32::from_rgb(0x4A, 0x2E, 0x22);

/// Chat list background - Dark brown
pub const CHAT_LIST_BG: Color32 = Color32::from_rgb(0x3A, 0x27, 0x21);

/// Chat list hovered or selected item - Lighter brown
pub const CHAT_LIST_HOVER: Color32 = Color32::from_rgb(0x5C, 0x3A, 0x2C);

/// Chat item text - Cream
pub const CHAT_ITEM_TEXT: Color32 = Color32::from_rgb(0xF0, 0xE0, 0xD6);

/// Main chat background - Off-white
pub const MAIN_CHAT_BG: Color32 = Color32::from_rgb(0xF7, 0xF2, 0xEC);

/// Message bubble outgoing - Tan
pub const BUBBLE_OUTGOING: Color32 = Color32::from_rgb(0xD8, 0xC0, 0xA8);

/// Message bubble incoming - Light tan
pub const BUBBLE_INCOMING: Color32 = Color32::from_rgb(0xEA, 0xDB, 0xC8);

/// Message bubble border - Muted brown
pub const BUBBLE_BORDER: Color32 = Color32::from_rgb(0xC7, 0xB2, 0x9A);

/// Top bar background - Dark brown
pub const TOP_BAR_BG: Color32 = Color32::from_rgb(0x3E, 0x2A, 0x24);

/// Input bar background - Light tan
pub const INPUT_BAR_BG: Color32 = Color32::from_rgb(0xE6, 0xD7, 0xC7);

/// Input bar border - Muted tan
pub const INPUT_BAR_BORDER: Color32 = Color32::from_rgb(0xC3, 0xA9, 0x90);

/// Icons - Light brown
pub const ICONS: Color32 = Color32::from_rgb(0xC6, 0xB2, 0x9E);

/// Text on dark backgrounds
pub const TEXT_LIGHT: Color32 = Color32::from_rgb(0xF0, 0xE0, 0xD6);

/// Text on light backgrounds
pub const TEXT_DARK: Color32 = Color32::from_rgb(0x2F, 0x1E, 0x1A);

/// Online status indicator - Green
pub const STATUS_ONLINE: Color32 = Color32::from_rgb(0x4C, 0xAF, 0x50);

/// Offline status indicator - Gray
pub const STATUS_OFFLINE: Color32 = Color32::from_rgb(0x9E, 0x9E, 0x9E);

/// Success color - Green
pub const SUCCESS: Color32 = Color32::from_rgb(0x4C, 0xAF, 0x50);

/// Error color - Red
pub const ERROR: Color32 = Color32::from_rgb(0xE5, 0x73, 0x73);

/// Warning color - Orange
pub const WARNING: Color32 = Color32::from_rgb(0xFF, 0xA7, 0x26);

/// Button primary background
pub const BUTTON_PRIMARY: Color32 = Color32::from_rgb(0x5C, 0x3A, 0x2C);

/// Button primary hover
pub const BUTTON_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x6D, 0x4B, 0x3D);

/// Button secondary background
pub const BUTTON_SECONDARY: Color32 = Color32::from_rgb(0xC7, 0xB2, 0x9A);

/// Unread badge background
pub const UNREAD_BADGE: Color32 = Color32::from_rgb(0x5C, 0x3A, 0x2C);

/// Timestamp text color
pub const TIMESTAMP: Color32 = Color32::from_rgb(0x8B, 0x7B, 0x6B);

/// Separator/divider color
pub const SEPARATOR: Color32 = Color32::from_rgb(0xD0, 0xC0, 0xB0);

/// Primary text color
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x2F, 0x1E, 0x1A);

/// Secondary text color (muted)
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x8B, 0x7B, 0x6B);

/// Accent color for highlights
pub const ACCENT: Color32 = Color32::from_rgb(0x5C, 0x3A, 0x2C);

/// Selected item background
pub const SELECTED_ITEM: Color32 = Color32::from_rgb(0x5C, 0x3A, 0x2C);

/// Hover item background
pub const HOVER_ITEM: Color32 = Color32::from_rgb(0x4A, 0x2E, 0x22);

/// Chat header background
pub const CHAT_HEADER_BG: Color32 = Color32::from_rgb(0x3E, 0x2A, 0x24);

/// Input background
pub const INPUT_BG: Color32 = Color32::from_rgb(0xE6, 0xD7, 0xC7);

/// Dark background for main areas
pub const BG_DARK: Color32 = Color32::from_rgb(0x2F, 0x1E, 0x1A);
