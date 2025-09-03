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
    NamespacePath,
    MAX_CASCADE_DEPTH, MAX_BATCH_INVALIDATION_SIZE, MAX_REGISTERED_ACCOUNTS,
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
/// Helper function to minimize stack usage in session creation
fn init_account_lookup_minimal(
    lookup: &mut SessionAccountLookup, 
    session_key: Pubkey, 
    owner_key: Pubkey
) {
    *lookup = SessionAccountLookup::new(session_key, owner_key);
}

/// Helper function to register accounts without stack buildup
fn register_accounts_minimal(
    lookup: &mut SessionAccountLookup,
    borrowable: &[RegisteredAccount],
    programs: &[RegisteredProgram],
) -> Result<()> {
    // Process in small batches to minimize stack frame
    for account in borrowable.iter().take(MAX_REGISTERED_ACCOUNTS) {
        lookup.register_borrowable(account.address, account.permissions, account.label)?;
    }
    for program in programs.iter().take(MAX_REGISTERED_ACCOUNTS) {
        lookup.register_program(program.address, program.label)?;
    }
    Ok(())
}

pub fn create_session_account(
    ctx: Context<CreateSession>,
    shard: Pubkey,
    params: CreateSessionParams,
    initial_borrowable: &[RegisteredAccount],
    initial_programs: &[RegisteredProgram],
) -> Result<()> {
    // Minimize local variables
    let session_key = ctx.accounts.session.key();
    let owner_key = ctx.accounts.owner.key();
    
    // Initialize lookup table with helper to reduce stack
    init_account_lookup_minimal(&mut ctx.accounts.account_lookup, session_key, owner_key);
    
    // Register accounts with helper to reduce stack
    register_accounts_minimal(&mut ctx.accounts.account_lookup, initial_borrowable, initial_programs)?;
    
    // Create session with minimal stack usage
    ctx.accounts.session.set_inner(Session::new(
        params, 
        owner_key, 
        shard, 
        ctx.accounts.guard_account.key(),
        ctx.accounts.account_lookup.key(),
        &Clock::get()?
    )?);

    // Handle parent tracking with boxed session to reduce stack
    if let Some(parent_key) = ctx.accounts.session.parent_session {
        if let Some(parent_info) = ctx.remaining_accounts.first() {
            if parent_info.key() == parent_key {
                let mut data = parent_info.try_borrow_mut_data()?;
                let mut parent = Box::new(Session::try_deserialize_unchecked(&mut data.as_ref())?);
                parent.track_child_session(session_key)?;
                parent.try_serialize(&mut data.as_mut())?;
            }
        }
    }

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

    /// The guard account for security policies
    pub guard_account: Account<'info, GuardAccount>,
    
    /// The owner of the session
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
}

// ================================
// Session Management
// ================================

/// Update the Account Lookup Table
/// 
/// # Errors
/// Returns errors for unauthorized updates or invalid parameters
#[allow(clippy::needless_pass_by_value)]
pub fn manage_alt(
    ctx: Context<ManageAlt>,
    add_borrowable: &[RegisteredAccount],
    add_programs: &[RegisteredProgram],
    remove_accounts: &[Pubkey],
) -> Result<()> {
    // Verify owner
    require!(
        ctx.accounts.session.owner == ctx.accounts.authority.key(),
        KernelError::Unauthorized
    );
    
    let alt = &mut ctx.accounts.account_lookup;
    
    // Add new borrowable accounts (limited to prevent stack overflow)
    for (i, account) in add_borrowable.iter().enumerate() {
        if i >= MAX_REGISTERED_ACCOUNTS { break; }
        alt.register_borrowable(account.address, account.permissions, account.label)?;
    }
    
    // Add new programs (limited to prevent stack overflow)
    for (i, program) in add_programs.iter().enumerate() {
        if i >= MAX_REGISTERED_ACCOUNTS { break; }
        alt.register_program(program.address, program.label)?;
    }
    
    // Remove accounts (limited to prevent stack overflow)
    for (i, account) in remove_accounts.iter().enumerate() {
        if i >= MAX_REGISTERED_ACCOUNTS { break; }
        alt.remove_account(account)?;
    }
    
    msg!("Account lookup table updated");
    msg!("  Added {} borrowable accounts", add_borrowable.len());
    msg!("  Added {} programs", add_programs.len());
    msg!("  Removed {} accounts", remove_accounts.len());
    
    Ok(())
}

/// Account context for ALT management
#[derive(Accounts)]
pub struct ManageAlt<'info> {
    /// The session that owns the ALT
    pub session: Box<Account<'info, Session>>,
    
    /// The ALT to update
    #[account(mut)]
    pub account_lookup: Box<Account<'info, SessionAccountLookup>>,
    
    /// The authority updating the ALT (must be session owner)
    pub authority: Signer<'info>,
}

// ================================
// Session Invalidation
// ================================

/// Invalidate a session, preventing further operations
/// 
/// This is useful for transferring ownership or emergency shutdown.
/// The session can later be reactivated by incrementing the nonce.
/// When a parent session is invalidated, all child sessions are also invalidated.
/// from continuing to operate. The new owner must create a new session.
/// Invalidate a session for ownership transfer with recursive cascading
/// 
/// # Errors
/// Returns errors for unauthorized invalidation
#[allow(clippy::needless_pass_by_value)]
pub fn invalidate_session(
    ctx: Context<InvalidateSession>,
) -> Result<()> {
    let session_key = ctx.accounts.session.key();
    let session = &mut ctx.accounts.session;
    let clock = Clock::get()?;
    
    // Only owner can invalidate
    require!(
        session.owner == ctx.accounts.owner.key(),
        KernelError::Unauthorized
    );
    
    // Store child session data before invalidating the session
    let child_sessions = session.child_sessions;
    let child_session_count = session.child_session_count;
    
    // Mark as inactive
    session.active = false;
    
    // Increment nonce to invalidate any cached references
    session.nonce = session.nonce.saturating_add(1);
    
    msg!("Session {} invalidated by owner", session_key);
    
    // Invalidate all child sessions recursively with depth tracking
    let children_invalidated = invalidate_child_sessions_with_depth(
        &child_sessions, 
        child_session_count, 
        ctx.remaining_accounts,
        MAX_CASCADE_DEPTH,
        session_key
    )?;
    
    // Emit invalidation event
    emit!(SessionInvalidated {
        session: session_key,
        children_invalidated,
        cascade_depth: MAX_CASCADE_DEPTH,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

/// Enhanced recursive invalidation with depth tracking and compute unit management
fn invalidate_child_sessions_with_depth(
    child_sessions: &[Pubkey; 8],
    child_count: u8,
    remaining_accounts: &[AccountInfo],
    remaining_depth: u8,
    parent_session: Pubkey,
) -> Result<u8> {
    // Check if we've reached maximum depth
    if remaining_depth == 0 {
        if child_count > 0 {
            // Emit event for deferred cascade
            let child_keys: Vec<Pubkey> = child_sessions[..child_count as usize].to_vec();
            emit!(CascadeInvalidationRequired {
                parent_session,
                child_sessions: child_keys,
                depth_remaining: 0,
                reason: CascadeDeferReason::MaxDepthReached,
            });
        }
        return Ok(0);
    }
    
    let mut invalidated_count = 0;
    let mut deferred_children = Vec::new();
    
    for &child_session_key in child_sessions.iter().take(child_count as usize) {
        
        // For now, skip compute unit checking as the API isn't available in this Solana version
        // This will be re-enabled when we upgrade to a version that supports runtime compute unit queries
        // TODO: Re-enable compute unit checking when available
        
        // Find the child session in remaining accounts
        let mut found = false;
        for account_info in remaining_accounts {
            if account_info.key() == child_session_key {
                found = true;
                
                // Deserialize child session
                let mut account_data = account_info.try_borrow_mut_data()?;
                let mut child_session = Session::try_deserialize_unchecked(&mut account_data.as_ref())?;
                
                // Only invalidate if still active
                if child_session.active {
                    // Mark as inactive
                    child_session.active = false;
                    child_session.nonce = child_session.nonce.saturating_add(1);
                    invalidated_count += 1;
                    
                    msg!("Child session {} invalidated (depth: {})", child_session_key, MAX_CASCADE_DEPTH - remaining_depth);
                    
                    // Recursively invalidate grandchildren
                    let grandchildren_invalidated = invalidate_child_sessions_with_depth(
                        &child_session.child_sessions,
                        child_session.child_session_count,
                        remaining_accounts,
                        remaining_depth - 1,
                        child_session_key
                    )?;
                    
                    invalidated_count += grandchildren_invalidated;
                    
                    // Serialize back
                    child_session.try_serialize(&mut account_data.as_mut())?;
                }
                
                break;
            }
        }
        
        // If child account not found, add to deferred list
        if !found {
            deferred_children.push(child_session_key);
        }
    }
    
    // Emit event for any deferred children due to missing accounts
    if !deferred_children.is_empty() {
        emit!(CascadeInvalidationRequired {
            parent_session,
            child_sessions: deferred_children,
            depth_remaining: remaining_depth - 1,
            reason: CascadeDeferReason::ChildAccountNotFound,
        });
    }
    
    Ok(invalidated_count)
}

// Legacy function removed to avoid dead code warnings
// If needed for backwards compatibility, can be re-enabled

/// Account context for session invalidation
#[derive(Accounts)]
pub struct InvalidateSession<'info> {
    /// The session to invalidate
    #[account(mut)]
    pub session: Account<'info, Session>,
    
    /// The owner invalidating the session
    pub owner: Signer<'info>,
}

// ================================
// Batch Session Invalidation
// ================================

/// Invalidate multiple sessions in a single batch with DoS protection
/// 
/// This function allows invalidating multiple sessions efficiently while preventing
/// compute unit exhaustion through careful batching and limits.
/// 
/// # Errors
/// Returns errors for unauthorized access, batch size limits, or compute exhaustion
#[allow(clippy::needless_pass_by_value)]
pub fn invalidate_session_batch(
    ctx: Context<InvalidateSessionBatch>,
    session_keys: &[Pubkey],
) -> Result<()> {
    let _clock = Clock::get()?;
    let authority = ctx.accounts.authority.key();
    
    // Enforce batch size limits to prevent DoS
    require!(
        session_keys.len() <= MAX_BATCH_INVALIDATION_SIZE,
        KernelError::InvalidParameters
    );
    
    let mut invalidated_count = 0;
    // TODO: Re-enable compute unit tracking when API is available
    let _start_compute_units = 0u64; // Placeholder for now
    
    // Process each session in the batch
    for &session_key in session_keys.iter() {
        // TODO: Re-enable compute unit checking when available
        // For now, process all sessions without early termination
        
        // Find the session in remaining accounts
        for account_info in ctx.remaining_accounts.iter() {
            if account_info.key() == session_key {
                // Verify the account is mutable and owned by the program
                if !account_info.is_writable || account_info.owner != &crate::ID {
                    continue;
                }
                
                // Deserialize session
                let mut account_data = account_info.try_borrow_mut_data()?;
                let mut session = Session::try_deserialize_unchecked(&mut account_data.as_ref())?;
                
                // Verify authority (only owner can invalidate)
                if session.owner != authority {
                    continue;
                }
                
                // Only process if session is still active
                if session.active {
                    // Store child data before invalidation
                    let child_sessions = session.child_sessions;
                    let child_count = session.child_session_count;
                    
                    // Mark session as inactive
                    session.active = false;
                    session.nonce = session.nonce.saturating_add(1);
                    invalidated_count += 1;
                    
                    msg!("Batch invalidated session {}", session_key);
                    
                    // Attempt limited cascade for immediate children only
                    // (deep cascading should use separate transactions)
                    let children_invalidated = invalidate_child_sessions_with_depth(
                        &child_sessions,
                        child_count,
                        ctx.remaining_accounts,
                        1, // Only 1 level deep in batch operations
                        session_key
                    )?;
                    
                    invalidated_count += u32::from(children_invalidated);
                    
                    // Serialize back to account
                    session.try_serialize(&mut account_data.as_mut())?;
                }
                
                break;
            }
        }
    }
    
    let compute_units_used = 0u64; // TODO: Calculate actual compute units when API is available
    
    // Emit batch completion event
    emit!(BatchInvalidated {
        parent: ctx.accounts.authority.key(),
        invalidated: invalidated_count,
        total: session_keys.len() as u32,
        compute_units_used,
    });
    
    msg!("Batch invalidation completed: {}/{} sessions invalidated", invalidated_count, session_keys.len());
    
    Ok(())
}

/// Account context for batch session invalidation
#[derive(Accounts)]
pub struct InvalidateSessionBatch<'info> {
    /// The authority invalidating the sessions (must be owner of each session)
    pub authority: Signer<'info>,
}

// ================================
// Cascading Invalidation Events
// ================================

/// Event emitted when a session is invalidated
#[event]
pub struct SessionInvalidated {
    /// The session that was invalidated
    pub session: Pubkey,
    /// Number of direct children invalidated
    pub children_invalidated: u8,
    /// Maximum cascade depth attempted
    pub cascade_depth: u8,
    /// Timestamp of invalidation
    pub timestamp: i64,
}

/// Event emitted when cascade invalidation is required but couldn't complete due to limits
#[event]
pub struct CascadeInvalidationRequired {
    /// Parent session that triggered the cascade
    pub parent_session: Pubkey,
    /// Child sessions that need invalidation
    pub child_sessions: Vec<Pubkey>,
    /// Remaining depth for cascade
    pub depth_remaining: u8,
    /// Reason cascade was deferred
    pub reason: CascadeDeferReason,
}

/// Event emitted when a batch of sessions is invalidated
#[event]
pub struct BatchInvalidated {
    /// Parent session that triggered the batch
    pub parent: Pubkey,
    /// Number of sessions successfully invalidated
    pub invalidated: u32,
    /// Total sessions attempted
    pub total: u32,
    /// Compute units consumed
    pub compute_units_used: u64,
}

/// Reasons why cascade invalidation might be deferred
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CascadeDeferReason {
    /// Not enough compute units remaining
    ComputeUnitsExhausted,
    /// Maximum cascade depth reached
    MaxDepthReached,
    /// Child session account not found in remaining_accounts
    ChildAccountNotFound,
    /// Transaction size limits
    TransactionTooLarge,
}

// ================================
// Session Queries (View Functions)
// ================================

/// Get current session state
/// 
/// # Errors
/// Returns errors for invalid session
#[allow(clippy::needless_pass_by_value)]
pub fn get_session_info(ctx: Context<GetSessionInfo>) -> Result<SessionInfo> {
    let session = &ctx.accounts.session;
    
    Ok(SessionInfo {
        namespace: session.namespace.clone(),
        owner: session.owner,
        shard: session.shard,
        guard_account: session.guard_account,
        account_lookup: session.account_lookup,
        parent_session: session.parent_session,
        usage_count: session.usage_count,
        created_at: session.created_at,
        updated_at: session.updated_at,
        active: session.active,
        nonce: session.nonce,
        borrowed_count: session.borrowed_bitmap.count_ones() as u8,
        child_count: session.child_count,
        child_session_count: session.child_session_count,
    })
}

/// Session information return type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionInfo {
    pub namespace: NamespacePath,
    pub owner: Pubkey,
    pub shard: Pubkey,
    pub guard_account: Pubkey,
    pub account_lookup: Pubkey,
    pub parent_session: Option<Pubkey>,
    pub usage_count: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub active: bool,
    pub nonce: u64,
    pub borrowed_count: u8,
    pub child_count: u8,
    pub child_session_count: u8,
}

/// Account context for session info query
#[derive(Accounts)]
pub struct GetSessionInfo<'info> {
    pub session: Account<'info, Session>,
}