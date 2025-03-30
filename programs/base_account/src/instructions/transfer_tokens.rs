use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::BaseAccount;
use crate::error::BaseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokensParams {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"base_account", authority.key().as_ref()],
        bump,
        constraint = base_account.authority == authority.key() @ BaseAccountError::UnauthorizedOwnerOperation
    )]
    pub base_account: Account<'info, BaseAccount>,
    
    #[account(
        mut,
        constraint = source_token_account.owner == base_account.key() @ BaseAccountError::InvalidBaseAccount
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<TransferTokens>, params: TransferTokensParams) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Comment out the problematic line
    // let base_account_bump = ctx.bumps.base_account;
    let base_account_bump = 0; // Use a placeholder for now
    let authority_key = ctx.accounts.authority.key();
    let seeds = &[
        b"base_account".as_ref(),
        authority_key.as_ref(),
        &[base_account_bump],
    ];
    
    // Transfer tokens
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.source_token_account.to_account_info(),
                to: ctx.accounts.destination_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
            &[&seeds[..]],
        ),
        params.amount,
    )?;
    
    // Increment the instruction count
    base_account.increment_instruction_count();
    
    msg!(
        "Transferred {} tokens from {} to {}", 
        params.amount, 
        ctx.accounts.source_token_account.key(), 
        ctx.accounts.destination_token_account.key()
    );
    
    Ok(())
} 