// Global kernel initialization and administration for valence-kernel deployment
//
// This module handles program-wide initialization and global configuration that
// establishes the foundation for all kernel operations. Shard initialization
// sets up the global state required for session management, CPI allowlists,
// and cross-session security policies.
//
// KERNEL LIFECYCLE: Shard initialization is the first step in kernel deployment,
// establishing the program authority, global allowlists, and foundational state
// that enables session creation and operation processing across the entire kernel
// instance.
//
// SECURITY FOUNDATION: The shard system establishes global security policies
// including CPI allowlists and program-wide configuration that applies to all
// sessions and operations within the kernel deployment.

use crate::state::AllowlistAccount;
use anchor_lang::prelude::*;

// ================================
// Shard Initialization
// ================================

/// Initialize the valence-kernel shard program
/// 
/// Sets up the program for operation. Currently performs minimal initialization
/// but provides an extension point for future global state requirements.
/// 
/// # Errors
/// Returns errors for authorization failures
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn initialize_shard(ctx: Context<InitializeShard>) -> Result<()> {
    // Log shard initialization
    msg!(
        "Valence kernel shard program initialized by authority: {}",
        ctx.accounts.authority.key()
    );

    // Future initialization tasks may include:
    // - Global configuration setup
    // - Fee structure initialization
    // - Version information storage

    msg!("Valence kernel shard initialization completed successfully");
    Ok(())
}

/// Account context for program shard initialization
#[derive(Accounts)]
pub struct InitializeShard<'info> {
    /// The authority performing initialization
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program for account operations
    pub system_program: Program<'info, System>,
}

// ================================
// CPI Allowlist Management
// ================================

/// Initialize the CPI allowlist
/// 
/// # Errors
/// Returns errors for initialization failures
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    **allowlist = AllowlistAccount::new(ctx.accounts.authority.key());
    Ok(())
}

/// Add a program to the allowlist
/// 
/// # Errors
/// Returns errors for authorization failures or allowlist full
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn add_program_to_cpi_allowlist(
    ctx: Context<ManageAllowlist>, 
    program_id: Pubkey
) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    allowlist.add_program(program_id)?;
    Ok(())
}

/// Remove a program from the allowlist
/// 
/// # Errors
/// Returns errors for authorization failures or program not found
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn remove_program_from_cpi_allowlist(
    ctx: Context<ManageAllowlist>,
    program_id: Pubkey
) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    allowlist.remove_program(&program_id)?;
    Ok(())
}

/// Initialize allowlist account context
#[derive(Accounts)]
pub struct InitializeAllowlist<'info> {
    #[account(
        init,
        payer = authority,
        space = AllowlistAccount::space(),
        seeds = [b"cpi_allowlist"],
        bump
    )]
    pub cpi_allowlist: Account<'info, AllowlistAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Manage allowlist account context
#[derive(Accounts)]
pub struct ManageAllowlist<'info> {
    #[account(
        mut,
        seeds = [b"cpi_allowlist"],
        bump,
        has_one = authority
    )]
    pub cpi_allowlist: Account<'info, AllowlistAccount>,
    
    pub authority: Signer<'info>,
}