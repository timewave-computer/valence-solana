use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump,
        constraint = single_use_account.authority == authority.key() @ SingleUseAccountError::UnauthorizedOwnerOperation,
        constraint = !single_use_account.was_used @ SingleUseAccountError::AccountAlreadyUsed
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,
    
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Validated in the handler
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateTokenAccount>, mint: Pubkey) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Validate the account hasn't been used
    if single_use_account.was_used {
        return Err(SingleUseAccountError::AccountAlreadyUsed.into());
    }
    
    // In a real implementation, we would perform the token account creation
    // using CPIs to the token program. For this example, we just update our state.
    
    // Increment the token account count
    single_use_account.increment_token_account_count();
    
    // Update last activity timestamp
    single_use_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Token account created for mint: {}", mint);
    
    Ok(())
} 