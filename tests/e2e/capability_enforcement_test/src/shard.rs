use anchor_lang::prelude::*;
use crate::functions::echo::process_echo;
use crate::functions::transfer::process_transfer;

/// Main shard processor
pub struct ShardProcessor;

impl ShardProcessor {
    /// Process initialize instruction
    pub fn process_initialize(ctx: &Context<crate::Initialize>) -> Result<()> {
        msg!("Shard initialized by: {}", ctx.accounts.payer.key());
        Ok(())
    }

    /// Process echo instruction
    pub fn process_echo(ctx: &Context<crate::Echo>, message: &str) -> Result<()> {
        msg!("Echo instruction from: {}", ctx.accounts.payer.key());
        process_echo(message)
    }
    
    /// Process transfer instruction - requires TRANSFER capability
    pub fn process_transfer(ctx: &Context<crate::Transfer>, amount: u64, recipient: Pubkey) -> Result<()> {
        msg!("Transfer instruction from: {}", ctx.accounts.payer.key());
        process_transfer(amount, recipient)
    }
}