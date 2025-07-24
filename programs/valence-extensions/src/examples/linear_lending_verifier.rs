//! Linear lending verifier example - enforces linear operation flow through counter
//!
//! This demonstrates how to implement a verifier that enforces a specific
//! sequence of operations for a lending protocol using linear types.

use anchor_lang::prelude::*;

// Example verifier function that would be implemented in a separate program
// declare_id!("VLinear111111111111111111111111111111111111");

// Operation types (encoded in remaining accounts)
const OP_DEPOSIT: u8 = 0;
const OP_TRANSFER_VOUCHER: u8 = 1;
const OP_ADD_COLLATERAL: u8 = 2;
const OP_WITHDRAW: u8 = 3;

/// Example verification logic for linear lending operations
/// This would be the `verify_account` instruction in a standalone verifier program
pub fn example_verify_linear_lending(
    _account: &AccountInfo,
    _caller: &Signer,
    managed_account_data: &[u8],
    operation: u8,
) -> Result<()> {
    // Get the counter from account metadata (first 4 bytes)
    let counter = u32::from_le_bytes(
        managed_account_data[..4].try_into().unwrap_or([0u8; 4])
    );
    
    // Verify the operation follows the correct linear flow
    let expected_counter = match operation {
        OP_DEPOSIT => 0,
        OP_TRANSFER_VOUCHER => 1,
        OP_ADD_COLLATERAL => 2,
        OP_WITHDRAW => 3,
        _ => return Err(ErrorCode::InvalidOperation.into()),
    };
    
    require_eq!(
        counter,
        expected_counter,
        ErrorCode::InvalidOperationOrder
    );
    
    msg!("Linear operation {} verified for counter {}", operation, counter);
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid operation type")]
    InvalidOperation,
    
    #[msg("Invalid operation order - counter doesn't match expected flow")]
    InvalidOperationOrder,
    
    #[msg("No voucher found in metadata")]
    NoVoucher,
}

// Example of how a lending protocol would use this
// 
// The linear flow ensures:
// 1. Must deposit before doing anything else
// 2. Must transfer voucher after deposit (proves position created)
// 3. Can then add collateral or withdraw
// 4. Each operation advances the counter, creating a linear type
// 
// This prevents:
// - Withdrawing before depositing
// - Transferring voucher before it exists
// - Skipping required steps
// - Replaying operations
// 
// The session's move semantics ensure the entire flow is owned by one party