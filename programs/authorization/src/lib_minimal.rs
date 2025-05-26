// Minimal authorization program to test compilation
use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

#[program]
pub mod authorization {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing authorization program");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
} 