//! Theme Module
//!
//! This module provides the color scheme and styling for the Telegram-style
//! messaging application. It includes:
//!
//! - Color constants for the brown/tan theme
//! - Styling helper functions for consistent UI appearance
//! - Frame builders for various UI components
//!
//! # Usage
//!
//! ```rust
//! use crate::egui_app::theme::{colors, styles};
//!
//! // Apply global theme
//! styles::apply_global_theme(ctx);
//!
//! // Use color constants
//! ui.painter().rect_filled(rect, 0.0, colors::SIDEBAR_BG);
//!
//! // Use frame builders
//! styles::sidebar_frame().show(ui, |ui| {
//!     // Sidebar content
//! });
//! ```

pub mod colors;
pub mod styles;

pub use colors::*;
pub use styles::*;

