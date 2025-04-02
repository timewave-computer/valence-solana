use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use std::collections::BTreeMap;
use crate::state::AccountState;
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<CreateTokenAccount>) -> Result<()> {
    let account_state = &ctx.accounts.account;
    
    // Only the owner can create token accounts
    if account_state.owner != ctx.accounts.payer.key() {
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Add the token account to the tracked accounts
    let mut account_state = ctx.accounts.account.to_account_info();
    let mut account_state_data = AccountState::try_from_slice(&account_state.data.borrow())?;
    account_state_data.add_token_account(ctx.accounts.token_account.key())?;
    account_state_data.serialize(&mut *account_state.data.borrow_mut())?;
    
    msg!("Created token account: {}", ctx.accounts.token_account.key());
    
    Ok(())
}

impl<'info> CreateTokenAccount<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, CreateTokenAccount<'info>>,
        _bumps: &BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// CHECK: This account is used as the authority for the token account
    pub recipient: UncheckedAccount<'info>,
    
    /// CHECK: This account is used as the vault authority
    pub vault_authority: UncheckedAccount<'info>,
    
    pub mint: Account<'info, Mint>,
    
    /// Token program account
    pub token_program: Program<'info, Token>,
    
    /// Payer account
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Token account of the user
    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 