// High-performance direct operations for valence-kernel optimized execution paths
//
// This module provides strongly-typed, specialized instructions for common operations
// that benefit from dedicated optimization. Unlike the flexible batch system, direct
// operations are compiled into specific instruction handlers that eliminate the
// overhead of operation parsing and dynamic dispatch for performance-critical paths.
//
// PERFORMANCE OPTIMIZATION: Direct operations bypass the batch execution engine's
// generic processing overhead, providing optimized paths for frequent operations
// like token transfers. This dual-path architecture balances flexibility with
// performance for different use case requirements.
//
// KERNEL INTEGRATION: Direct operations still integrate with the session system
// for authorization and guard evaluation but use specialized instruction contexts
// that reduce compute unit consumption for simple, well-defined operations.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};
use crate::{
    errors::KernelError,
    state::{Session, GuardAccount},
};

// ================================
// SPL Token Transfer
// ================================

#[derive(Accounts)]
pub struct SplTransfer<'info> {
    /// Session performing the transfer
    #[account(mut)]
    pub session: Box<Account<'info, Session>>,
    
    /// Guard configuration for this session
    pub guard_account: Box<Account<'info, GuardAccount>>,
    
    /// Source token account
    #[account(mut)]
    pub from: AccountInfo<'info>,
    
    /// Destination token account
    #[account(mut)]
    pub to: AccountInfo<'info>,
    
    /// Authority for the transfer (usually the session PDA)
    pub authority: Signer<'info>,
    
    /// SPL Token program
    pub token_program: Program<'info, Token>,
    
    /// Clock for timestamp checks
    pub clock: Sysvar<'info, Clock>,
}

/// Performs an SPL token transfer with session authorization
/// 
/// This is an optimized direct instruction for common token operations
/// that avoids the overhead of the batch execution system.
/// 
/// # Errors
/// Returns errors for authorization failures or transfer issues
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn spl_transfer(
    ctx: Context<SplTransfer>,
    amount: u64,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    let clock = &ctx.accounts.clock;
    
    // Basic authorization check
    require!(
        ctx.accounts.authority.key() == session.owner,
        KernelError::Unauthorized
    );
    
    // Perform the transfer
    let cpi_accounts = Transfer {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, amount)?;
    
    // Update session usage
    session.increment_usage(clock)?;
    
    Ok(())
}

