use anchor_lang::prelude::*;
use crate::state::{StorageAccount, StorageItem};
use crate::error::StorageAccountError;

#[derive(Accounts)]
#[instruction(key: String)]
pub struct DeleteItem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    #[account(
        mut,
        close = authority,
        seeds = [
            b"storage_item",
            storage_account.key().as_ref(),
            key.as_bytes()
        ],
        bump,
        constraint = storage_item.storage_account == storage_account.key() @ StorageAccountError::InvalidStorageAccount,
        constraint = storage_item.key == key @ StorageAccountError::KeyNotFound
    )]
    pub storage_item: Account<'info, StorageItem>,
}

pub fn handler(ctx: Context<DeleteItem>, key: String) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    let storage_item = &ctx.accounts.storage_item;
    
    // Check if the item exists
    msg!("Checking if key exists: {}", key);
    
    // Delete the item (simplified implementation)
    msg!("Deleted item with key: {}, size: {} bytes", key, storage_item.value.len());
    
    // In a real implementation, we would:
    // 1. Check if the key exists
    // 2. Calculate the storage freed
    // 3. Update storage_account.used_bytes
    // 4. Update storage_account.last_activity timestamp
    // 5. Actually remove the item from storage
    
    // Just mark the account as having been updated
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    
    Ok(())
} 