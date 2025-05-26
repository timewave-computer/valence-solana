use anchor_lang::prelude::*;
// use std::collections::HashMap; // Unused for now
use crate::error::StorageAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum ValueType {
    String,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Bool,
    Pubkey,
    Bytes,
}

#[account]
pub struct StorageAccount {
    /// The authority (owner) of the storage account
    pub authority: Pubkey,
    /// The authorization token used to validate operations
    pub auth_token: Pubkey,
    /// Set of approved library addresses that can be used with this account
    pub approved_libraries: Vec<Pubkey>,
    /// Number of token accounts managed by this storage account
    pub token_account_count: u32,
    /// Total number of instructions executed
    pub instruction_count: u64,
    /// Timestamp of the last activity
    pub last_activity: i64,
    /// The storage authority PDA for managing items
    pub storage_authority: Pubkey,
    /// Number of items stored in this account
    pub item_count: u32,
    /// Maximum storage capacity in bytes
    pub max_capacity: u32,
    /// Current storage usage in bytes
    pub current_usage: u32,
}

#[account]
pub struct StorageItem {
    /// The storage account this item belongs to
    pub storage_account: Pubkey,
    /// The item key (max 32 chars)
    pub key: String,
    /// The value type
    pub value_type: ValueType,
    /// The actual value data (serialized)
    pub value: Vec<u8>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Version of the item (incremented on updates)
    pub version: u32,
}

impl StorageAccount {
    /// Calculate space needed for this account
    pub fn space(max_libraries: usize) -> usize {
        8 + // discriminator
        32 + // authority
        32 + // auth_token
        4 + (max_libraries * 32) + // approved_libraries vec
        4 + // token_account_count
        8 + // instruction_count
        8 + // last_activity
        32 + // storage_authority
        4 + // item_count
        4 + // max_capacity
        4 // current_usage
    }
    
    pub fn is_library_approved(&self, library: &Pubkey) -> bool {
        self.approved_libraries.contains(library)
    }
    
    pub fn approve_library(&mut self, library: Pubkey) -> Result<()> {
        if !self.approved_libraries.contains(&library) {
            self.approved_libraries.push(library);
        }
        Ok(())
    }
    
    pub fn record_instruction_execution(&mut self) {
        self.instruction_count = self.instruction_count.saturating_add(1);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_token_account_count(&mut self) {
        self.token_account_count = self.token_account_count.saturating_add(1);
    }
    
    pub fn increment_item_count(&mut self) {
        self.item_count = self.item_count.saturating_add(1);
    }
    
    pub fn decrement_item_count(&mut self) {
        self.item_count = self.item_count.saturating_sub(1);
    }
    
    pub fn increase_usage(&mut self, bytes: u32) -> Result<()> {
        self.current_usage = self.current_usage.saturating_add(bytes);
        Ok(())
    }
    
    pub fn decrease_usage(&mut self, bytes: u32) {
        self.current_usage = self.current_usage.saturating_sub(bytes);
    }
    
    pub fn has_capacity_for(&self, bytes: u32) -> bool {
        self.current_usage.saturating_add(bytes) <= self.max_capacity
    }
}

impl StorageItem {
    /// Calculate space needed for this item
    pub fn space(key_length: usize, value_length: usize) -> usize {
        8 + // discriminator
        32 + // storage_account
        4 + key_length + // key (string with length prefix)
        1 + // value_type
        4 + value_length + // value (vec with length prefix)
        8 + // created_at
        8 + // updated_at
        4 // version
    }
    
    pub fn get_size(&self) -> usize {
        Self::space(self.key.len(), self.value.len())
    }
    
    pub fn deserialize_value<T: anchor_lang::AnchorDeserialize>(&self) -> Result<T> {
        T::try_from_slice(&self.value)
            .map_err(|_| error!(StorageAccountError::ValueTypeMismatch))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StorageItemUpdate {
    pub key: String,
    pub value_type: ValueType,
    pub value: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StorageItemDelete {
    pub key: String,
} 