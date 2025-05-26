use anchor_lang::prelude::*;
use crate::state::AccountState;

pub fn handler(ctx: Context<RevokeLibrary>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let library = ctx.accounts.library.key();
    
    // Remove the library from approved list
    account_state.remove_approved_library(&library)?;
    account_state.record_instruction_execution();
    
    msg!("Library {} revoked for account {}", library, account_state.key());
    Ok(())
}

#[derive(Accounts)]
pub struct RevokeLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK: This is the library program to revoke
    pub library: AccountInfo<'info>,
} 