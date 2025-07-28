// Create session instruction for valence-kernel
// Handles initialization of new sessions with guard configuration
use crate::state::{CreateSessionParams, Session};
use anchor_lang::prelude::*;

// ================================
// Instruction Handler
// ================================

/// Create a new session with specified configuration
/// 
/// Sessions provide controlled access to program states through guards.
/// Each session encapsulates authorization logic and usage tracking.
pub fn create_session_account(
    ctx: Context<CreateSession>,
    shard: Pubkey,
    params: CreateSessionParams,
) -> Result<()> {
    let session_account = &mut ctx.accounts.session;
    let clock = Clock::get()?;

    // Initialize session
    **session_account = Session::new(params, ctx.accounts.owner.key(), shard, &clock);

    // Log creation details
    msg!("Session created successfully");
    msg!("  Owner: {}", ctx.accounts.owner.key());
    msg!("  Shard: {}", shard);
    msg!("  Scope: {:?}", session_account.scope);
    if let Some(binding) = session_account.bound_to {
        msg!("  Bound to: {}", binding);
    }
    msg!("  Guard data: {}", session_account.guard_data);

    Ok(())
}

// ================================
// Account Context
// ================================

/// Account context for session creation
#[derive(Accounts)]
pub struct CreateSession<'info> {
    /// The session account being created
    #[account(
        init, 
        payer = owner, 
        space = Session::calculate_space(),
    )]
    pub session: Account<'info, Session>,

    /// The owner creating the session
    #[account(mut)]
    pub owner: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}