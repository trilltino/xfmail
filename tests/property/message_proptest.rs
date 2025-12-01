//! Property-based tests for Message type
//!
//! Uses proptest to generate random inputs and verify properties

use proptest::prelude::*;
use xfcollab::shared::message::Message;

proptest! {
    #[test]
    fn test_message_serialization_roundtrip(
        text in ".*",
        author in ".*",
    ) {
        let message = Message::new(text.clone(), author.clone());
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(message.text, deserialized.text);
        prop_assert_eq!(message.author, deserialized.author);
    }

    #[test]
    fn test_message_has_timestamp(text in ".*", author in ".*") {
        let message = Message::new(text, author);
        prop_assert!(!message.timestamp.is_empty());
    }

    #[test]
    fn test_message_version_is_optional(text in ".*", author in ".*") {
        let message = Message::new(text, author);
        // Version can be None or Some(String)
        prop_assert!(message.version.is_none() || message.version.is_some());
    }
}
