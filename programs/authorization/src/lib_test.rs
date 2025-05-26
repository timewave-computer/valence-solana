use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod authorization_test {
    use super::*;

    pub fn test_function(ctx: Context<TestAccounts>) -> Result<()> {
        msg!("Test function called");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TestAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
} 