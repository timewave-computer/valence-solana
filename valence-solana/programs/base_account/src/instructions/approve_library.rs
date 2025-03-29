use anchor_lang::prelude::*;
use crate::state::BaseAccount;
use crate::error::BaseAccountError;

#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"base_account", authority.key().as_ref()],
        bump,
        constraint = base_account.authority == authority.key() @ BaseAccountError::UnauthorizedOwnerOperation
    )]
    pub base_account: Account<'info, BaseAccount>,
}

pub fn handler(ctx: Context<ApproveLibrary>, library: Pubkey) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Validate the authority is the owner of the base account
    if base_account.authority != ctx.accounts.authority.key() {
        return Err(BaseAccountError::UnauthorizedOwnerOperation.into());
    }
    
    // Approve the library
    base_account.approve_library(library)?;
    
    // Update the last activity timestamp
    base_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Library approved for use with base account: {}", library);
    
    Ok(())
} 