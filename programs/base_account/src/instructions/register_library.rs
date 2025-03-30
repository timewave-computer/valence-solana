use anchor_lang::prelude::*;
use crate::state::BaseAccount;
use crate::error::BaseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterLibraryParams {
    pub library: Pubkey,
    pub auto_approve: bool,
}

#[derive(Accounts)]
pub struct RegisterLibrary<'info> {
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

pub fn handler(ctx: Context<RegisterLibrary>, params: RegisterLibraryParams) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Validate the authority is the owner of the base account
    if base_account.authority != ctx.accounts.authority.key() {
        return Err(BaseAccountError::UnauthorizedOwnerOperation.into());
    }
    
    // Register and optionally approve the library
    if params.auto_approve {
        base_account.approve_library(params.library)?;
        msg!("Library registered and approved: {}", params.library);
    } else {
        msg!("Library registered (pending approval): {}", params.library);
    }
    
    // Update the last activity timestamp
    base_account.last_activity = Clock::get()?.unix_timestamp;
    
    Ok(())
} 