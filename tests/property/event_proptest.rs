//! Property-based tests for Event type

use proptest::prelude::*;
use xfcollab::shared::event::{EventType, RealtimeEvent};
use xfcollab::shared::message::Message;

proptest! {
    #[test]
    fn test_event_serialization_roundtrip(
        text in ".*",
        author in ".*",
    ) {
        let message = Message::new(text, author);
        let event = RealtimeEvent::new_message_event(&message);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealtimeEvent = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(event.event_type, deserialized.event_type);
    }

    #[test]
    fn test_event_has_timestamp(text in ".*", author in ".*") {
        let message = Message::new(text, author);
        let event = RealtimeEvent::new_message_event(&message);
        prop_assert!(!event.timestamp.is_empty());
    }
}
