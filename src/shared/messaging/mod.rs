//! Messaging Module
//!
//! This module contains all the data structures for the messaging system:
//!
//! - `Contact` - A user's contact/friend
//! - `ChatMessage` - A message in a conversation
//! - `Conversation` - A conversation between users
//! - `FriendRequest` - A friend request between users
//!
//! # Usage
//!
//! ```rust
//! use xfmail::shared::messaging::{Contact, ChatMessage, Conversation, FriendRequest};
//! ```

pub mod contact;
pub mod message;
pub mod conversation;
pub mod friend_request;
pub mod message_crdt;

// Re-export all types
pub use contact::{Contact, ListContactsResponse, GetContactResponse};
pub use message::{
    ChatMessage, MessageType, SendMessageRequest, SendMessageResponse,
    ListMessagesRequest, ListMessagesResponse,
};
pub use conversation::{
    Conversation, ListConversationsResponse, CreateConversationRequest,
    CreateConversationResponse,
};
pub use friend_request::{
    FriendRequest, FriendRequestStatus, SendFriendRequestRequest,
    SendFriendRequestResponse, RespondFriendRequestRequest,
    RespondFriendRequestResponse, ListFriendRequestsResponse,
};
pub use message_crdt::{
    LamportTimestamp, MessageOperation, MessageOpType, MessageState, MessageEntry,
};

