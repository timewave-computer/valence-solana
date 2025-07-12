//! Function import management

use anchor_lang::prelude::*;
use crate::{FunctionImport, ShardConfig, ShardError};

/// Import a function from the registry
pub fn import_function(
    ctx: Context<ImportFunction>,
    function_hash: [u8; 32],
    respect_deregistration: Option<bool>,
) -> Result<()> {
    let shard_config = &ctx.accounts.shard_config;
    let function_import = &mut ctx.accounts.function_import;
    
    // Use provided policy or default from shard config
    let respect_deregistration = respect_deregistration
        .unwrap_or(shard_config.default_respect_deregistration);
    
    // Verify the function exists in registry by checking the account data
    let registry_entry = &ctx.accounts.registry_entry;
    require!(
        !registry_entry.data_is_empty(),
        ShardError::FunctionNotFound
    );
    
    // Parse the registry entry data to get the program
    let data = registry_entry.try_borrow_data()?;
    require!(
        data.len() >= 8 + 32 + 32 + 32, // discriminator + hash + program + authority
        ShardError::FunctionNotFound
    );
    
    // Skip discriminator (8 bytes) and hash (32 bytes) to get to program
    let program_bytes = &data[40..72];
    let program = Pubkey::new_from_array(program_bytes.try_into().unwrap());
    
    // Initialize function import
    function_import.shard = ctx.accounts.shard_config.key();
    function_import.function_hash = function_hash;
    function_import.program = program;
    function_import.respect_deregistration = respect_deregistration;
    function_import.imported_at = Clock::get()?.unix_timestamp;
    
    msg!("Imported function with hash {:?}, respect_deregistration: {}", 
        function_hash, respect_deregistration);
    
    Ok(())
}

/// Update function import policy
pub fn update_import_policy(
    ctx: Context<UpdateImportPolicy>,
    function_hash: [u8; 32],
    respect_deregistration: bool,
) -> Result<()> {
    let function_import = &mut ctx.accounts.function_import;
    
    // Verify function hash matches
    require!(
        function_import.function_hash == function_hash,
        ShardError::FunctionMismatch
    );
    
    // Verify authority
    require!(
        ctx.accounts.authority.key() == ctx.accounts.shard_config.authority,
        ShardError::Unauthorized
    );
    
    // Update policy
    function_import.respect_deregistration = respect_deregistration;
    
    msg!("Updated import policy for function {:?}, respect_deregistration: {}", 
        function_hash, respect_deregistration);
    
    Ok(())
}

// Account contexts

#[derive(Accounts)]
#[instruction(function_hash: [u8; 32])]
pub struct ImportFunction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Shard configuration
    #[account(
        seeds = [b"shard_config", authority.key().as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
    
    /// Function import record
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 1 + 8, // discriminator + shard + hash + program + bool + timestamp
        seeds = [b"function_import", shard_config.key().as_ref(), &function_hash],
        bump,
    )]
    pub function_import: Account<'info, FunctionImport>,
    
    /// Registry entry for the function
    /// CHECK: Validated by reading from registry program
    #[account(
        seeds = [b"function", function_hash.as_ref()],
        seeds::program = registry_program.key(),
        bump,
    )]
    pub registry_entry: UncheckedAccount<'info>,
    
    /// Registry program
    /// CHECK: Known program ID
    pub registry_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(function_hash: [u8; 32])]
pub struct UpdateImportPolicy<'info> {
    pub authority: Signer<'info>,
    
    /// Shard configuration
    #[account(
        seeds = [b"shard_config", shard_config.authority.as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
    
    /// Function import record
    #[account(
        mut,
        seeds = [b"function_import", shard_config.key().as_ref(), &function_hash],
        bump,
    )]
    pub function_import: Account<'info, FunctionImport>,
}

// We'll read the registry entry directly without deserializing