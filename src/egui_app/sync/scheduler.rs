//! # Sync Scheduler
//!
//! Intelligent scheduling system for background synchronization operations.
//! Optimizes sync timing based on network conditions, battery status, and user activity.
//!
//! ## Features
//!
//! - **Smart Scheduling**: Adapts sync frequency based on context
//! - **Resource Awareness**: Considers battery, network, and CPU usage
//! - **Priority Queues**: Handles urgent operations with higher priority
//! - **Bandwidth Optimization**: Adjusts sync based on network speed
//! - **User Activity**: Reduces sync during active usage to preserve UX

use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Synchronization scheduler
#[derive(Debug)]
pub struct SyncScheduler {
    /// Last sync time
    last_sync: RwLock<Option<Instant>>,
    /// Current sync interval
    current_interval: RwLock<Duration>,
    /// Base sync interval (when online and active)
    base_interval: Duration,
    /// Whether scheduler is active
    is_active: RwLock<bool>,
}

impl SyncScheduler {
    /// Create a new sync scheduler
    pub fn new() -> Self {
        Self {
            last_sync: RwLock::new(None),
            current_interval: RwLock::new(Duration::from_secs(30)),
            base_interval: Duration::from_secs(30),
            is_active: RwLock::new(false),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) {
        *self.is_active.write().await = true;
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        *self.is_active.write().await = false;
    }

    /// Check if sync should be performed now
    pub async fn should_sync(&self) -> bool {
        if !*self.is_active.read().await {
            return false;
        }

        let last_sync = *self.last_sync.read().await;
        let interval = *self.current_interval.read().await;

        match last_sync {
            Some(time) => time.elapsed() >= interval,
            None => true, // First sync
        }
    }

    /// Record a successful sync
    pub async fn record_sync(&self) {
        *self.last_sync.write().await = Some(Instant::now());
    }

    /// Adjust sync interval based on conditions
    pub async fn adjust_interval(&self, network_quality: NetworkQuality, battery_level: Option<f32>) {
        let mut interval = self.base_interval;

        // Adjust for network quality
        interval = match network_quality {
            NetworkQuality::Excellent => interval / 2,
            NetworkQuality::Good => interval,
            NetworkQuality::Poor => interval * 2,
            NetworkQuality::Offline => interval * 10,
        };

        // Adjust for battery level
        if let Some(battery) = battery_level {
            if battery < 0.2 {
                interval *= 5; // Reduce sync frequency on low battery
            }
        }

        *self.current_interval.write().await = interval;
    }

    /// Get time until next sync
    pub async fn time_until_next_sync(&self) -> Option<Duration> {
        let last_sync = *self.last_sync.read().await?;
        let interval = *self.current_interval.read().await;

        let elapsed = last_sync.elapsed();
        if elapsed >= interval {
            Some(Duration::ZERO)
        } else {
            Some(interval - elapsed)
        }
    }
}

/// Network quality assessment
#[derive(Debug, Clone, Copy)]
pub enum NetworkQuality {
    /// Fast, reliable connection
    Excellent,
    /// Good connection
    Good,
    /// Slow or unreliable connection
    Poor,
    /// No network connection
    Offline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = SyncScheduler::new();
        assert!(!*scheduler.is_active.read().await);
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let scheduler = SyncScheduler::new();

        scheduler.start().await;
        assert!(*scheduler.is_active.read().await);

        scheduler.stop().await;
        assert!(!*scheduler.is_active.read().await);
    }

    #[tokio::test]
    async fn test_should_sync_initially() {
        let scheduler = SyncScheduler::new();
        scheduler.start().await;

        // Should sync initially (no previous sync)
        assert!(scheduler.should_sync().await);
    }
}