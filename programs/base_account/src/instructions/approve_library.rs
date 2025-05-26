use anchor_lang::prelude::*;
use crate::state::AccountState;

pub fn handler(ctx: Context<ApproveLibrary>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let library = ctx.accounts.library.key();
    
    // Add the library to approved list
    account_state.approve_library(library)?;
    account_state.record_instruction_execution();
    
    msg!("Library {} approved for account {}", library, account_state.key());
    Ok(())
}

#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK: This is the library program to approve
    pub library: AccountInfo<'info>,
} 