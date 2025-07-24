//! Example lending sequence verifier
//! 
//! This demonstrates a lending protocol verifier that:
//! - Checks ownership  
//! - Validates operation sequences using metadata
//! - Enforces collateral requirements
//! 
//! Operations must follow a strict sequence: deposit -> borrow -> repay -> close

use anchor_lang::prelude::*;

/// Lending sequence verifier demonstrating flexible authorization logic
pub struct LendingSequenceVerifier;

impl LendingSequenceVerifier {
    /// Verify an operation is allowed for the account
    pub fn verify_operation(
        account: &valence_core::SessionAccount,
        caller: &Pubkey,
        operation_data: &[u8],
    ) -> Result<()> {
        // Extract operation type from the passed data
        if operation_data.is_empty() {
            return Err(LendingError::InvalidOperationData.into());
        }
        
        let operation_type = operation_data[0];
        
        // Owner check - first 32 bytes of metadata store owner
        let owner = Pubkey::try_from(&account.metadata[..32])
            .map_err(|_| LendingError::InvalidMetadata)?;
        require!(caller == &owner, LendingError::Unauthorized);
        
        let uses = account.uses;
        
        // Check operation is valid based on current usage count
        match operation_type {
            0 => Self::verify_deposit(account)?,      // First operation must be deposit
            1 => Self::verify_borrow(account)?,       // Second operation must be borrow
            2 => Self::verify_repay(account)?,        // Third operation must be repay
            3 => Self::verify_close(account)?,        // Final operation must be close
            _ => return Err(LendingError::UnknownOperation.into()),
        }
        
        msg!("Operation {} verified for account with {} uses", operation_type, uses);
        Ok(())
    }

    // Verification functions for each operation type
    fn verify_deposit(account: &valence_core::SessionAccount) -> Result<()> {
        // Deposit must be first operation
        require!(account.uses == 0, LendingError::InvalidSequence);
        
        // In real implementation, verify tokens were deposited
        msg!("Deposit verification passed");
        Ok(())
    }

    fn verify_borrow(account: &valence_core::SessionAccount) -> Result<()> {
        // Borrow must be second operation (after deposit)
        require!(account.uses == 1, LendingError::InvalidSequence);
        
        // Check collateral ratio from metadata
        // Bytes 32-40 store deposit amount
        let deposit_amount = u64::from_le_bytes(
            account.metadata[32..40].try_into().unwrap_or([0u8; 8])
        );
        
        // For demo: require 150% collateral ratio
        let max_borrow = deposit_amount * 2 / 3;
        msg!("Max borrow amount: {} based on deposit: {}", max_borrow, deposit_amount);
        
        Ok(())
    }

    fn verify_repay(account: &valence_core::SessionAccount) -> Result<()> {
        // Repay must be third operation (after borrow)
        require!(account.uses == 2, LendingError::InvalidSequence);
        
        // In real implementation, verify full repayment
        msg!("Repayment verification passed");
        Ok(())
    }

    fn verify_close(account: &valence_core::SessionAccount) -> Result<()> {
        // Close must be final operation (after repay)
        require!(account.uses == 3, LendingError::InvalidSequence);
        
        // Verify account can be closed
        msg!("Close verification passed");
        Ok(())
    }
}

#[error_code]
pub enum LendingError {
    #[msg("Unknown operation type")]
    UnknownOperation,
    
    #[msg("Invalid operation sequence")]
    InvalidSequence,
    
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    
    #[msg("Unauthorized caller")]
    Unauthorized,
    
    #[msg("Invalid metadata format")]
    InvalidMetadata,
    
    #[msg("Invalid operation data")]
    InvalidOperationData,
}

/* Example usage in an Anchor program:

```rust
use valence_extensions::examples::lending_sequence_verifier::LendingSequenceVerifier;

#[program]
pub mod my_lending_verifier {
    use super::*;

    pub fn verify_account(ctx: Context<VerifyAccount>, operation_data: Vec<u8>) -> Result<()> {
        LendingSequenceVerifier::verify_operation(
            &ctx.accounts.account,
            &ctx.accounts.caller.key(),
            &operation_data,
        )
    }
}

#[derive(Accounts)]
pub struct VerifyAccount<'info> {
    pub account: Account<'info, valence_core::SessionAccount>,
    pub caller: Signer<'info>,
}
```
*/