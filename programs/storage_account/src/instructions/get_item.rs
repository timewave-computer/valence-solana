use anchor_lang::prelude::*;
use crate::state::{StorageAccount, StorageItem};
use crate::error::StorageAccountError;

#[derive(Accounts)]
#[instruction(key: String)]
pub struct GetItem<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    #[account(
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

pub fn handler(ctx: Context<GetItem>, key: String) -> Result<()> {
    let storage_item = &ctx.accounts.storage_item;
    
    // This is primarily a read operation, but we log the information for client reference
    msg!(
        "Retrieved storage item - key: {}, type: {:?}, version: {}, size: {} bytes, created: {}, updated: {}", 
        storage_item.key,
        storage_item.value_type,
        storage_item.version,
        storage_item.value.len(),
        storage_item.created_at,
        storage_item.updated_at
    );
    
    // No need to modify any state, this is a pure read operation
    Ok(())
} 