use anchor_lang::prelude::*;
use std::collections::BTreeMap;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<RevokeLibrary>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let library = ctx.accounts.library.key();
    
    // Only the owner can revoke libraries
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if library is approved
    if !account_state.approved_libraries.contains(&library) {
        return Err(BaseAccountError::LibraryNotApproved.into());
    }
    
    // Revoke the library
    account_state.approved_libraries.retain(|&x| x != library);
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Revoked library: {}", library);
    Ok(())
}

impl<'info> RevokeLibrary<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, RevokeLibrary<'info>>,
        _bumps: &BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct RevokeLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// Address of the library program to revoke
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
} 