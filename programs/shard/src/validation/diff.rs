//! Diff verification logic

use anchor_lang::prelude::*;
use crate::ShardError;

/// Diff entry representing a state change
#[derive(Debug, Clone)]
pub struct DiffEntry {
    /// Previous state hash
    pub prev_hash: [u8; 32],
    /// Operation that caused the change
    pub operation: String,
    /// Result data
    pub result: Vec<u8>,
    /// New state hash
    pub new_hash: [u8; 32],
}

/// Calculate diff hash for state transition
pub fn calculate_diff_hash(
    prev_hash: [u8; 32],
    operation_result: &[u8],
) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    // Create hasher and update with previous hash
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    
    // Add operation result
    hasher.update(operation_result);
    
    // Finalize and return new hash
    let result = hasher.finalize();
    let mut new_hash = [0u8; 32];
    new_hash.copy_from_slice(&result);
    
    new_hash
}

/// Verify diff matches expected value
pub fn verify_diff(
    expected: [u8; 32],
    actual: [u8; 32],
) -> Result<()> {
    require!(
        expected == actual,
        ShardError::DiffMismatch
    );
    Ok(())
}

/// Build a diff entry for a state transition
pub fn build_diff_entry(
    prev_hash: [u8; 32],
    operation: String,
    result: Vec<u8>,
) -> DiffEntry {
    let new_hash = calculate_diff_hash(prev_hash, &result);
    
    DiffEntry {
        prev_hash,
        operation,
        result,
        new_hash,
    }
}

/// Verify a chain of diff entries
pub fn verify_diff_chain(entries: &[DiffEntry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    
    // Verify each entry's hash calculation
    for entry in entries {
        let calculated = calculate_diff_hash(entry.prev_hash, &entry.result);
        require!(
            calculated == entry.new_hash,
            ShardError::DiffMismatch
        );
    }
    
    // Verify chain continuity
    for i in 1..entries.len() {
        require!(
            entries[i].prev_hash == entries[i - 1].new_hash,
            ShardError::DiffMismatch
        );
    }
    
    Ok(())
}

/// Create initial state hash
pub fn initial_state_hash() -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    // Hash of "initial_state" to provide a non-zero starting point
    let mut hasher = Sha256::new();
    hasher.update(b"initial_state");
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Declarative state verification
pub fn verify_expected_state(
    current_hash: [u8; 32],
    operations: &[(String, Vec<u8>)],
    expected_final_hash: [u8; 32],
) -> Result<()> {
    let mut hash = current_hash;
    
    // Apply each operation
    for (op_name, result) in operations {
        msg!("Applying operation: {}", op_name);
        hash = calculate_diff_hash(hash, result);
    }
    
    // Verify final state
    require!(
        hash == expected_final_hash,
        ShardError::DiffMismatch
    );
    
    Ok(())
}