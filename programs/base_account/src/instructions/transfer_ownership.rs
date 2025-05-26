use anchor_lang::prelude::*;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<TransferOwnership>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let new_owner = ctx.accounts.new_owner.key();
    
    // Only the current owner can transfer ownership
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Update the owner
    account_state.owner = new_owner;
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Transferred ownership to: {}", new_owner);
    Ok(())
}




#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// CHECK: This is the new owner address
    pub new_owner: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
} 