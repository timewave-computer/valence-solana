use anchor_lang::prelude::*;
use anchor_spl::token::{Token, CloseAccount};
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<CloseTokenAccount>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let token_account = ctx.accounts.token_account.key();
    
    // Only the owner can close token accounts
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if token account exists in state
    if !account_state.token_accounts.contains(&token_account) {
        return Err(BaseAccountError::TokenAccountNotFound.into());
    }
    
    // Remove token account from state
    account_state.token_accounts.retain(|&x| x != token_account);
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Closed token account: {}", token_account);
    Ok(())
}

#[derive(Accounts)]
pub struct CloseTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(
        mut,
        constraint = token_account.owner == account.vault_authority @ BaseAccountError::InvalidVaultAuthority
    )]
    pub token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK: This is the destination for the closed token account funds
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
} 