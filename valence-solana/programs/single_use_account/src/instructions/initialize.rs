use anchor_lang::prelude::*;
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub auth_token: Pubkey,
    pub required_destination: Option<Pubkey>,
    pub expiration_time: Option<i64>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = SingleUseAccount::SIZE,
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    let authority = ctx.accounts.authority.key();
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate expiration time if provided
    if let Some(expiration_time) = params.expiration_time {
        if expiration_time <= current_time {
            return Err(SingleUseAccountError::ExpirationInPast.into());
        }
    }
    
    // Initialize single-use account
    single_use_account.authority = authority;
    single_use_account.auth_token = params.auth_token;
    single_use_account.approved_libraries = Vec::new();
    single_use_account.token_account_count = 0;
    single_use_account.instruction_count = 0;
    single_use_account.last_activity = current_time;
    single_use_account.was_used = false;
    single_use_account.required_destination = params.required_destination;
    single_use_account.expiration_time = params.expiration_time;
    single_use_account.reserved = [0; 64];
    
    msg!("Single-use account initialized for authority: {}", single_use_account.authority);
    
    if let Some(destination) = single_use_account.required_destination {
        msg!("Required destination: {}", destination);
    }
    
    if let Some(expiration) = single_use_account.expiration_time {
        msg!("Expiration time: {} (current time: {})", expiration, current_time);
    }
    
    Ok(())
} 