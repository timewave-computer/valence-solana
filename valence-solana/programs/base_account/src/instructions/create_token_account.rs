use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::BaseAccount;
use crate::error::BaseAccountError;

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"base_account", authority.key().as_ref()],
        bump,
        constraint = base_account.authority == authority.key() @ BaseAccountError::UnauthorizedOwnerOperation
    )]
    pub base_account: Account<'info, BaseAccount>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = base_account,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateTokenAccount>, mint: Pubkey) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Ensure the mint in the context matches the provided mint
    if ctx.accounts.mint.key() != mint {
        return Err(BaseAccountError::InvalidExecutionContext.into());
    }
    
    // Increment the token account count
    base_account.increment_token_account_count();
    
    // Update last activity timestamp
    base_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Token account created for mint: {}", mint);
    
    Ok(())
} 