use anchor_lang::prelude::*;
use crate::state::StorageAccount;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub auth_token: Pubkey,
    pub max_capacity: u32,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = StorageAccount::SIZE,
        seeds = [b"storage_account", authority.key().as_ref()],
        bump
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    let authority = ctx.accounts.authority.key();
    
    // Derive storage authority PDA
    let (storage_authority, _) = Pubkey::find_program_address(
        &[b"storage_authority", storage_account.key().as_ref()],
        ctx.program_id
    );
    
    // Initialize storage account
    storage_account.authority = authority;
    storage_account.auth_token = params.auth_token;
    storage_account.approved_libraries = Vec::new();
    storage_account.token_account_count = 0;
    storage_account.instruction_count = 0;
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    storage_account.storage_authority = storage_authority;
    storage_account.item_count = 0;
    storage_account.max_capacity = params.max_capacity;
    storage_account.current_usage = 0;
    storage_account.reserved = [0; 64];
    
    msg!("Storage account initialized for authority: {}", storage_account.authority);
    msg!("Storage capacity: {} bytes", storage_account.max_capacity);
    
    Ok(())
} 