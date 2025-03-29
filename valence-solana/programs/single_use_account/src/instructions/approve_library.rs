use anchor_lang::prelude::*;
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
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
}

pub fn handler(ctx: Context<ApproveLibrary>, library: Pubkey) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Validate the account hasn't been used
    if single_use_account.was_used {
        return Err(SingleUseAccountError::AccountAlreadyUsed.into());
    }
    
    // Approve the library
    single_use_account.approve_library(library)?;
    
    // Update the last activity timestamp
    single_use_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Library approved for use with single-use account: {}", library);
    
    Ok(())
} 