use anchor_lang::prelude::*;

// Module declarations
pub mod functions;
pub mod shard;
pub mod instruction;

// Import the shard processor
use shard::ShardProcessor;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod capability_enforcement_test {
    use super::*;

    /// Initialize instruction
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ShardProcessor::process_initialize(&ctx)
    }

    /// Echo instruction
    pub fn echo(ctx: Context<Echo>, message: String) -> Result<()> {
        ShardProcessor::process_echo(&ctx, &message)
    }
    
    /// Transfer instruction - requires TRANSFER capability
    pub fn transfer(ctx: Context<Transfer>, amount: u64, recipient: Pubkey) -> Result<()> {
        ShardProcessor::process_transfer(&ctx, amount, recipient)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Echo<'info> {
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub payer: Signer<'info>,
}