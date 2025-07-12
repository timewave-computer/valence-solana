//! Account management logic (controller)

use anchor_lang::prelude::*;
use crate::{AccountRequest, ShardError, capabilities, ValenceAccount};

/// Request a new account
pub fn request_account(
    ctx: Context<RequestAccount>,
    capabilities: Vec<String>,
    init_state_hash: [u8; 32],
) -> Result<()> {
    // Validate capabilities
    require!(
        !capabilities.is_empty() && capabilities.len() <= 10,
        ShardError::InvalidSessionRequest
    );
    
    // Normalize and validate each capability
    let mut normalized_capabilities = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    for cap in capabilities {
        let normalized = capabilities::normalize_capability(&cap);
        
        // Optionally validate against standard capabilities
        // For now, allow any capability but normalize them
        require!(
            !normalized.is_empty() && normalized.len() <= 64,
            ShardError::InvalidSessionRequest
        );
        
        // Check for duplicates
        require!(
            seen.insert(normalized.clone()),
            ShardError::InvalidSessionRequest
        );
        
        normalized_capabilities.push(normalized);
    }
    
    // Create account request
    let request_key = ctx.accounts.account_request.key();
    let request = &mut ctx.accounts.account_request;
    request.id = request_key;
    request.owner = ctx.accounts.owner.key();
    request.capabilities = normalized_capabilities;
    request.init_state_hash = init_state_hash;
    request.created_at = Clock::get()?.unix_timestamp;
    
    msg!("Account requested with {} capabilities", request.capabilities.len());
    Ok(())
}

/// Initialize an account (called by off-chain service)
pub fn initialize_account(
    ctx: Context<InitializeAccount>,
    request_id: Pubkey,
    init_state_data: Vec<u8>,
) -> Result<()> {
    let request = &ctx.accounts.account_request;
    
    // Verify request ID matches
    require!(
        request.id == request_id,
        ShardError::InvalidSessionRequest
    );
    
    // Verify state hash matches
    let computed_hash = compute_hash(&init_state_data);
    require!(
        computed_hash == request.init_state_hash,
        ShardError::InvalidStateHash
    );
    
    // Create account
    let account_key = ctx.accounts.account.key();
    let account = &mut ctx.accounts.account;
    account.id = account_key;
    account.owner = request.owner;
    account.capabilities = request.capabilities.clone();
    account.state_hash = computed_hash;
    account.is_active = true;
    account.created_at = Clock::get()?.unix_timestamp;
    
    msg!("Account initialized with {} capabilities", account.capabilities.len());
    
    // Request will be closed by anchor
    Ok(())
}

fn compute_hash(data: &[u8]) -> [u8; 32] {
    // Simple hash implementation - in production use proper hashing
    let mut hash = [0u8; 32];
    if !data.is_empty() {
        hash[0] = data[0];
        hash[31] = data[data.len() - 1];
    }
    hash
}

// Account contexts

#[derive(Accounts)]
pub struct RequestAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 200 + 32 + 8, // discriminator + id + owner + capabilities + hash + timestamp
    )]
    pub account_request: Account<'info, AccountRequest>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(request_id: Pubkey)]
pub struct InitializeAccount<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    
    #[account(
        mut,
        constraint = account_request.id == request_id @ ShardError::InvalidSessionRequest,
        close = initializer,
    )]
    pub account_request: Account<'info, AccountRequest>,
    
    #[account(
        init,
        payer = initializer,
        space = 8 + 32 + 32 + 200 + 32 + 1 + 8, // discriminator + id + owner + capabilities + hash + active + timestamp
    )]
    pub account: Account<'info, ValenceAccount>,
    
    pub system_program: Program<'info, System>,
}