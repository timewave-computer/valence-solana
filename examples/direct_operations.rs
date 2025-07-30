//! Example: Direct Operations for Simple Token Transfer
//! 
//! This example demonstrates when and how to use direct operations for
//! straightforward, well-defined tasks. Direct operations are preferred
//! for their clarity, performance, and simplicity.

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use valence_kernel::instructions::direct_operations::{spl_transfer, SplTransfer};
use std::collections::BTreeMap;

/// Example: Simple SPL token transfer using direct operation
/// 
/// Use Case: Alice wants to send 100 tokens to Bob
/// Why Direct Operation: Single, well-defined action with known accounts
pub fn simple_token_transfer(ctx: Context<SimpleTransfer>, amount: u64) -> Result<()> {
    msg!("Executing direct SPL transfer of {} tokens", amount);
    
    // Validate session authorization
    require!(
        ctx.accounts.authority.key() == ctx.accounts.session.owner,
        valence_kernel::errors::KernelError::Unauthorized
    );
    
    // This example shows the concept - in practice, you'd use the kernel's spl_transfer
    // function with a properly constructed SplTransfer context
    msg!("Transfer of {} tokens would be executed here", amount);
    msg!("From: {}", ctx.accounts.from.key());
    msg!("To: {}", ctx.accounts.to.key());
    msg!("Authority: {}", ctx.accounts.authority.key());
    
    Ok(())
}

#[derive(Accounts)]
pub struct SimpleTransfer<'info> {
    /// The session authorizing this transfer
    pub session: Account<'info, valence_kernel::state::Session>,
    
    /// Authority (must be session owner)
    pub authority: Signer<'info>,
    
    /// Source token account
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    /// Destination token account
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    /// SPL Token program
    pub token_program: Program<'info, Token>,
}

impl<'info> SimpleTransfer<'info> {
    // Helper method to extract accounts for direct CPI
    fn get_transfer_accounts(&self) -> (&Account<'info, TokenAccount>, &Account<'info, TokenAccount>) {
        (&self.from, &self.to)
    }
}

/// Example: Managing Account Lookup Table
/// 
/// Use Case: Register a new token account for future operations
/// Why Direct Operation: Administrative task with predictable structure
pub fn register_token_account(
    ctx: Context<RegisterAccount>,
    permissions: u8,
) -> Result<()> {
    use valence_kernel::state::RegisteredAccount;
    
    msg!("Registering new token account in ALT");
    
    let account_to_add = RegisteredAccount {
        address: ctx.accounts.token_account.key(),
        permissions,
        label: *b"user_token_account              ",
    };
    
    // This example shows the concept - in practice, you'd call manage_alt
    // with a properly constructed ManageALT context
    msg!("Would register account: {} with permissions: {}", 
        account_to_add.address, permissions);
    
    Ok(())
}

#[derive(Accounts)]
pub struct RegisterAccount<'info> {
    #[account(mut)]
    pub account_lookup: Account<'info, valence_kernel::state::SessionAccountLookup>,
    pub session: Account<'info, valence_kernel::state::Session>,
    pub authority: Signer<'info>,
    pub token_account: Account<'info, TokenAccount>,
}

impl<'info> RegisterAccount<'info> {
    // Helper method to validate account relationships
    fn validate_session_authority(&self) -> Result<()> {
        require!(
            self.account_lookup.session == self.session.key(),
            valence_kernel::errors::KernelError::InvalidSessionConfig
        );
        Ok(())
    }
}

/// Example: Session Invalidation for Move Semantics
/// 
/// Use Case: Transfer ownership by invalidating old session
/// Why Direct Operation: Critical security operation that must be atomic
pub fn invalidate_for_transfer(ctx: Context<InvalidateTransfer>) -> Result<()> {
    msg!("Invalidating session {} for ownership transfer", 
        ctx.accounts.session.key());
    
    // Validate ownership
    require!(
        ctx.accounts.owner.key() == ctx.accounts.session.owner,
        valence_kernel::errors::KernelError::Unauthorized
    );
    
    // This example shows the concept - in practice, you'd call invalidate_session
    // with a properly constructed InvalidateSession context
    msg!("Session would be invalidated - ready for ownership transfer");
    Ok(())
}

#[derive(Accounts)]
pub struct InvalidateTransfer<'info> {
    #[account(mut)]
    pub session: Account<'info, valence_kernel::state::Session>,
    pub owner: Signer<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direct_operations_compile() {
        // This test ensures all the example code compiles correctly
        // In a real implementation, you would have integration tests
        println!("Direct operations examples compiled successfully");
    }
}

// Summary: When to Use Direct Operations
// 
// Direct operations are ideal for:
// 1. Simple, well-defined tasks (token transfers, account updates)
// 2. Operations with known, fixed account sets
// 3. Administrative tasks (ALT management, session lifecycle)
// 4. Performance-critical operations
// 
// Benefits:
// - Maximum performance (no batch overhead)
// - Clear intent (single-purpose instructions)
// - Simpler client code
// - Easier to audit and verify

fn main() {
    println!("Valence Direct Operations Example");
    println!("=================================");
    println!();
    println!("This example demonstrates direct operations in Valence:");
    println!("1. Simple token transfers with session authorization");
    println!("2. Account Lookup Table management for pre-approved accounts"); 
    println!("3. Session invalidation for ownership transfer");
    println!();
    println!("Key concepts:");
    println!("- Direct operations provide optimized, single-purpose instructions");
    println!("- They bypass batch execution overhead for common operations");
    println!("- All operations integrate with the session system for security");
    println!("- Account authorization is enforced through session ownership");
    println!();
    println!("See the function implementations above for detailed examples.");
}