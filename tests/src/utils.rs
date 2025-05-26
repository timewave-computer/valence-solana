// Common utilities for Valence Protocol tests
use anchor_lang::prelude::*;

/// Common test setup for all programs
pub struct TestContext {
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub user: Pubkey,
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
    use anchor_lang::solana_program::hash::{hash, Hasher};
    let mut hasher = Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}

/// Generate a test public key
pub fn generate_test_pubkey(seed: &str) -> Pubkey {
    Pubkey::new_from_array(generate_test_hash(seed.as_bytes()))
} 