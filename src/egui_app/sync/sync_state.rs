//! # Sync State Management
//!
//! Comprehensive synchronization state tracking and management.
//!
//! ## Features
//!
//! - **State Tracking**: Current sync status and progress
//! - **Metrics Collection**: Performance and error metrics
//! - **Status Updates**: Real-time sync state changes
//! - **Error Handling**: Sync error tracking and reporting

#[derive(Debug, Clone)]
pub struct SyncState {
    pub is_syncing: bool,
    pub last_sync: Option<String>,
    pub progress: f32,
    pub pending_operations: usize,
    pub failed_operations: usize,
    pub network_status: crate::egui_app::sync::network_monitor::NetworkStatus,
    pub errors: Vec<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            is_syncing: false,
            last_sync: None,
            progress: 0.0,
            pending_operations: 0,
            failed_operations: 0,
            network_status: crate::egui_app::sync::network_monitor::NetworkStatus::Offline,
            errors: Vec::new(),
        }
    }
}