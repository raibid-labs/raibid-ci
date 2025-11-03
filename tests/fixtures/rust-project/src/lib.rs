//! Test fixture for E2E CI pipeline testing
//!
//! This is a minimal Rust project used to test the complete CI flow.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub timestamp: u64,
}

impl Message {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
        assert_eq!(greet("Rust"), "Hello, Rust!");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new("Test message");
        assert_eq!(msg.text, "Test message");
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new("Test");
        let json = msg.to_json();
        assert!(json.contains("Test"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"text":"Hello","timestamp":1234567890}"#;
        let msg = Message::from_json(json).unwrap();
        assert_eq!(msg.text, "Hello");
        assert_eq!(msg.timestamp, 1234567890);
    }

    #[test]
    fn test_message_roundtrip() {
        let original = Message {
            text: "Roundtrip test".to_string(),
            timestamp: 9999999999,
        };
        let json = original.to_json();
        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}
