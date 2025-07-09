/// Session instruction handlers
/// This file contains all the instruction handlers for session operations
use anchor_lang::prelude::*;
use crate::error::ValenceSessionError;
use crate::sessions::state::{SessionState, SessionMetadata, SessionData};
use anchor_spl::token::{self, Transfer, Approve};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::TokenAccount;
use anchor_lang::system_program;

/// Initialize a new session
#[derive(Accounts)]
#[instruction(session_id: String)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + SessionState::get_space(
            session_id.len(),
            &[], // Default empty namespaces
            &SessionMetadata::default(),
        ),
        seeds = [b"session", owner.key().as_ref(), session_id.as_bytes()],
        bump
    )]
    pub session_state: Account<'info, SessionState>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Execute a call instruction
#[derive(Accounts)]
pub struct ExecuteCall<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Create a token account
#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    pub mint: Account<'info, anchor_spl::token::Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = session_state,
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// Transfer tokens
#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

/// Transfer SOL
#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    /// CHECK: Any account can receive SOL
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

/// Approve token spending
#[derive(Accounts)]
pub struct ApproveToken<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: Any account can be a delegate
    pub delegate: AccountInfo<'info>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

/// Store data in session
#[derive(Accounts)]
#[instruction(key: String)]
pub struct StoreData<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub caller: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + SessionData::get_space(key.len(), 1024),
        seeds = [b"session_data", session_state.key().as_ref(), key.as_bytes()],
        bump
    )]
    pub session_data: Account<'info, SessionData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Retrieve data from session
#[derive(Accounts)]
#[instruction(key: String)]
pub struct RetrieveData<'info> {
    pub session_state: Account<'info, SessionState>,
    #[account(
        seeds = [b"session_data", session_state.key().as_ref(), key.as_bytes()],
        bump
    )]
    pub session_data: Account<'info, SessionData>,
}

/// Update session metadata
#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(mut)]
    pub session_state: Account<'info, SessionState>,
    pub owner: Signer<'info>,
}

/// Close session
#[derive(Accounts)]
pub struct CloseSession<'info> {
    #[account(mut, close = owner)]
    pub session_state: Account<'info, SessionState>,
    pub owner: Signer<'info>,
}

// Handler functions
pub fn initialize(
    ctx: Context<Initialize>,
    owner: Pubkey,
    eval_program: Pubkey,
    session_id: String,
    namespaces: Vec<String>,
) -> Result<()> {
    let session_state = &mut ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Validate inputs
    require!(
        owner != Pubkey::default(),
        ValenceSessionError::SessionInvalidOwner
    );
    require!(
        eval_program != Pubkey::default(),
        ValenceSessionError::SessionInvalidEvalProgram
    );
    require!(
        !session_id.is_empty() && session_id.len() <= 64,
        ValenceSessionError::SessionIdTooLong
    );
    
    // Initialize session state
    session_state.owner = owner;
    session_state.eval_program = eval_program;
    session_state.session_id = session_id.clone();
    session_state.namespaces = namespaces;
    session_state.is_active = true;
    session_state.metadata = SessionMetadata::default();
    session_state.total_executions = 0;
    session_state.created_at = clock.unix_timestamp;
    session_state.last_activity = clock.unix_timestamp;
    session_state.bump = ctx.bumps.session_state;
    
    emit!(SessionInitializedEvent {
        session_id,
        owner,
        eval_program,
        namespace_count: session_state.namespaces.len() as u8,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn execute_call(
    ctx: Context<ExecuteCall>,
    target_program: Pubkey,
    _call_data: Vec<u8>,
) -> Result<()> {
    let session_state = &mut ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only eval can execute calls
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Update session metrics
    session_state.total_executions = session_state
        .total_executions
        .checked_add(1)
        .unwrap_or(u64::MAX);
    session_state.last_activity = clock.unix_timestamp;
    
    emit!(CallExecutedEvent {
        session_id: session_state.session_id.clone(),
        target_program,
        caller: ctx.accounts.caller.key(),
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn create_token_account(
    ctx: Context<CreateTokenAccount>,
    mint: Pubkey,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only eval can create token accounts
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    emit!(TokenAccountCreatedEvent {
        session_id: session_state.session_id.clone(),
        mint,
        token_account: ctx.accounts.token_account.key(),
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn transfer_token(
    ctx: Context<TransferToken>,
    amount: u64,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only eval can transfer tokens from the session
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Prepare seeds for signing
    let session_seeds = &[
        b"session",
        session_state.owner.as_ref(),
        session_state.session_id.as_bytes(),
        &[session_state.bump],
    ];
    let signer_seeds = &[&session_seeds[..]];
    
    // Transfer tokens
    let cpi_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.to_token_account.to_account_info(),
        authority: ctx.accounts.session_state.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    
    token::transfer(cpi_ctx, amount)?;
    
    emit!(TokenTransferredEvent {
        session_id: session_state.session_id.clone(),
        from: ctx.accounts.from_token_account.key(),
        to: ctx.accounts.to_token_account.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn transfer_sol(
    ctx: Context<TransferSol>,
    amount: u64,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only eval can transfer SOL from the session
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Transfer SOL
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.session_state.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
            },
        ),
        amount,
    )?;
    
    emit!(SolTransferredEvent {
        session_id: session_state.session_id.clone(),
        to: ctx.accounts.to.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn approve_token(
    ctx: Context<ApproveToken>,
    amount: u64,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only eval can approve token spending
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Prepare seeds for signing
    let session_seeds = &[
        b"session",
        session_state.owner.as_ref(),
        session_state.session_id.as_bytes(),
        &[session_state.bump],
    ];
    let signer_seeds = &[&session_seeds[..]];
    
    // Approve tokens
    let cpi_accounts = Approve {
        to: ctx.accounts.token_account.to_account_info(),
        delegate: ctx.accounts.delegate.to_account_info(),
        authority: ctx.accounts.session_state.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    
    token::approve(cpi_ctx, amount)?;
    
    emit!(TokenApprovedEvent {
        session_id: session_state.session_id.clone(),
        token_account: ctx.accounts.token_account.key(),
        delegate: ctx.accounts.delegate.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn store_data(
    ctx: Context<StoreData>,
    key: String,
    value: Vec<u8>,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let session_data = &mut ctx.accounts.session_data;
    let clock = Clock::get()?;
    
    // Only eval can store data
    require!(
        ctx.accounts.caller.key() == session_state.eval_program,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Validate inputs
    require!(
        !key.is_empty() && key.len() <= 64,
        ValenceSessionError::SessionDataKeyTooLong
    );
    require!(
        value.len() <= 1024,
        ValenceSessionError::SessionDataValueTooLarge
    );
    
    // Store data
    session_data.session = ctx.accounts.session_state.key();
    session_data.key = key.clone();
    session_data.value = value;
    session_data.created_at = clock.unix_timestamp;
    session_data.last_updated = clock.unix_timestamp;
    session_data.bump = ctx.bumps.session_data;
    
    emit!(DataStoredEvent {
        session_id: session_state.session_id.clone(),
        key,
        value_size: session_data.value.len() as u32,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn retrieve_data(
    ctx: Context<RetrieveData>,
    key: String,
) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let session_data = &ctx.accounts.session_data;
    
    // Verify session is active
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Verify data belongs to this session
    require!(
        session_data.session == ctx.accounts.session_state.key(),
        ValenceSessionError::SessionDataNotFound
    );
    
    // Verify key matches
    require!(
        session_data.key == key,
        ValenceSessionError::SessionDataNotFound
    );
    
    emit!(DataRetrievedEvent {
        session_id: session_state.session_id.clone(),
        key,
        value_size: session_data.value.len() as u32,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

pub fn update_metadata(
    ctx: Context<UpdateMetadata>,
    new_metadata: SessionMetadata,
) -> Result<()> {
    let session_state = &mut ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only owner can update metadata
    require!(
        ctx.accounts.owner.key() == session_state.owner,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    require!(
        session_state.is_active,
        ValenceSessionError::SessionNotActive
    );
    
    // Update metadata
    session_state.metadata = new_metadata;
    session_state.last_activity = clock.unix_timestamp;
    
    emit!(MetadataUpdatedEvent {
        session_id: session_state.session_id.clone(),
        updater: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn close_session(ctx: Context<CloseSession>) -> Result<()> {
    let session_state = &ctx.accounts.session_state;
    let clock = Clock::get()?;
    
    // Only owner can close session
    require!(
        ctx.accounts.owner.key() == session_state.owner,
        ValenceSessionError::SessionUnauthorizedCaller
    );
    
    emit!(SessionClosedEvent {
        session_id: session_state.session_id.clone(),
        owner: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

// Event definitions
#[event]
pub struct SessionInitializedEvent {
    pub session_id: String,
    pub owner: Pubkey,
    pub eval_program: Pubkey,
    pub namespace_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct CallExecutedEvent {
    pub session_id: String,
    pub target_program: Pubkey,
    pub caller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenAccountCreatedEvent {
    pub session_id: String,
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenTransferredEvent {
    pub session_id: String,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct SolTransferredEvent {
    pub session_id: String,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokenApprovedEvent {
    pub session_id: String,
    pub token_account: Pubkey,
    pub delegate: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DataStoredEvent {
    pub session_id: String,
    pub key: String,
    pub value_size: u32,
    pub timestamp: i64,
}

#[event]
pub struct DataRetrievedEvent {
    pub session_id: String,
    pub key: String,
    pub value_size: u32,
    pub timestamp: i64,
}

#[event]
pub struct MetadataUpdatedEvent {
    pub session_id: String,
    pub updater: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SessionClosedEvent {
    pub session_id: String,
    pub owner: Pubkey,
    pub timestamp: i64,
}