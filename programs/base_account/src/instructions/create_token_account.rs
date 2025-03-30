use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<CreateTokenAccount>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let token_account = ctx.accounts.token_account.key();
    let mint = ctx.accounts.mint.key();
    
    // Only the owner can create token accounts
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if token account already exists in state
    if account_state.token_accounts.contains(&token_account) {
        return Err(BaseAccountError::TokenAccountAlreadyExists.into());
    }
    
    // Add token account to state
    account_state.token_accounts.push(token_account);
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Created token account for mint {}: {}", mint, token_account);
    Ok(())
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(
        seeds = [b"vault", account.key().as_ref()],
        bump = account.vault_bump_seed
    )]
    /// CHECK: This is a PDA used as a token authority
    pub vault_authority: UncheckedAccount<'info>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = vault_authority,
    )]
    pub token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 