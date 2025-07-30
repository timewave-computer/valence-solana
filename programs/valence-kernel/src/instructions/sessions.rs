// Session lifecycle management for valence-kernel execution contexts
//
// Sessions form the primary execution context for valence-kernel operations,
// providing isolated environments with dedicated account registries, security
// policies, and namespace organization. This module handles the complete session
// lifecycle from initialization through management to termination.
//
// KERNEL INTEGRATION: Sessions serve as the authorization boundary for batch
// execution, maintaining account borrow states, enforcing security policies
// through guard accounts, and tracking operation history. The kernel routes all
// operations through session contexts for security and isolation.
//
// SECURITY MODEL: Each session maintains its own guard account with security
// policies, account lookup table for pre-validated addresses, and namespace
// isolation. Sessions enforce borrowing semantics and prevent unauthorized
// access to accounts outside their registered scope.

use crate::{
    state::{CreateSessionParams, GuardAccount, Session, SessionAccountLookup, RegisteredAccount, RegisteredProgram},
    errors::KernelError,
};
use anchor_lang::prelude::*;

// ================================
// Guard Account Creation
// ================================

/// Create a new guard account with security policy configuration
/// 
/// This instruction creates a minimal account to store security policy flags.
/// Create a new guard account with security policy
/// 
/// # Errors
/// Returns errors for invalid guard configuration
#[allow(clippy::needless_pass_by_value)]
pub fn create_guard_account(
    ctx: Context<CreateGuardAccount>,
    session: Pubkey,
    allow_unregistered_cpi: bool,
) -> Result<()> {
    let guard_account = &mut ctx.accounts.guard_account;
    
    **guard_account = GuardAccount::new(session, allow_unregistered_cpi);
    
    Ok(())
}

/// Account context for guard account creation
#[derive(Accounts)]
#[instruction(session: Pubkey, allow_unregistered_cpi: bool)]
pub struct CreateGuardAccount<'info> {
    /// The guard account being created with fixed sizing
    #[account(
        init,
        payer = payer,
        space = GuardAccount::space(),
    )]
    pub guard_account: Account<'info, GuardAccount>,
    
    /// The payer for the account creation
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
}

// ================================
// Session Creation
// ================================

/// Create a new session with specified configuration and initial registrations
/// 
/// Sessions provide controlled access to program states.
/// This also creates and populates the associated Account Lookup Table.
/// Create a new session with initial registrations
/// 
/// # Errors
/// Returns errors for invalid session parameters or failed initialization
#[allow(clippy::needless_pass_by_value)]
pub fn create_session_account(
    ctx: Context<CreateSession>,
    shard: Pubkey,
    params: CreateSessionParams,
    initial_borrowable: &[RegisteredAccount],
    initial_programs: &[RegisteredProgram],
) -> Result<()> {
    let session_key = ctx.accounts.session.key();
    let owner_key = ctx.accounts.owner.key();
    let clock = Clock::get()?;

    // Initialize the account lookup table
    let alt = &mut ctx.accounts.account_lookup;
    alt.set_inner(SessionAccountLookup::new(session_key, owner_key));
    
    // Populate initial registrations
    for account in initial_borrowable {
        alt.register_borrowable(account.address, account.permissions, account.label)?;
    }
    
    for program in initial_programs {
        alt.register_program(program.address, program.label)?;
    }
    
    // Now we can take the mutable borrow of session
    let session_account = &mut ctx.accounts.session;

    // Initialize session with reference to account lookup tabke
    session_account.set_inner(Session::new(
        params, 
        owner_key, 
        shard, 
        ctx.accounts.guard_account.key(),
        ctx.accounts.account_lookup.key(),
        &clock
    )?);

    // Log creation details
    msg!("Session created successfully");
    msg!("  Owner: {}", owner_key);
    msg!("  Shard: {}", shard);
    msg!("  Namespace: {}", session_account.namespace.as_str().unwrap_or("<invalid>"));
    if let Some(parent) = session_account.parent_session {
        msg!("  Parent session: {}", parent);
    }
    msg!("  Guard account: {}", session_account.guard_account);
    msg!("  Account lookup: {}", session_account.account_lookup);

    Ok(())
}

/// Account context for session creation
#[derive(Accounts)]
pub struct CreateSession<'info> {
    /// The session account being created
    #[account(
        init, 
        payer = owner, 
        space = Session::calculate_space(),
    )]
    pub session: Box<Account<'info, Session>>,

    /// The account lookup table for this session
    #[account(
        init,
        payer = owner,
        space = SessionAccountLookup::calculate_space(), // Fixed size for all categories
    )]
    pub account_lookup: Box<Account<'info, SessionAccountLookup>>,

    /// Guard account containing the compiled guard program
    /// CHECK: Guard account validation happens in handler
    pub guard_account: AccountInfo<'info>,

    /// The owner creating the session
    #[account(mut)]
    pub owner: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

// ================================
// Account Lookup Table Management
// ================================

/// Unified account lookup table management
/// 
/// This single instruction handles all ALT modifications including adding
/// borrowable accounts, programs, and removing entries.
/// Manage account lookup table entries
/// 
/// # Errors
/// Returns errors for unauthorized modifications or invalid accounts
#[allow(clippy::needless_pass_by_value)]
pub fn manage_alt(
    ctx: Context<ManageALT>,
    add_borrowable: &[RegisteredAccount],
    add_programs: &[RegisteredProgram],
    remove_accounts: &[Pubkey],
) -> Result<()> {
    let alt = &mut ctx.accounts.account_lookup;
    
    // Verify authority
    require_keys_eq!(
        alt.authority,
        ctx.accounts.authority.key(),
        KernelError::Unauthorized
    );
    
    // Add borrowable accounts
    for account in add_borrowable {
        alt.register_borrowable(account.address, account.permissions, account.label)?;
        msg!("Added borrowable account: {}", account.address);
    }
    
    // Add programs
    for program in add_programs {
        alt.register_program(program.address, program.label)?;
        msg!("Added program: {}", program.address);
    }
    
    // Remove accounts (simplified - just mark as inactive)
    for address in remove_accounts {
        // Find and deactivate in borrowable accounts
        for i in 0..alt.borrowable_count as usize {
            if alt.borrowable_accounts[i].address == *address {
                // Move last element to this position and decrement count
                if i < (alt.borrowable_count as usize - 1) {
                    alt.borrowable_accounts[i] = alt.borrowable_accounts[alt.borrowable_count as usize - 1].clone();
                }
                alt.borrowable_count -= 1;
                msg!("Removed borrowable account: {}", address);
                break;
            }
        }
        
        // Find and deactivate in program accounts
        for i in 0..alt.program_count as usize {
            if alt.program_accounts[i].address == *address {
                alt.program_accounts[i].active = false;
                msg!("Deactivated program: {}", address);
                break;
            }
        }
    }
    
    Ok(())
}

/// Account context for unified ALT management
#[derive(Accounts)]
pub struct ManageALT<'info> {
    /// The account lookup table to modify
    #[account(
        mut,
        constraint = account_lookup.session == session.key() @ KernelError::InvalidSessionConfig
    )]
    pub account_lookup: Box<Account<'info, SessionAccountLookup>>,
    
    /// The session that owns this ALT
    pub session: Box<Account<'info, Session>>,
    
    /// Authority that can modify the ALT
    pub authority: Signer<'info>,
}

// ================================
// Session Invalidation
// ================================

/// Invalidate a session for move semantics
/// 
/// This allows clean ownership transfer by preventing the old session
/// from continuing to operate. The new owner must create a new session.
/// Invalidate a session for ownership transfer
/// 
/// # Errors
/// Returns errors for unauthorized invalidation
#[allow(clippy::needless_pass_by_value)]
pub fn invalidate_session(
    ctx: Context<InvalidateSession>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    
    // Only owner can invalidate
    require!(
        session.owner == ctx.accounts.owner.key(),
        KernelError::Unauthorized
    );
    
    // Mark as inactive
    session.active = false;
    
    // Increment nonce to invalidate any cached references
    session.nonce = session.nonce.saturating_add(1);
    
    msg!("Session {} invalidated by owner", ctx.accounts.session.key());
    
    Ok(())
}

/// Account context for session invalidation
#[derive(Accounts)]
pub struct InvalidateSession<'info> {
    /// The session to invalidate
    #[account(mut)]
    pub session: Account<'info, Session>,
    
    /// The owner invalidating the session
    pub owner: Signer<'info>,
}