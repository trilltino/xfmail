/**
 * Real-time Event System
 * 
 * This module defines event types for the real-time notification system.
 * Events can represent different types of updates: messages, notifications,
 * status changes, etc.
 */
use serde::{Deserialize, Serialize};

/// Type of real-time event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Chat message event
    Message,
    /// User notification event
    Notification,
    /// Status update event
    Status,
    /// Typing indicator event
    Typing,
    /// Custom event type
    Custom(String),
}

/// Real-time event that can be broadcast to all subscribers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealtimeEvent {
    /// Type of event
    pub event_type: EventType,
    /// Event payload (JSON-serializable data)
    pub payload: serde_json::Value,
    /// Timestamp when event occurred
    pub timestamp: String,
    /// Optional version ID for Braid protocol
    pub version: Option<String>,
}

impl RealtimeEvent {
    /// Create a new real-time event
    pub fn new(event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            event_type,
            payload,
            timestamp: get_timestamp(),
            version: None,
        }
    }
    
    /// Create a message event
    pub fn message(payload: serde_json::Value) -> Self {
        Self::new(EventType::Message, payload)
    }
    
    /// Create a notification event
    pub fn notification(title: String, message: String) -> Self {
        Self::new(
            EventType::Notification,
            serde_json::json!({
                "title": title,
                "message": message,
            }),
        )
    }
    
    /// Create a status event
    pub fn status(status: String, details: Option<serde_json::Value>) -> Self {
        Self::new(
            EventType::Status,
            serde_json::json!({
                "status": status,
                "details": details,
            }),
        )
    }
    
    /// Create a typing event
    pub fn typing(user: String, is_typing: bool) -> Self {
        Self::new(
            EventType::Typing,
            serde_json::json!({
                "user": user,
                "is_typing": is_typing,
            }),
        )
    }
    
    /// Create a message event from a Message struct
    pub fn new_message_event(message: &crate::shared::message::Message) -> Self {
        let payload = serde_json::to_value(message).unwrap();
        Self::message(payload)
    }
    
    /// Set the version ID
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

/// Get the current timestamp as an RFC3339 string
fn get_timestamp() -> String {
    #[cfg(feature = "ssr")]
    {
        chrono::Utc::now().to_rfc3339()
    }

    #[cfg(not(feature = "ssr"))]
    {
        String::from("1970-01-01T00:00:00Z")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let event = RealtimeEvent::new(
            EventType::Message,
            serde_json::json!({"text": "Hello"}),
        );
        assert_eq!(event.event_type, EventType::Message);
        assert!(!event.timestamp.is_empty());
        assert!(event.version.is_none());
    }

    #[test]
    fn test_event_message() {
        let payload = serde_json::json!({"text": "Hello"});
        let event = RealtimeEvent::message(payload.clone());
        assert_eq!(event.event_type, EventType::Message);
        assert_eq!(event.payload, payload);
    }

    #[test]
    fn test_event_notification() {
        let event = RealtimeEvent::notification("Title".to_string(), "Message".to_string());
        assert_eq!(event.event_type, EventType::Notification);
        assert_eq!(event.payload["title"], "Title");
        assert_eq!(event.payload["message"], "Message");
    }

    #[test]
    fn test_event_status() {
        let event = RealtimeEvent::status("online".to_string(), None);
        assert_eq!(event.event_type, EventType::Status);
        assert_eq!(event.payload["status"], "online");
    }

    #[test]
    fn test_event_typing() {
        let event = RealtimeEvent::typing("user1".to_string(), true);
        assert_eq!(event.event_type, EventType::Typing);
        assert_eq!(event.payload["user"], "user1");
        assert_eq!(event.payload["is_typing"], true);
    }

    #[test]
    fn test_event_with_version() {
        let event = RealtimeEvent::new(EventType::Message, serde_json::json!({}))
            .with_version("v1".to_string());
        assert_eq!(event.version, Some("v1".to_string()));
    }

    #[test]
    fn test_event_serialization() {
        let event = RealtimeEvent::message(serde_json::json!({"text": "Hello"}));
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealtimeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type, deserialized.event_type);
    }

    #[test]
    fn test_event_type_custom() {
        let event_type = EventType::Custom("custom_type".to_string());
        let json = serde_json::to_string(&event_type).unwrap();
        assert!(json.contains("custom_type"));
    }

    #[test]
    fn test_new_message_event() {
        use crate::shared::message::Message;
        let message = Message::new("Hello".to_string(), "Alice".to_string());
        let event = RealtimeEvent::new_message_event(&message);
        assert_eq!(event.event_type, EventType::Message);
        assert_eq!(event.payload["text"], "Hello");
        assert_eq!(event.payload["author"], "Alice");
    }
}

