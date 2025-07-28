// Initialize instruction for valence-kernel
// Handles one-time program setup and configuration
use anchor_lang::prelude::*;

// ================================
// Instruction Handler
// ================================

/// Initialize the valence-kernel program
/// 
/// Sets up the program for operation. Currently performs minimal initialization
/// but provides an extension point for future global state requirements.
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    // Log initialization
    msg!(
        "Valence kernel program initialized by authority: {}",
        ctx.accounts.authority.key()
    );

    // Future initialization tasks may include:
    // - Global configuration setup
    // - Fee structure initialization
    // - Version information storage

    msg!("Valence kernel initialization completed successfully");
    Ok(())
}

// ================================
// Account Context
// ================================

/// Account context for program initialization
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The authority performing initialization
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program for account operations
    pub system_program: Program<'info, System>,
}