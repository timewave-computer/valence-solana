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
    
    // Update the storage account to reflect deleted storage
    storage_account.decrement_item_count();
    storage_account.decrease_usage(storage_item.value.len() as u32);
    
    // Update the last activity timestamp
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!(
        "Deleted storage item - key: {}, version: {}, freed: {} bytes", 
        storage_item.key,
        storage_item.version,
        storage_item.value.len()
    );
    
    // The account will be automatically closed and lamports returned due to the close = authority constraint
    Ok(())
} 