// Common utilities for Valence Protocol tests
use anchor_lang::prelude::*;

/// Common test setup for all programs
pub struct TestContext {
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub user: Pubkey,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        
        Self {
            payer,
            authority,
            user,
        }
    }
}

/// Assert that two values are equal with a helpful error message
#[macro_export]
macro_rules! assert_eq_with_msg {
    ($left:expr, $right:expr, $msg:expr) => {
        assert_eq!($left, $right, "{}: expected {}, got {}", $msg, $right, $left);
    };
}

/// Assert that a condition is true with a helpful error message
#[macro_export]
macro_rules! assert_with_msg {
    ($condition:expr, $msg:expr) => {
        assert!($condition, "{}", $msg);
    };
}

/// Generate a test hash for testing purposes
pub fn generate_test_hash(data: &[u8]) -> [u8; 32] {
    use anchor_lang::solana_program::hash::Hasher;
    let mut hasher = Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}

/// Generate a test public key
pub fn generate_test_pubkey(seed: &str) -> Pubkey {
    Pubkey::new_from_array(generate_test_hash(seed.as_bytes()))
}

/// Create a test authorization with default values
pub fn create_test_authorization(label: &str, user: Pubkey) -> authorization::state::Authorization {
    authorization::state::Authorization {
        label: label.to_string(),
        owner: user,
        is_active: true,
        permission_type: authorization::state::PermissionType::Allowlist,
        allowed_users: vec![user],
        not_before: 0,
        expiration: Some(2000000000),
        max_concurrent_executions: 5,
        priority: authorization::state::Priority::Medium,
        subroutine_type: authorization::state::SubroutineType::Atomic,
        current_executions: 0,
        bump: 254,
    }
}

/// Create a test library info with default values
pub fn create_test_library_info(program_id: Pubkey, library_type: &str) -> registry::state::LibraryInfo {
    registry::state::LibraryInfo {
        program_id,
        library_type: library_type.to_string(),
        description: format!("Test library for {}", library_type),
        is_approved: true,
        version: "1.0.0".to_string(),
        last_updated: 1000000000,
        dependencies: vec![],
        bump: 254,
    }
}

/// Create a test message batch with default values
pub fn create_test_message_batch(execution_id: u64, caller: Pubkey) -> processor::state::MessageBatch {
    processor::state::MessageBatch {
        execution_id,
        messages: vec![
            processor::state::ProcessorMessage {
                program_id: generate_test_pubkey("target_program"),
                data: vec![1, 2, 3, 4],
                accounts: vec![
                    processor::state::AccountMetaData {
                        pubkey: caller,
                        is_signer: true,
                        is_writable: false,
                    },
                ],
            },
        ],
        subroutine_type: processor::state::SubroutineType::Atomic,
        expiration_time: Some(2000000000),
        priority: processor::state::Priority::Medium,
        caller,
        callback_address: generate_test_pubkey("callback"),
        created_at: 1000000000,
        bump: 254,
    }
}

/// Create a test verification key with default values
pub fn create_test_verification_key(program_id: Pubkey, registry_id: u64) -> zk_verifier::state::VerificationKey {
    zk_verifier::state::VerificationKey {
        program_id,
        registry_id,
        vk_hash: [1u8; 32], // Simple test hash
        key_type: zk_verifier::state::VerificationKeyType::Groth16,
        is_active: true,
        bump: 255,
    }
}

/// Validate that a timestamp is within a reasonable range
pub fn is_valid_timestamp(timestamp: i64) -> bool {
    // Timestamps should be between 2020 and 2050 (in seconds)
    (1577836800..=2524608000).contains(&timestamp)
}

/// Calculate the success rate as a percentage
pub fn calculate_success_rate(successful: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (successful as f64 / total as f64) * 100.0
    }
}

/// Validate that a version string follows semantic versioning
pub fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Generate test data of a specific size
pub fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Create a test queue state with default values
pub fn create_test_queue_state(capacity: u64) -> processor::state::QueueState {
    processor::state::QueueState::new(capacity)
} 