use anchor_lang::prelude::*;
use crate::{state::*, errors::*};

// ================================
// Session Lifecycle Instructions
// ================================

/// Create a new session (optionally as child of another session)
/// Sessions are containers that enforce linear type semantics for protocol choreography
pub fn create_session(
    ctx: Context<CreateSession>,
    protocol_type: [u8; 32],
    parent_session: Option<Pubkey>,
) -> Result<()> {
    // ===== Parent Session Validation =====
    
    // If creating a child session, verify parent exists and is accessible
    if let Some(parent) = parent_session {
        if let Some(parent_account) = ctx.remaining_accounts.first() {
            // Verify the provided account matches the parent key
            require!(parent_account.key() == parent, CoreError::InvalidParentSession);
            
            // Deserialize and check parent session access rights
            let parent_data = parent_account.try_borrow_data()?;
            if parent_data.len() >= 8 + Session::SIZE {
                let parent_session = Session::try_deserialize(&mut &parent_data[8..])?;
                // Ensure caller has access to parent session
                require!(
                    parent_session.can_access(&ctx.accounts.owner.key()),
                    CoreError::ParentSessionAccessDenied
                );
            }
        } else {
            return Err(CoreError::ParentSessionNotProvided.into());
        }
    }
    
    // ===== Initialize New Session =====
    
    let session = &mut ctx.accounts.session;
    
    // Set PDA bump for future derivation
    session.bump = ctx.bumps.session;
    
    // Initialize ownership and state
    session.owner = ctx.accounts.owner.key();
    session.consumed = false;
    
    // Initialize account management
    session.accounts = Vec::with_capacity(MAX_ACCOUNTS_PER_SESSION);
    
    // Set protocol identification
    session.protocol_type = protocol_type;
    session.created_at = Clock::get()?.unix_timestamp;
    
    // Initialize verification context
    session.verification_data = [0u8; 256];
    session.parent_session = parent_session;
    
    // Log session creation for monitoring
    msg!("Session created for protocol: {:?}, parent: {:?}", 
        &protocol_type[..8],
        parent_session
    );
    Ok(())
}

// ===== Session Ownership Transfer =====

/// Move session (consumes it, making it unusable)
/// Implements linear type semantics - session can only be moved once
pub fn move_session(ctx: Context<MoveSession>, new_owner: Pubkey) -> Result<()> {
    let session = &mut ctx.accounts.session;
    
    // Verify session hasn't already been consumed
    require!(!session.consumed, CoreError::SessionConsumed);
    
    // Verify caller is current owner
    require!(session.owner == ctx.accounts.owner.key(), CoreError::NotOwner);
    
    // Store the old owner for logging
    let old_owner = session.owner;
    
    // Execute move semantics - consume and transfer ownership atomically
    session.consumed = true;
    session.owner = new_owner;
    
    // Log ownership transfer for audit trail
    msg!("Session {} moved from {} to {}", 
        ctx.accounts.session.key(), 
        old_owner, 
        new_owner
    );
    msg!("Session consumed - original owner can no longer use");
    
    Ok(())
}

// ===== Session State Management =====

/// Update session verification data (for sharing state between verifiers)
/// Allows authorized parties to update shared verification context
pub fn update_session_data(
    ctx: Context<UpdateSessionData>,
    data: Vec<u8>,
) -> Result<()> {
    // Validate input size
    require!(data.len() <= 256, CoreError::DataTooLarge);
    
    // Ensure session is still active
    require!(!ctx.accounts.session.consumed, CoreError::SessionConsumed);
    
    // ===== Authorization Check =====
    
    // Allow either the session owner or an account within the session to update
    let is_owner = ctx.accounts.authority.key() == ctx.accounts.session.owner;
    let is_account_in_session = ctx.accounts.session.accounts.iter()
        .any(|&acc| acc == ctx.accounts.authority.key());
    
    require!(
        is_owner || is_account_in_session,
        CoreError::Unauthorized
    );
    
    // ===== Update Verification Data =====
    
    let session = &mut ctx.accounts.session;
    
    // Clear existing data and copy new data
    session.verification_data = [0u8; 256];
    session.verification_data[..data.len()].copy_from_slice(&data);
    
    // Log update for monitoring
    msg!("Session data updated, {} bytes written", data.len());
    Ok(())
}

// ===== Session Maintenance =====

/// Drop expired accounts from session
/// Anyone can call this to clean up expired accounts and save rent
pub fn cleanup_session(ctx: Context<CleanupSession>) -> Result<()> {
    let session = &mut ctx.accounts.session;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Track initial state for reporting
    let initial_count = session.accounts.len();
    
    // Filter out expired accounts
    let mut expired_count = 0;
    session.accounts.retain(|&account_key| {
        // Check each account's expiration status
        match check_account_expiration(ctx.remaining_accounts, account_key, current_time) {
            Ok(is_expired) => {
                if is_expired {
                    expired_count += 1;
                    msg!("Account {} expired", account_key);
                    false // Remove from session
                } else {
                    true // Keep in session
                }
            }
            // If we can't check, keep the account to avoid accidental removal
            Err(_) => true
        }
    });
    
    // Report cleanup results
    msg!("Cleanup complete: {} accounts expired out of {}", 
        expired_count, 
        initial_count
    );
    
    Ok(())
}

// ===== Helper Functions =====

/// Helper function to check if an account is expired
/// Extracts expiration timestamp directly from account data
fn check_account_expiration(
    remaining_accounts: &[AccountInfo],
    account_key: Pubkey,
    current_time: i64,
) -> Result<bool> {
    // Locate the account in the provided list
    let account_info = remaining_accounts
        .iter()
        .find(|acc| acc.key() == account_key)
        .ok_or(CoreError::AccountMismatch)?;
    
    // Borrow account data for reading
    let data = account_info.try_borrow_data()
        .map_err(|_| CoreError::AccountMismatch)?;
    
    // Validate account data size
    if data.len() < 8 + SessionAccount::SIZE {
        return Err(CoreError::AccountMismatch.into());
    }
    
    // Extract expires_at field directly using known offset
    let expires_at_bytes: [u8; 8] = data[SessionAccount::EXPIRES_AT_OFFSET..SessionAccount::EXPIRES_AT_OFFSET + 8]
        .try_into()
        .map_err(|_| CoreError::AccountMismatch)?;
    
    // Convert and compare timestamps
    let expires_at = i64::from_le_bytes(expires_at_bytes);
    Ok(current_time >= expires_at)
}

// ===== Session Cleanup =====

/// Close a consumed session and return rent  
/// Only consumed sessions can be closed to prevent accidental closure
pub fn close_session(ctx: Context<CloseSession>) -> Result<()> {
    // Ensure session has been consumed before allowing closure
    require!(ctx.accounts.session.consumed, CoreError::SessionNotConsumed);
    
    // Log closure for audit trail
    msg!("Session {} closed, rent returned to {}", 
        ctx.accounts.session.key(), 
        ctx.accounts.rent_receiver.key()
    );
    
    Ok(())
}

// ================================
// Account Validation Contexts
// ================================

/// Account validation for session creation
#[derive(Accounts)]
pub struct CreateSession<'info> {
    // Session creator who pays rent and becomes owner
    #[account(mut)]
    pub owner: Signer<'info>,
    
    // New session account to initialize
    #[account(
        init,
        payer = owner,
        space = 8 + Session::SIZE,
        seeds = [SESSION_SEED, session_id.key().as_ref()],
        bump
    )]
    pub session: Account<'info, Session>,
    
    // Unique identifier for session PDA derivation
    /// CHECK: Random pubkey for session ID
    pub session_id: UncheckedAccount<'info>,
    
    // Required for account creation
    pub system_program: Program<'info, System>,
}

/// Account validation for session ownership transfer
#[derive(Accounts)]
pub struct MoveSession<'info> {
    // Current owner initiating the move
    pub owner: Signer<'info>,
    
    // Session to be moved (consumed)
    #[account(
        mut,
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump
    )]
    pub session: Account<'info, Session>,
}

/// Account validation for updating session verification data
#[derive(Accounts)]
pub struct UpdateSessionData<'info> {
    // Authority updating the data (owner or session account)
    pub authority: Signer<'info>,
    
    // Session containing verification data
    #[account(
        mut,
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump
    )]
    pub session: Account<'info, Session>,
}

/// Account validation for session cleanup (permissionless)
#[derive(Accounts)]
pub struct CleanupSession<'info> {
    // Anyone can trigger cleanup to save rent
    pub anyone: Signer<'info>,
    
    // Session to clean expired accounts from
    #[account(
        mut,
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump
    )]
    pub session: Account<'info, Session>,
}

/// Account validation for closing consumed sessions
#[derive(Accounts)]
pub struct CloseSession<'info> {
    // Owner closing the session
    pub owner: Signer<'info>,
    
    // Consumed session to close
    #[account(
        mut,
        close = rent_receiver,
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump,
        has_one = owner @ CoreError::NotOwner
    )]
    pub session: Account<'info, Session>,
    
    // Destination for reclaimed rent
    /// CHECK: Receives returned rent
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,
}