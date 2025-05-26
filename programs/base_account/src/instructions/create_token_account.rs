use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::AccountState;

pub fn handler(ctx: Context<CreateTokenAccount>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let token_account = ctx.accounts.token_account.key();
    
    // Add token account to tracking
    account_state.add_token_account(token_account)?;
    account_state.record_instruction_execution();
    
    msg!("Token account {} created for account {}", token_account, account_state.key());
    Ok(())
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// The token account to track
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
} 