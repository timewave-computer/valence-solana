// Storage account program for managing key-value storage
use anchor_lang::prelude::*;

declare_id!("StrgAcntpYbpLUXMvUb8ZuUjQBnMLFZnBxwxrRjrYJU");

pub mod error;
pub mod state;

use state::StorageAccount;

#[program]
pub mod storage_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, max_capacity: u32) -> Result<()> {
        let storage_account = &mut ctx.accounts.storage_account;
        
        // Initialize the storage account
        storage_account.authority = ctx.accounts.signer.key();
        storage_account.auth_token = ctx.accounts.signer.key(); // For now, use same as authority
        storage_account.approved_libraries = Vec::new();
        storage_account.token_account_count = 0;
        storage_account.instruction_count = 0;
        storage_account.last_activity = Clock::get()?.unix_timestamp;
        storage_account.storage_authority = ctx.accounts.signer.key(); // For now, use same as authority
        storage_account.item_count = 0;
        storage_account.max_capacity = max_capacity;
        storage_account.current_usage = 0;
        
        msg!("Storage account initialized with authority: {}", storage_account.authority);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = StorageAccount::space(10), // Allow up to 10 approved libraries
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 