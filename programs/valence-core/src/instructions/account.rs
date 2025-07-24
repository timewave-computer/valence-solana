use anchor_lang::prelude::*;
use crate::{state::*, errors::*, events::*, instructions::{verify_account, VerifyAccount}};

// ================================
// Account Lifecycle Instructions
// ================================

/// Add an account to the session with verifier
/// Creates a new SessionAccount with delegated authorization to a verifier program
pub fn add_account(
    ctx: Context<AddAccount>, 
    verifier: Pubkey,
    max_uses: u32,
    lifetime_seconds: i64,
    metadata: Option<Vec<u8>>,
) -> Result<()> {
    // ===== Validation =====
    
    // Ensure session is still active
    require!(!ctx.accounts.session.consumed, CoreError::SessionConsumed);
    
    // Check session capacity
    require!(!ctx.accounts.session.is_full(), CoreError::TooManyAccounts);
    
    // ===== Initialize Account State =====
    
    let account = &mut ctx.accounts.account;
    
    // Set PDA bump for future derivation
    account.bump = ctx.bumps.account;
    
    // Link to parent session and verifier
    account.session = ctx.accounts.session.key();
    account.verifier = verifier;
    
    // Initialize security counter for replay protection
    account.nonce = 0;
    
    // Configure lifecycle parameters
    account.uses = 0;
    account.max_uses = max_uses;
    account.expires_at = Clock::get()?.unix_timestamp + lifetime_seconds;
    account.created_at = Clock::get()?.unix_timestamp;
    
    // Store optional metadata for verifier use
    account.metadata = [0u8; 64];
    if let Some(data) = metadata {
        let len = data.len().min(64);
        account.metadata[..len].copy_from_slice(&data[..len]);
    }
    
    // ===== Update Session State =====
    
    // Register account in parent session
    ctx.accounts.session.accounts.push(ctx.accounts.account.key());
    
    // Log account creation for monitoring
    msg!("Account created with verifier: {}, max uses: {}", 
        verifier, 
        max_uses
    );
    
    Ok(())
}

// ===== Account Usage with Authorization =====

/// Use an account with verifier authorization
/// Delegates authorization to external verifier via CPI
pub fn use_account<'info>(
    ctx: Context<'_, '_, '_, 'info, UseAccount<'info>>,
    operation_data: Vec<u8>,
) -> Result<()> {
    let account = &ctx.accounts.account;
    
    // ===== Pre-Authorization Checks =====
    
    // Verify account hasn't expired or exceeded usage limits
    require!(account.is_active(), CoreError::AccountExpired);
    
    // ===== Replay Protection =====
    
    // Extract and validate nonce from operation data
    require!(
        operation_data.len() >= 8,
        CoreError::InvalidOperationData
    );
    let expected_nonce = u64::from_le_bytes(
        operation_data[..8].try_into().unwrap()
    );
    require!(
        expected_nonce == account.nonce,
        CoreError::InvalidNonce
    );
    
    // ===== Verifier Authorization via CPI =====
    
    // Prepare CPI context for verifier call
    let cpi_accounts = VerifyAccount {
        account: ctx.accounts.account.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts,
    ).with_remaining_accounts(ctx.remaining_accounts.to_vec());
    
    // Delegate authorization decision to verifier
    verify_account(cpi_ctx)?;
    
    // ===== Post-Authorization State Update =====
    
    let account = &mut ctx.accounts.account;
    
    // Increment nonce to prevent replay
    account.nonce = account.nonce.checked_add(1).ok_or(CoreError::NonceOverflow)?;
    
    // Track usage for lifecycle management
    account.uses = account.uses.checked_add(1).ok_or(CoreError::UsageOverflow)?;
    
    // Log successful usage for monitoring
    msg!("Account used successfully, nonce: {}, uses: {}", 
        account.nonce, 
        account.uses
    );
    
    Ok(())
}

// ===== Conditional Account Usage =====

/// Use account conditionally based on a simple condition check
/// Allows protocols to implement conditional logic without custom verifiers
pub fn use_account_if(ctx: Context<UseAccountIf>, condition_type: u8, condition_value: u64) -> Result<()> {
    let account = &ctx.accounts.account;
    let clock = Clock::get()?;
    
    // ===== Expiration Check =====
    
    // Ensure account hasn't expired
    require!(
        clock.unix_timestamp < account.expires_at,
        CoreError::AccountExpired
    );
    
    // ===== Condition Evaluation =====
    
    // Evaluate the specified condition type
    let should_use = match condition_type {
        // Condition 0: Usage count less than threshold
        0 => account.uses < condition_value as u32,
        
        // Condition 1: Account age less than threshold (seconds)
        1 => (clock.unix_timestamp - account.created_at) < condition_value as i64,
        
        // Condition 2: Usage count equals specific value
        2 => account.uses == condition_value as u32,
        
        // Condition 3: Metadata value equals condition
        3 => {
            // Extract first 8 bytes of metadata as u64
            let metadata_value = u64::from_le_bytes(
                account.metadata[..8].try_into().unwrap_or([0u8; 8])
            );
            metadata_value == condition_value
        },
        
        // Invalid condition type
        _ => return Err(CoreError::InvalidCondition.into()),
    };
    
    // ===== Conditional Execution =====
    
    if should_use {
        // Prepare CPI to verifier for authorization
        let cpi_accounts = VerifyAccount {
            account: ctx.accounts.account.to_account_info(),
            caller: ctx.accounts.caller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.verifier_program.to_account_info(),
            cpi_accounts,
        );
        
        // Verify authorization
        verify_account(cpi_ctx)?;
        
        // Update usage count on success
        let account = &mut ctx.accounts.account;
        account.uses = account.uses.checked_add(1).ok_or(CoreError::UsageOverflow)?;
        
        msg!("Conditional use succeeded for account {}", ctx.accounts.account.key());
    } else {
        msg!("Condition not met, account not used");
    }
    
    Ok(())
}

// ===== Account Metadata Management =====

/// Update account metadata (protocols can store voucher IDs, path info, etc)
/// Allows authorized parties to update the 64-byte metadata field
pub fn update_account_metadata(ctx: Context<UpdateAccountMetadata>, metadata: Vec<u8>) -> Result<()> {
    // Validate metadata size
    require!(metadata.len() <= 64, CoreError::MetadataTooLarge);
    
    let account = &mut ctx.accounts.account;
    
    // Clear existing metadata and copy new data
    account.metadata = [0u8; 64];
    account.metadata[..metadata.len()].copy_from_slice(&metadata);
    
    Ok(())
}

// ===== Account Cleanup =====

/// Close an expired account and return rent
/// Allows rent recovery from expired accounts or owner-authorized closure
pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
    let account = &ctx.accounts.account;
    let clock = Clock::get()?;
    
    // ===== Authorization Check =====
    
    // Account can be closed if:
    // 1. It has expired, OR
    // 2. The session owner authorizes closure
    let can_close = clock.unix_timestamp >= account.expires_at 
        || ctx.accounts.session.owner == ctx.accounts.authority.key();
    
    require!(can_close, CoreError::CannotClose);
    
    // Log closure for audit trail
    msg!("Account {} closed, rent returned to {}", 
        ctx.accounts.account.key(), 
        ctx.accounts.rent_receiver.key()
    );
    
    Ok(())
}

// ===== Convenience Helper Functions =====

/// Create a single-participant session (common case)
/// This helper creates both session and account in one transaction
pub fn create_account_with_session(
    ctx: Context<CreateAccountWithSession>,
    protocol_type: [u8; 32],
    verifier: Pubkey,
    max_uses: u32,
    lifetime_seconds: i64,
    metadata: Option<Vec<u8>>,
) -> Result<()> {
    // Store keys before mutable borrows
    let session_key = ctx.accounts.session.key();
    let account_key = ctx.accounts.account.key();
    
    // ===== Initialize Session =====
    
    let session = &mut ctx.accounts.session;
    
    // Set PDA bump and ownership
    session.bump = ctx.bumps.session;
    session.owner = ctx.accounts.owner.key();
    
    // Initialize for single account
    session.accounts = Vec::with_capacity(1);
    session.consumed = false;
    
    // Set protocol and timing
    session.created_at = Clock::get()?.unix_timestamp;
    session.protocol_type = protocol_type;
    
    // Initialize verification context
    session.verification_data = [0u8; 256];
    session.parent_session = None;
    
    // ===== Initialize Account =====
    
    let account = &mut ctx.accounts.account;
    
    // Set PDA bump and relationships
    account.bump = ctx.bumps.account;
    account.session = session_key;
    account.verifier = verifier;
    
    // Initialize security counter
    account.nonce = 0;
    
    // Configure lifecycle
    account.uses = 0;
    account.max_uses = max_uses;
    account.expires_at = Clock::get()?.unix_timestamp + lifetime_seconds;
    account.created_at = Clock::get()?.unix_timestamp;
    
    // Store optional metadata
    account.metadata = [0u8; 64];
    if let Some(data) = metadata {
        let len = data.len().min(64);
        account.metadata[..len].copy_from_slice(&data[..len]);
    }
    
    // ===== Link Account to Session =====
    
    session.accounts.push(account_key);
    
    // ===== Emit Creation Event =====
    
    emit!(AccountCreated {
        account: account_key,
        session: session_key,
        protocol_type,
        verifier,
    });
    
    msg!("Created single-account session for protocol: {:?}", &protocol_type[..8]);
    Ok(())
}

// ================================
// Account Validation Contexts
// ================================

// ===== Account Creation Context =====

/// Validation context for adding new accounts to a session
#[derive(Accounts)]
pub struct AddAccount<'info> {
    // Session owner who can add accounts
    #[account(mut)]
    pub owner: Signer<'info>,
    
    // Parent session that will manage the new account
    #[account(
        mut,
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump,
        has_one = owner @ CoreError::NotOwner
    )]
    pub session: Account<'info, Session>,
    
    // New account to be created and linked to session
    #[account(
        init,
        payer = owner,
        space = 8 + SessionAccount::SIZE,
        seeds = [ACCOUNT_SEED, session.key().as_ref(), account_id.key().as_ref()],
        bump
    )]
    pub account: Account<'info, SessionAccount>,
    
    // Unique identifier for account PDA derivation
    /// CHECK: Random pubkey for account ID
    pub account_id: UncheckedAccount<'info>,
    
    // Required for account creation
    pub system_program: Program<'info, System>,
}

// ===== Account Usage Context =====

/// Validation context for using an account with verifier authorization
#[derive(Accounts)]
pub struct UseAccount<'info> {
    // Entity requesting to use the account
    pub caller: Signer<'info>,
    
    // Account to be used (will be updated with new nonce)
    #[account(mut)]
    pub account: Account<'info, SessionAccount>,
    
    // External verifier program that authorizes usage
    /// CHECK: Verifier program
    pub verifier_program: UncheckedAccount<'info>,
}

// ===== Conditional Usage Context =====

/// Validation context for conditional account usage
#[derive(Accounts)]
pub struct UseAccountIf<'info> {
    // Entity requesting conditional use
    pub caller: Signer<'info>,
    
    // Account to be conditionally used
    #[account(mut)]
    pub account: Account<'info, SessionAccount>,
    
    // Verifier for authorization if condition passes
    /// CHECK: Verifier program
    pub verifier_program: UncheckedAccount<'info>,
}

// ===== Metadata Update Context =====

/// Validation context for updating account metadata
#[derive(Accounts)]
pub struct UpdateAccountMetadata<'info> {
    // Authority allowed to update metadata (must be authorized)
    pub authority: Signer<'info>,
    
    // Account whose metadata will be updated
    #[account(mut)]
    pub account: Account<'info, SessionAccount>,
}

// ===== Account Closure Context =====

/// Validation context for closing expired accounts
#[derive(Accounts)]
pub struct CloseAccount<'info> {
    // Authority closing the account (owner or anyone if expired)
    pub authority: Signer<'info>,
    
    // Account to close and reclaim rent from
    #[account(
        mut,
        close = rent_receiver,
        seeds = [ACCOUNT_SEED, session.key().as_ref(), account.key().as_ref()],
        bump = account.bump
    )]
    pub account: Account<'info, SessionAccount>,
    
    // Parent session for ownership verification
    #[account(
        seeds = [SESSION_SEED, session.key().as_ref()],
        bump = session.bump
    )]
    pub session: Account<'info, Session>,
    
    // Destination for reclaimed rent
    /// CHECK: Receives returned rent
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,
}

// ===== Combined Creation Context =====

/// Validation context for creating session and account together
#[derive(Accounts)]
pub struct CreateAccountWithSession<'info> {
    // Creator who pays rent and owns the session
    #[account(mut)]
    pub owner: Signer<'info>,
    
    // New session to create
    #[account(
        init,
        payer = owner,
        space = 8 + Session::SIZE,
        seeds = [SESSION_SEED, session_id.key().as_ref()],
        bump
    )]
    pub session: Account<'info, Session>,
    
    // New account to create within the session
    #[account(
        init,
        payer = owner,
        space = 8 + SessionAccount::SIZE,
        seeds = [ACCOUNT_SEED, session.key().as_ref(), account_id.key().as_ref()],
        bump
    )]
    pub account: Account<'info, SessionAccount>,
    
    // Unique identifier for session PDA
    /// CHECK: Random pubkey for session ID
    pub session_id: UncheckedAccount<'info>,
    
    // Unique identifier for account PDA
    /// CHECK: Random pubkey for account ID
    pub account_id: UncheckedAccount<'info>,
    
    // Required for account creation
    pub system_program: Program<'info, System>,
}