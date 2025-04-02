use anchor_lang::prelude::*;
use std::collections::BTreeMap;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<ApproveLibrary>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let library = ctx.accounts.library.key();
    
    // Only the owner can approve libraries
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if library is already approved
    if account_state.approved_libraries.contains(&library) {
        return Err(BaseAccountError::LibraryAlreadyApproved.into());
    }
    
    // Approve the library
    account_state.approved_libraries.push(library);
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Approved library: {}", library);
    Ok(())
}

impl<'info> ApproveLibrary<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, ApproveLibrary<'info>>,
        _bumps: &BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// Address of the library program to approve
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
} 