//! raibid-agent
//!
//! CI agent runner that polls the job queue and executes builds.
//! This crate will handle:
//! - Job polling from Redis Streams
//! - Build execution in isolated environments
//! - Cache management for dependencies
//! - Result reporting back to the server
//!
//! ## Status: Placeholder
//! This crate is a placeholder for future implementation.

#![allow(dead_code)]

use anyhow::Result;

/// Agent configuration
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_type: AgentType,
}

/// Type of CI agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Rust,
    // Future: Go, Node, Python, etc.
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: uuid::Uuid::new_v4().to_string(),
            agent_type: AgentType::Rust,
        }
    }
}

/// Start the CI agent
pub async fn start_agent(_config: AgentConfig) -> Result<()> {
    // Placeholder implementation
    // Will be implemented in a future issue
    anyhow::bail!("Agent implementation pending")
}

// Temporary implementation to avoid compilation errors
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }
    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "00000000-0000-0000-0000-000000000000")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.agent_type, AgentType::Rust);
    }
}
