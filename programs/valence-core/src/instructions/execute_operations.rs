// Execute operations instruction for session-based execution
// Handles structured operations with account borrowing
use crate::{
    errors::ValenceError, 
    operations::{self, OperationBatch, SessionOperation},
    state::{Session, GuardData, CPIAllowlist}
};
use anchor_lang::prelude::*;
use anchor_lang::{solana_program, system_program};

// ================================
// Instruction Handler
// ================================

/// Execute a batch of operations through a session
/// 
/// This instruction provides the core functionality for session-based
/// account abstraction, allowing complex operations while maintaining
/// security through guards and borrowing semantics.
pub fn execute_operations(
    ctx: Context<ExecuteOperations>, 
    batch: OperationBatch
) -> Result<()> {
    // Validate the entire batch first
    batch.validate()?;
    
    let caller = ctx.accounts.caller.key();
    let _clock = ctx.accounts.clock.unix_timestamp;
    
    // Simplified guard evaluation using built-in validation
    require!(
        ctx.accounts.guard_data.session == ctx.accounts.session.key(),
        ValenceError::InvalidSessionConfig
    );
    
    // Basic authorization - check if caller is session owner
    require!(
        caller == ctx.accounts.session.owner,
        ValenceError::Unauthorized
    );
    
    // Process the batch of operations
    let session = &mut *ctx.accounts.session;
    for operation in batch.operations.iter() {
        
        match operation {
            SessionOperation::BorrowAccount { account, mode } => {
                process_borrow_account(
                    session, 
                    account, 
                    *mode, 
                    ctx.remaining_accounts, 
                    &ctx.accounts.clock
                )?;
            }
            
            SessionOperation::ReleaseAccount { account } => {
                session.release_account(account, &ctx.accounts.clock)?;
            }
            
            SessionOperation::UpdateMetadata { metadata } => {
                session.set_metadata(*metadata, &ctx.accounts.clock);
            }
            
            SessionOperation::InvokeProgram { manifest_index, data, account_indices } => {
                process_invoke_program(
                    session,
                    *manifest_index,
                    data,
                    account_indices,
                    &batch,
                    ctx.remaining_accounts,
                    &ctx.accounts.cpi_allowlist,
                )?;
            }
            
            SessionOperation::Custom { program_id, discriminator: _, data: _ } => {
                // SECURITY: Validate program against allowlist
                require!(
                    ctx.accounts.cpi_allowlist.is_allowed(program_id),
                    ValenceError::ProgramNotAllowed
                );
                
                // Custom operations are extensions that operate on session context
                // For now, we implement this as a stub that validates the allowlist
                // Full CPI implementation would require careful lifetime management
                msg!("Custom operation validated for program: {}", program_id);
                
                // Extension point for protocol-specific operations
                // Protocol implementations would handle the CPI to program_id
                // with the discriminator and data
            }
        }
    }
    
    if batch.auto_release {
        session.release_all(&ctx.accounts.clock);
    }
    
    // Increment usage for the primary session
    session.increment_usage(&ctx.accounts.clock)?;
    
    Ok(())
}

// ================================
// Helper Functions
// ================================

#[inline(never)]
fn process_borrow_account<'info>(
    session: &mut Session,
    account: &Pubkey,
    mode: u8,
    remaining_accounts: &[AccountInfo<'info>],
    clock: &Clock,
) -> Result<()> {
    let account_info = remaining_accounts
        .iter()
        .find(|a| a.key() == *account)
        .ok_or(ValenceError::InvalidParameters)?;
        
    // SECURITY: Validate account properties
    if mode & operations::MODE_WRITE != 0 {
        require!(account_info.is_writable, ValenceError::AccountNotWritable);
    }
    
    // Validate account is not a program
    require!(
        !account_info.executable,
        ValenceError::InvalidParameters
    );
    
    // Validate account owner if system account expected
    if account_info.owner == &system_program::ID {
        require!(
            account_info.lamports() > 0,
            ValenceError::InvalidParameters
        );
    }
    
    session.borrow_account(*account, mode, clock)
}

#[inline(never)]
fn process_invoke_program<'info>(
    session: &Session,
    manifest_index: u8,
    data: &[u8],
    account_indices: &[u8],
    batch: &OperationBatch,
    remaining_accounts: &[AccountInfo<'info>],
    cpi_allowlist: &CPIAllowlist,
) -> Result<()> {
    let program_entry = batch.program_manifest
        .get(manifest_index as usize)
        .ok_or(ValenceError::InvalidParameters)?;

    // SECURITY: Validate program against allowlist
    require!(
        cpi_allowlist.is_allowed(&program_entry.program_id),
        ValenceError::ProgramNotAllowed
    );
    
    let mut cpi_accounts = Vec::new();
    for &index in account_indices {
        if index >= 8 || (session.borrowed_bitmap & (1 << index)) == 0 {
            return Err(ValenceError::InvalidParameters.into());
        }
        
        let borrowed = &session.borrowed_accounts[index as usize];
        let account_info = remaining_accounts
            .iter()
            .find(|a| a.key() == borrowed.address)
            .ok_or(ValenceError::InvalidParameters)?;
            
        cpi_accounts.push(account_info.clone());
    }
    
    let ix = solana_program::instruction::Instruction {
        program_id: program_entry.program_id,
        accounts: cpi_accounts.iter().map(|a| AccountMeta {
            pubkey: *a.key,
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        }).collect(),
        data: data.to_vec(),
    };
    
    solana_program::program::invoke(&ix, &cpi_accounts)?;
    Ok(())
}


// ================================
// Account Context
// ================================

/// Account context for executing operations
#[derive(Accounts)]
pub struct ExecuteOperations<'info> {
    /// The session being used for operations.
    #[account(mut)]
    pub session: Box<Account<'info, Session>>,
    
    /// The guard data account associated with the primary session.
    pub guard_data: Box<Account<'info, GuardData>>,
    
    /// CPI allowlist for security validation
    pub cpi_allowlist: Box<Account<'info, CPIAllowlist>>,
    
    /// Optional binding session for hierarchical authorization.
    #[account(mut)]
    pub binding_session: Option<Box<Account<'info, Session>>>,
    
    /// Optional guard data for the binding session.
    pub binding_guard_data: Option<Box<Account<'info, GuardData>>>,
    
    /// The caller executing the operations.
    pub caller: Signer<'info>,
    
    /// The Solana clock sysvar, used for time-based guards.
    pub clock: Sysvar<'info, Clock>,
    
    // Remaining accounts are passed to the guard evaluator for CPIs
    // and to the operation processor for InvokeProgram.
}