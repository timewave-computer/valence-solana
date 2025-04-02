use anchor_lang::prelude::*;
use crate::state::{StorageAccount, StorageItem, ValueType};
use crate::error::StorageAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SetItemParams {
    pub key: String,
    pub value_type: ValueType,
    pub value: Vec<u8>,
}

#[derive(Accounts)]
#[instruction(params: SetItemParams)]
pub struct SetItem<'info> {
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
        init_if_needed,
        payer = authority,
        space = StorageItem::get_space_requirement(params.key.len(), params.value.len()),
        seeds = [
            b"storage_item",
            storage_account.key().as_ref(),
            params.key.as_bytes()
        ],
        bump
    )]
    pub storage_item: Account<'info, StorageItem>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> SetItem<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, SetItem<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation beyond the account constraints can be added here
        // For example, checking key length or value size limits
        let params = ctx.remaining_accounts.get(0)
            .ok_or(StorageAccountError::MissingRequiredParameters)?;
        
        // Validate that the key length is reasonable
        let key_length = params.data.borrow_mut()[0] as usize;
        if key_length == 0 || key_length > 100 {
            return Err(StorageAccountError::InvalidKeyLength.into());
        }
        
        Ok(())
    }
}

pub fn handler(ctx: Context<SetItem>, params: SetItemParams) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    let storage_item = &mut ctx.accounts.storage_item;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Check if the account has enough capacity
    let item_size = params.value.len() as u32;
    let is_update = storage_item.key == params.key && !storage_item.key.is_empty();
    
    // For updates, calculate the net change in storage usage
    if is_update {
        let old_size = storage_item.value.len() as u32;
        if item_size > old_size {
            let additional_bytes = item_size - old_size;
            if !storage_account.has_capacity_for(additional_bytes) {
                return Err(StorageAccountError::StorageCapacityExceeded.into());
            }
            storage_account.increase_usage(additional_bytes)?;
        } else if old_size > item_size {
            let reduced_bytes = old_size - item_size;
            storage_account.decrease_usage(reduced_bytes);
        }
    } else {
        // For new items, check if we have enough capacity
        if !storage_account.has_capacity_for(item_size) {
            return Err(StorageAccountError::StorageCapacityExceeded.into());
        }
        
        // Update storage account state for new items
        storage_account.increment_item_count();
        storage_account.increase_usage(item_size)?;
    }
    
    // Update the storage item
    if !is_update {
        // Initialize a new item
        storage_item.storage_account = storage_account.key();
        storage_item.key = params.key.clone();
        storage_item.created_at = current_time;
        storage_item.version = 1;
    } else {
        // Update existing item
        storage_item.version = storage_item.version.saturating_add(1);
    }
    
    storage_item.value_type = params.value_type;
    storage_item.value = params.value;
    storage_item.updated_at = current_time;
    
    // Update the last activity timestamp
    storage_account.last_activity = current_time;
    
    msg!(
        "Set storage item with key: {}, version: {}, size: {} bytes", 
        storage_item.key,
        storage_item.version,
        storage_item.value.len()
    );
    
    Ok(())
} 