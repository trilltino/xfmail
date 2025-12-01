/**
 * Real-time Event Broadcasting
 * 
 * This module provides utilities for broadcasting real-time events to all
 * subscribers. It includes the broadcast type definition and the broadcast
 * helper function.
 * 
 * # Broadcasting
 * 
 * Events are broadcast using `tokio::sync::broadcast`, which provides
 * a multi-producer, multi-consumer channel. All subscribers receive
 * a copy of each event.
 * 
 * # Event Types
 * 
 * The broadcast system supports multiple event types:
 * - Messages
 * - Notifications
 * - Status updates
 * - Typing indicators
 * - Custom events
 */

use crate::shared::RealtimeEvent;
use tokio::sync::broadcast;

/// Real-time update event broadcast
/// 
/// This type represents a broadcast channel for real-time events.
/// It can be cloned and shared across multiple handlers to allow
/// broadcasting events from anywhere in the application.
/// 
/// # Usage
/// 
/// ```rust
/// use braid_site::backend::realtime::RealtimeEventBroadcast;
/// 
/// let (tx, _) = broadcast::channel::<RealtimeEvent>(1000);
/// let broadcast: RealtimeEventBroadcast = tx;
/// ```
pub type RealtimeEventBroadcast = broadcast::Sender<RealtimeEvent>;

/// Broadcast a real-time event to all subscribers
/// 
/// This is a helper function that can be called from anywhere in the application
/// to broadcast real-time events to all connected clients.
/// 
/// # Arguments
/// 
/// * `broadcast_tx` - The broadcast sender
/// * `event` - The event to broadcast
/// 
/// # Returns
/// 
/// Number of active subscribers that received the event (0 if no subscribers)
/// 
/// # Example
/// 
/// ```rust
/// use braid_site::shared::RealtimeEvent;
/// use braid_site::backend::realtime::broadcast::broadcast_event;
/// 
/// let event = RealtimeEvent::notification(
///     "New Update".to_string(),
///     "Something happened!".to_string()
/// );
/// 
/// let subscriber_count = broadcast_event(&broadcast_tx, event).await;
/// println!("Event broadcast to {} subscribers", subscriber_count);
/// ```
#[cfg(feature = "ssr")]
pub async fn broadcast_event(
    broadcast_tx: &RealtimeEventBroadcast,
    event: RealtimeEvent,
) -> usize {
    match broadcast_tx.send(event) {
        Ok(subscriber_count) => {
            tracing::info!("[Realtime] Event broadcast to {} subscribers", subscriber_count);
            subscriber_count
        }
        Err(e) => {
            // No subscribers, that's okay
            tracing::debug!("[Realtime] No subscribers to receive event: {:?}", e);
            0
        }
    }
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use super::*;
    use crate::shared::event::EventType;

    #[tokio::test]
    async fn test_broadcast_event_with_subscribers() {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<RealtimeEvent>(100);
        
        // Spawn a subscriber
        let mut rx_clone = tx.subscribe();
        tokio::spawn(async move {
            let _ = rx_clone.recv().await;
        });
        
        // Give subscriber time to register
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let event = RealtimeEvent::new(EventType::Message, serde_json::json!({"text": "Hello"}));
        let count = broadcast_event(&tx, event).await;
        
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_broadcast_event_no_subscribers() {
        let (tx, _) = tokio::sync::broadcast::channel::<RealtimeEvent>(100);
        
        let event = RealtimeEvent::new(EventType::Message, serde_json::json!({"text": "Hello"}));
        let count = broadcast_event(&tx, event).await;
        
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_multiple_subscribers() {
        let (tx, _) = tokio::sync::broadcast::channel::<RealtimeEvent>(100);
        
        // Create multiple subscribers
        let mut sub1 = tx.subscribe();
        let mut sub2 = tx.subscribe();
        let mut sub3 = tx.subscribe();
        
        tokio::spawn(async move { let _ = sub1.recv().await; });
        tokio::spawn(async move { let _ = sub2.recv().await; });
        tokio::spawn(async move { let _ = sub3.recv().await; });
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let event = RealtimeEvent::new(EventType::Message, serde_json::json!({"text": "Hello"}));
        let count = broadcast_event(&tx, event).await;
        
        assert!(count >= 3);
    }
}

