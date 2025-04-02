use anchor_lang::prelude::*;
use crate::state::StorageAccount;
use crate::error::StorageAccountError;

#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
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

impl<'info> ApproveLibrary<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, ApproveLibrary<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


pub fn handler(ctx: Context<ApproveLibrary>, library: Pubkey) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    
    // Validate the authority is the owner of the storage account
    if storage_account.authority != ctx.accounts.authority.key() {
        return Err(StorageAccountError::UnauthorizedOwnerOperation.into());
    }
    
    // Approve the library
    storage_account.approve_library(library)?;
    
    // Update the last activity timestamp
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Library approved for use with storage account: {}", library);
    
    Ok(())
} 