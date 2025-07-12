//! Admin operations (controller)

use anchor_lang::prelude::*;
use crate::{ShardConfig, ShardError};

/// Initialize shard configuration
pub fn initialize(
    ctx: Context<Initialize>,
    max_operations_per_bundle: u16,
    default_respect_deregistration: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.shard_config;
    
    config.authority = ctx.accounts.authority.key();
    config.is_paused = false;
    config.max_operations_per_bundle = max_operations_per_bundle;
    config.default_respect_deregistration = default_respect_deregistration;
    config._reserved = [0u8; 31];
    
    msg!("Shard initialized with max_ops: {}, respect_dereg: {}", 
        max_operations_per_bundle, default_respect_deregistration);
    Ok(())
}

/// Update shard configuration
pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_config: ShardConfig,
) -> Result<()> {
    let config = &mut ctx.accounts.shard_config;
    
    // Verify authority
    require!(
        config.authority == ctx.accounts.authority.key(),
        ShardError::Unauthorized
    );
    
    // Update config
    config.is_paused = new_config.is_paused;
    config.max_operations_per_bundle = new_config.max_operations_per_bundle;
    config.default_respect_deregistration = new_config.default_respect_deregistration;
    
    msg!("Shard config updated");
    Ok(())
}

/// Pause or unpause the shard
pub fn set_paused(
    ctx: Context<SetPaused>,
    paused: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.shard_config;
    
    // Verify authority
    require!(
        config.authority == ctx.accounts.authority.key(),
        ShardError::Unauthorized
    );
    
    config.is_paused = paused;
    
    msg!("Shard {} paused", if paused { "is now" } else { "is no longer" });
    Ok(())
}

// Account contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 2 + 1 + 31, // discriminator + authority + paused + max_ops + respect_dereg + reserved
        seeds = [b"shard_config", authority.key().as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = shard_config.authority == authority.key() @ ShardError::Unauthorized,
    )]
    pub shard_config: Account<'info, ShardConfig>,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = shard_config.authority == authority.key() @ ShardError::Unauthorized,
    )]
    pub shard_config: Account<'info, ShardConfig>,
}