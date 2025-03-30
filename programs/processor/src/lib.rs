use anchor_lang::prelude::*;

declare_id!("ProcE5soriVLiEHDXAWzZNaZrGwEWzUmT5Waa4Cyr7A");

pub mod state;
pub mod instructions;
pub mod error;
pub mod queue;

#[program]
pub mod processor {
    use super::*;
    
    pub fn initialize(_ctx: Context<Empty>, _auth_id: Pubkey) -> Result<()> {
        msg!("Processor initialization placeholder");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Empty<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}
