//! # Sync Metrics and Analytics
//!
//! Performance monitoring and analytics for synchronization operations.
//!
//! ## Features
//!
//! - **Performance Metrics**: Sync speed, latency, and throughput
//! - **Error Tracking**: Failure rates and error patterns
//! - **Bandwidth Usage**: Network usage monitoring
//! - **User Experience**: Sync impact on application responsiveness

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct SyncMetrics {
    pub total_syncs: u64,
    pub successful_syncs: u64,
    pub failed_syncs: u64,
    pub average_sync_duration: Duration,
    pub total_bytes_synced: u64,
    pub last_sync_duration: Option<Duration>,
    pub last_sync_start: Option<Instant>,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            total_syncs: 0,
            successful_syncs: 0,
            failed_syncs: 0,
            average_sync_duration: Duration::from_secs(0),
            total_bytes_synced: 0,
            last_sync_duration: None,
            last_sync_start: None,
        }
    }

    pub fn record_sync_start(&mut self) {
        self.last_sync_start = Some(Instant::now());
        self.total_syncs += 1;
    }

    pub fn record_sync_success(&mut self, bytes_synced: u64) {
        if let Some(start) = self.last_sync_start.take() {
            let duration = start.elapsed();
            self.last_sync_duration = Some(duration);
            self.successful_syncs += 1;
            self.total_bytes_synced += bytes_synced;

            // Update rolling average
            let total_duration = self.average_sync_duration * (self.successful_syncs - 1) + duration;
            self.average_sync_duration = total_duration / self.successful_syncs;
        }
    }

    pub fn record_sync_failure(&mut self) {
        self.last_sync_start = None;
        self.failed_syncs += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_syncs == 0 {
            0.0
        } else {
            self.successful_syncs as f64 / self.total_syncs as f64
        }
    }
}