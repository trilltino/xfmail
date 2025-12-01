/**
 * Message Data Structure
 * 
 * This module defines the Message struct used for chat messages
 * and their serialization/deserialization for Braid protocol communication.
 * 
 * The Message struct is shared between frontend and backend, allowing
 * seamless serialization over HTTP and deserialization in both contexts.
 */
use serde::{Deserialize, Serialize};

/// Represents a single chat message
///
/// This structure is used both on the server (for storage and Braid responses)
/// and on the client (for display in the UI). It's serialized to/from JSON
/// for communication over the Braid HTTP protocol.
///
/// # Fields
/// * `text` - The message content
/// * `author` - The author's name or identifier
/// * `timestamp` - ISO 8601 formatted timestamp (RFC3339)
/// * `version` - Optional Braid version ID (assigned by server)
///
/// # Example
/// ```rust
/// use braid_site::shared::Message;
///
/// let message = Message::new(
///     "Hello, world!".to_string(),
///     "Alice".to_string()
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// The message text content
    pub text: String,
    /// The author's name
    pub author: String,
    /// ISO 8601 timestamp (RFC3339 format)
    pub timestamp: String,
    /// Optional Braid version ID
    /// 
    /// This is assigned by the server when the message is created.
    /// It's used for version tracking in the Braid DAG.
    /// 
    /// When sending a message from the client, this should be `None`.
    /// The server will assign a version ID and return it in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Message {
    /// Create a new message with the current timestamp
    /// 
    /// This is the primary constructor for creating new messages.
    /// The timestamp is automatically set to the current UTC time.
    /// 
    /// # Arguments
    /// * `text` - The message text
    /// * `author` - The author's name
    /// 
    /// # Returns
    /// A new Message instance with the current UTC timestamp
    /// 
    /// # Example
    /// ```rust
    /// use braid_site::shared::Message;
    /// 
    /// let message = Message::new(
    ///     "Hello, world!".to_string(),
    ///     "Alice".to_string()
    /// );
    /// 
    /// assert_eq!(message.text, "Hello, world!");
    /// assert_eq!(message.author, "Alice");
    /// assert!(message.version.is_none());
    /// ```
    pub fn new(text: String, author: String) -> Self {
        Self {
            text,
            author,
            timestamp: get_timestamp(),
            version: None,
        }
    }
    
    /// Create a new message with a specific version
    /// 
    /// This constructor is typically used by the server when creating
    /// messages from stored data that already has a version ID.
    /// 
    /// # Arguments
    /// * `text` - The message text
    /// * `author` - The author's name
    /// * `version` - The Braid version ID
    /// 
    /// # Returns
    /// A new Message instance with the specified version
    /// 
    /// # Example
    /// ```rust
    /// use braid_site::shared::Message;
    /// 
    /// let message = Message::with_version(
    ///     "Hello, world!".to_string(),
    ///     "Alice".to_string(),
    ///     "abc123".to_string()
    /// );
    /// 
    /// assert_eq!(message.version, Some("abc123".to_string()));
    /// ```
    pub fn with_version(text: String, author: String, version: String) -> Self {
        Self {
            text,
            author,
            timestamp: get_timestamp(),
            version: Some(version),
        }
    }
}

/// Get the current timestamp as an RFC3339 string
/// 
/// This function uses chrono when available (with ssr or hydrate features),
/// or returns a placeholder string when chrono is not available.
fn get_timestamp() -> String {
    // Use chrono on server (ssr) and in wasm targets (client/web)
    #[cfg(any(feature = "ssr", target_arch = "wasm32"))]
    {
        chrono::Utc::now().to_rfc3339()
    }

    // Fallback when chrono isn't enabled and not targeting wasm
    #[cfg(not(any(feature = "ssr", target_arch = "wasm32")))]
    {
        String::from("1970-01-01T00:00:00Z")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let message = Message::new("Hello".to_string(), "Alice".to_string());
        assert_eq!(message.text, "Hello");
        assert_eq!(message.author, "Alice");
        assert!(message.version.is_none());
        assert!(!message.timestamp.is_empty());
    }

    #[test]
    fn test_message_with_version() {
        let message = Message::with_version(
            "Hello".to_string(),
            "Alice".to_string(),
            "v1".to_string(),
        );
        assert_eq!(message.text, "Hello");
        assert_eq!(message.author, "Alice");
        assert_eq!(message.version, Some("v1".to_string()));
        assert!(!message.timestamp.is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let message = Message::new("Hello".to_string(), "Alice".to_string());
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(message.text, deserialized.text);
        assert_eq!(message.author, deserialized.author);
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"text":"Hello","author":"Alice","timestamp":"2023-01-01T00:00:00Z"}"#;
        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.text, "Hello");
        assert_eq!(message.author, "Alice");
    }

    #[test]
    fn test_message_equality() {
        let msg1 = Message::new("Hello".to_string(), "Alice".to_string());
        let msg2 = Message::new("Hello".to_string(), "Alice".to_string());
        // Messages with same content should be equal (timestamps are the same in test environment)
        assert_eq!(msg1, msg2);

        // But messages with different content should not be equal
        let msg3 = Message::new("Different".to_string(), "Alice".to_string());
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_message_with_version_serialization() {
        let message = Message::with_version(
            "Hello".to_string(),
            "Alice".to_string(),
            "v1".to_string(),
        );
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("v1"));
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, Some("v1".to_string()));
    }
}

