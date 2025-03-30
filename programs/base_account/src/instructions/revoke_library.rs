use anchor_lang::prelude::*;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<RevokeLibrary>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let library = ctx.accounts.library.key();
    
    // Only the owner can revoke libraries
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if library is already approved
    if !account_state.is_library_approved(&library) {
        return Err(BaseAccountError::LibraryNotApproved.into());
    }
    
    // Remove library from approved list
    account_state.remove_approved_library(&library)?;
    
    msg!("Revoked library approval: {}", library);
    Ok(())
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