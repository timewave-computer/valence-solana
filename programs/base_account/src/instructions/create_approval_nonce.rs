use anchor_lang::prelude::*;
use crate::state::{AccountState, ApprovalNonce};
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<CreateApprovalNonce>, expiration: i64) -> Result<()> {
    let account_state = &ctx.accounts.account;
    let approval_nonce = &mut ctx.accounts.approval_nonce;
    let library = ctx.accounts.library.key();
    
    // Only the owner can create approval nonces
    if account_state.owner != ctx.accounts.signer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Check if library is approved
    if !account_state.is_library_approved(&library) {
        return Err(BaseAccountError::LibraryNotApproved.into());
    }
    
    // Initialize the approval nonce
    approval_nonce.library = library;
    approval_nonce.nonce = Clock::get()?.unix_timestamp as u64;  // Use current timestamp as nonce
    approval_nonce.owner = ctx.accounts.signer.key();
    approval_nonce.expiration = expiration;
    approval_nonce.is_used = false;
    approval_nonce.bump = *ctx.bumps.get("approval_nonce").unwrap();
    
    msg!("Created approval nonce for library: {}, expires at: {}", library, expiration);
    Ok(())
}

#[derive(Accounts)]
#[instruction(expiration: i64)]
pub struct CreateApprovalNonce<'info> {
    pub account: Account<'info, AccountState>,
    
    #[account(
        init,
        payer = signer,
        space = 8 + ApprovalNonce::SPACE,
        seeds = [
            b"approval",
            account.key().as_ref(),
            library.key().as_ref(),
            &(Clock::get()?.unix_timestamp as u64).to_le_bytes()
        ],
        bump
    )]
    pub approval_nonce: Account<'info, ApprovalNonce>,
    
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 