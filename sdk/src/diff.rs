//! Diff construction utilities

use anchor_lang::prelude::*;

/// Calculate diff hash for state transition
pub fn calculate_diff_hash(prev_hash: [u8; 32], operation_result: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(operation_result);
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Verify diff matches expected value
pub fn verify_diff(expected: [u8; 32], actual: [u8; 32]) -> Result<()> {
    if expected != actual {
        return Err(error!(ErrorCode::DiffMismatch));
    }
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Diff hash mismatch")]
    DiffMismatch,
}