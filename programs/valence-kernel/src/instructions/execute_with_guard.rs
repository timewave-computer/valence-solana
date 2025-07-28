// Execute with guard instruction for simple guard evaluation
// Provides a lightweight interface for external programs to validate operations
use crate::{
    errors::KernelError,
    state::{Session, GuardData},
};
use anchor_lang::prelude::*;

// ================================
// Account Context
// ================================

/// Context for executing with guard validation
#[derive(Accounts)]
pub struct ExecuteWithGuard<'info> {
    /// Session being used for the operation
    #[account(mut)]
    pub session: Account<'info, Session>,
    
    /// Guard data containing compiled guard for this session
    #[account(
        constraint = guard_data.session == session.key() @ KernelError::InvalidSessionConfig
    )]
    pub guard_data: Account<'info, GuardData>,
    
    /// Caller requesting the operation
    pub caller: Signer<'info>,
    
    /// Clock sysvar for timestamp-based guards
    pub clock: Sysvar<'info, Clock>,
    
    // remaining_accounts can contain accounts required by external guards
}

// ================================
// Instruction Handler
// ================================

/// Execute with guard validation
/// 
/// This instruction evaluates the session's guard against the provided operation
/// and returns success/failure. It's designed for external programs that need
/// to validate operations through the Valence guard system.
pub fn execute_with_guard(
    ctx: Context<ExecuteWithGuard>,
    operation: Vec<u8>
) -> Result<()> {
    // Validate operation size
    require!(
        operation.len() <= crate::validation::MAX_OPERATION_SIZE,
        KernelError::InvalidParameters
    );
    
    let caller = ctx.accounts.caller.key();
    let clock = &ctx.accounts.clock;
    
    // Basic authorization - check if caller is session owner
    require!(
        caller == ctx.accounts.session.owner,
        KernelError::Unauthorized
    );
    
    // TODO: Implement full guard evaluation
    // For now, just do basic owner check and session validation
    
    // Update session usage if validation passed
    ctx.accounts.session.increment_usage(clock)?;
    
    Ok(())
}