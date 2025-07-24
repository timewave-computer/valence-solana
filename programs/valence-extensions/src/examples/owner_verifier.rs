//! Simple owner-only verifier example
//! 
//! This demonstrates how to implement a verifier that only allows
//! account usage by a specific owner encoded in the account parameters.

use anchor_lang::prelude::*;

// Example verifier function that would be implemented in a separate program
// declare_id!("VOwner1111111111111111111111111111111111111");

/// Example verification logic for owner-only access
/// This would be the `verify_account` instruction in a standalone verifier program
pub fn example_verify_account_owner(
    _account: &AccountInfo,
    caller: &Signer,
    managed_account_data: &[u8],
) -> Result<()> {
    // Parse owner from first 32 bytes of params in the SessionAccount
    let owner = Pubkey::try_from(&managed_account_data[..32])
        .map_err(|_| ErrorCode::InvalidParams)?;
    
    // Verify caller is the owner
    require_keys_eq!(
        caller.key(),
        owner,
        ErrorCode::Unauthorized
    );
    
    msg!("Account verified for owner {}", caller.key());
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid parameters")]
    InvalidParams,
    
    #[msg("Unauthorized caller")]
    Unauthorized,
}

// Example usage:
// Deploy this verifier
// let verifier_id = deploy_program("owner_verifier.so");
// 
// Create account with owner in params
// let mut params = [0u8; 256];
// params[..32].copy_from_slice(&owner.to_bytes());
// 
// add_account(session, verifier_id, params, lifetime);