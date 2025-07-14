//! Test helpers and utilities for shard-related integration tests

pub use ::shard::*;

/// Test-specific shard configuration
pub struct TestShardConfig {
    /// Default capabilities for test sessions
    pub default_capabilities: Capabilities,
    /// Test session metadata
    pub test_metadata: Vec<u8>,
}

impl Default for TestShardConfig {
    fn default() -> Self {
        Self {
            default_capabilities: Capabilities(
                Capabilities::READ | Capabilities::WRITE | Capabilities::EXECUTE
            ),
            test_metadata: b"test_session".to_vec(),
        }
    }
}

/// Helper to create test sessions with common configurations
pub fn create_test_session_params() -> (Capabilities, Vec<u8>) {
    let config = TestShardConfig::default();
    (config.default_capabilities, config.test_metadata)
}

/// Helper to verify session state for tests
pub fn verify_session_state(session: &Session, expected_capabilities: Capabilities) -> bool {
    session.capabilities == expected_capabilities && !session.consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestShardConfig::default();
        assert_eq!(
            config.default_capabilities,
            Capabilities(Capabilities::READ | Capabilities::WRITE | Capabilities::EXECUTE)
        );
        assert_eq!(config.test_metadata, b"test_session");
    }

    #[test]
    fn test_session_params() {
        let (caps, metadata) = create_test_session_params();
        assert_eq!(
            caps,
            Capabilities(Capabilities::READ | Capabilities::WRITE | Capabilities::EXECUTE)
        );
        assert_eq!(metadata, b"test_session");
    }
}