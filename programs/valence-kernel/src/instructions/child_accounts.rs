// Child account creation for valence-kernel session management
//
// This module provides dedicated instructions for creating child accounts within
// session namespaces. Child accounts are derived accounts that belong to a session's
// namespace hierarchy and can be managed through the batch execution system.
//
// SEPARATION OF CONCERNS: Child account creation is separated from batch operations
// to avoid complex lifetime management and provide cleaner instruction contexts.
// This follows Anchor best practices for distinct operation types.
//
// SECURITY MODEL: Child accounts inherit namespace permissions from their parent
// session and are tracked for proper cleanup and management. PDA derivation ensures
// deterministic addressing and prevents account conflicts.

use anchor_lang::prelude::*;
use crate::{
    errors::KernelError,
    state::Session,
};

// ================================
// Create Child Account Instruction
// ================================

/// Create a child account within the session's namespace
/// 
/// Child accounts are PDA-derived accounts that belong to a session's namespace
/// hierarchy. They can be used for temporary storage, metadata, or other
/// session-specific data that needs dedicated account space.
/// 
/// # Errors
/// Returns errors for invalid namespace paths, insufficient funds, or PDA conflicts
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn create_child_account(
    ctx: Context<CreateChildAccount>,
    namespace_suffix: String,
    initial_lamports: u64,
    space: u64,
    owner_program: Pubkey,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    let clock = Clock::get()?;
    
    // Validate input parameters
    require!(
        !namespace_suffix.is_empty() && namespace_suffix.len() <= 32,
        KernelError::InvalidParameters
    );
    
    require!(
        !namespace_suffix.contains('/'),
        KernelError::InvalidParameters
    );
    
    require!(
        space > 0 && space < 10_000_000, // 10MB max
        KernelError::InvalidParameters
    );
    
    // Create child namespace path
    let child_namespace = session.namespace.child(&namespace_suffix)?;
    
    // Derive expected PDA for the child account
    let (expected_child_pda, bump) = crate::namespace::Namespace::derive_pda(
        &child_namespace, 
        &crate::ID
    );
    
    // Verify the provided child account matches the expected PDA
    require_keys_eq!(
        ctx.accounts.child_account.key(),
        expected_child_pda,
        KernelError::InvalidParameters
    );
    
    // Verify the account is uninitialized
    require!(
        ctx.accounts.child_account.lamports() == 0 && 
        ctx.accounts.child_account.data_is_empty(),
        KernelError::AccountAlreadyExists
    );
    
    // Calculate total lamports needed (rent + initial funding)
    let rent = ctx.accounts.rent.minimum_balance(space.try_into().map_err(|_| KernelError::InvalidParameters)?);
    let total_lamports = rent.saturating_add(initial_lamports);
    
    // Create the account using system program CPI with PDA seeds
    let namespace_path_bytes = &child_namespace.path[..child_namespace.len as usize];
    let seeds = &[
        crate::namespace::Namespace::SEED_PREFIX,
        namespace_path_bytes,
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    anchor_lang::system_program::create_account(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.child_account.to_account_info(),
            },
            signer_seeds,
        ),
        total_lamports,
        space,
        &owner_program,
    )?;
    
    // Track the child account in the session for cleanup
    session.track_child_account(expected_child_pda)?;
    
    // Update session metadata
    session.increment_usage(&clock)?;
    
    msg!("Created child account: {}", expected_child_pda);
    msg!("  Namespace: {}", child_namespace.as_str().unwrap_or("<invalid>"));
    msg!("  Space: {} bytes", space);
    msg!("  Total lamports: {}", total_lamports);
    msg!("  Owner program: {}", owner_program);
    
    Ok(())
}

/// Account context for child account creation
#[derive(Accounts)]
#[instruction(namespace_suffix: String)]
pub struct CreateChildAccount<'info> {
    /// Session that will own the child account
    #[account(
        mut,
        constraint = session.active @ KernelError::SessionInactive
    )]
    pub session: Account<'info, Session>,
    
    /// Child account being created (must be uninitialized PDA)
    /// CHECK: PDA validation is performed in the instruction handler
    #[account(mut)]
    pub child_account: AccountInfo<'info>,
    
    /// Payer for the account creation and rent
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
    
    /// Rent sysvar for rent calculation
    pub rent: Sysvar<'info, Rent>,
}

// ================================
// Close Child Account Instruction
// ================================

/// Close a child account and return lamports to the session owner
/// 
/// This instruction allows cleanup of child accounts when they're no longer
/// needed, recovering the rent and any remaining lamports.
/// 
/// # Errors
/// Returns errors for unauthorized access or invalid child accounts
#[allow(clippy::needless_pass_by_value)] // Anchor requires owned Context
pub fn close_child_account(
    ctx: Context<CloseChildAccount>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    let child_key = ctx.accounts.child_account.key();
    
    // Verify this is actually a child account of the session
    require!(
        session.is_child_account(&child_key),
        KernelError::Unauthorized
    );
    
    // Remove from session tracking
    session.untrack_child_account(child_key)?;
    
    // Transfer all lamports to the receiver
    let child_lamports = ctx.accounts.child_account.lamports();
    **ctx.accounts.child_account.try_borrow_mut_lamports()? -= child_lamports;
    **ctx.accounts.lamport_receiver.try_borrow_mut_lamports()? += child_lamports;
    
    // Zero out the account data
    let mut child_data = ctx.accounts.child_account.try_borrow_mut_data()?;
    child_data.fill(0);
    
    msg!("Closed child account: {}", child_key);
    msg!("  Recovered lamports: {}", child_lamports);
    
    Ok(())
}

/// Account context for closing child accounts
#[derive(Accounts)]
pub struct CloseChildAccount<'info> {
    /// Session that owns the child account
    #[account(
        mut,
        constraint = session.owner == authority.key() @ KernelError::Unauthorized
    )]
    pub session: Account<'info, Session>,
    
    /// Child account being closed
    /// CHECK: Child account validation is performed in the instruction handler
    #[account(mut)]
    pub child_account: AccountInfo<'info>,
    
    /// Authority that can close the child account (session owner)
    pub authority: Signer<'info>,
    
    /// Account that will receive the recovered lamports
    /// CHECK: Lamport receiver can be any account
    #[account(mut)]
    pub lamport_receiver: AccountInfo<'info>,
}