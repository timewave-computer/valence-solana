use anchor_lang::prelude::*;
use std::collections::HashSet;
use crate::state::BaseAccount;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub auth_token: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = BaseAccount::SIZE,
        seeds = [b"base_account", authority.key().as_ref()],
        bump
    )]
    pub base_account: Account<'info, BaseAccount>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Initialize base account
    base_account.authority = ctx.accounts.authority.key();
    base_account.auth_token = params.auth_token;
    base_account.approved_libraries = HashSet::new();
    base_account.token_account_count = 0;
    base_account.instruction_count = 0;
    base_account.last_activity = Clock::get()?.unix_timestamp;
    base_account.reserved = [0; 64];
    
    msg!("Base account initialized for authority: {}", base_account.authority);
    
    Ok(())
} 