//! Message types for LLM communication
//!
//! This module provides the core message types used throughout the conductor system.
//! Messages are immutable and serializable, following OpenAI's chat completion format.

use serde::{Deserialize, Serialize};

/// Represents a single message in a conversation
///
/// Compatible with OpenAI's message format:
/// ```json
/// {
///   "role": "user",
///   "content": "Hello, world!"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// Role of the message sender (system, user, assistant, tool)
    pub role: String,

    /// Content of the message
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("user", "Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_helpers() {
        assert_eq!(Message::system("sys").role, "system");
        assert_eq!(Message::user("usr").role, "user");
        assert_eq!(Message::assistant("asst").role, "assistant");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Test");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}
