//! Real-time broadcast integration tests

#[cfg(feature = "ssr")]
mod tests {
    use tokio::time::{timeout, Duration};
    use xfcollab::backend::realtime::broadcast_event;
    use xfcollab::shared::event::{EventType, RealtimeEvent};
    use xfcollab::shared::message::Message;

    #[tokio::test]
    async fn test_broadcast_message_event() {
        let message = Message::new("Test message".to_string(), "test_user".to_string());
        let event = RealtimeEvent::new_message_event(&message);

        // Test that event can be created and serialized
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealtimeEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_type, EventType::Message);
    }
}
