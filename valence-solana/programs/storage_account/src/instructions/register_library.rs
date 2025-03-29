use anchor_lang::prelude::*;
use crate::state::StorageAccount;
use crate::error::StorageAccountError;

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
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
}

pub fn handler(ctx: Context<RegisterLibrary>, params: RegisterLibraryParams) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    
    // Validate the authority is the owner of the storage account
    if storage_account.authority != ctx.accounts.authority.key() {
        return Err(StorageAccountError::UnauthorizedOwnerOperation.into());
    }
    
    // Register and optionally approve the library
    if params.auto_approve {
        storage_account.approve_library(params.library)?;
        msg!("Library registered and approved: {}", params.library);
    } else {
        msg!("Library registered (pending approval): {}", params.library);
    }
    
    // Update the last activity timestamp
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    
    Ok(())
} 