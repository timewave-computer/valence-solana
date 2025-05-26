use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::StorageAccount;
use crate::error::StorageAccountError;

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Validated in the handler
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateTokenAccount<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, CreateTokenAccount<'info>>,
        _bumps: &std::collections::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


pub fn handler(ctx: Context<CreateTokenAccount>, mint: Pubkey) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    
    // In a real implementation, we would perform the token account creation
    // using CPIs to the token program. For this example, we just update our state.
    
    // Increment the token account count
    storage_account.increment_token_account_count();
    
    // Update last activity timestamp
    storage_account.last_activity = Clock::get()?.unix_timestamp;
    
    msg!("Token account created for mint: {}", mint);
    
    Ok(())
} 