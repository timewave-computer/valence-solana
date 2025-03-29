use anchor_lang::prelude::*;
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

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
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump,
        constraint = single_use_account.authority == authority.key() @ SingleUseAccountError::UnauthorizedOwnerOperation,
        constraint = !single_use_account.was_used @ SingleUseAccountError::AccountAlreadyUsed
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,
}

pub fn handler(ctx: Context<RegisterLibrary>, params: RegisterLibraryParams) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Validate the account hasn't been used
    if single_use_account.was_used {
        return Err(SingleUseAccountError::AccountAlreadyUsed.into());
    }
    
    // Register and optionally approve the library
    if params.auto_approve {
        single_use_account.approve_library(params.library)?;
        msg!("Library registered and approved: {}", params.library);
    } else {
        msg!("Library registered (pending approval): {}", params.library);
    }
    
    // Update the last activity timestamp
    single_use_account.last_activity = Clock::get()?.unix_timestamp;
    
    Ok(())
} 