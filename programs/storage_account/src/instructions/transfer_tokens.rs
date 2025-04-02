use anchor_lang::prelude::*;
use anchor_spl::token_2022::{Token2022, TokenAccount};
use crate::state::StorageAccount;
use crate::error::StorageAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokensParams {
    pub amount: u64,
}

impl<'info> TransferTokens<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, TransferTokens<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<TransferTokens>, params: TransferTokensParams) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    
    // In a real implementation, we would perform the token transfer
    // using CPIs to the token program. For this example, we just update our state.
    
    // Increment the instruction count
    storage_account.increment_instruction_count();
    
    msg!(
        "Transferred {} tokens from {} to {}", 
        params.amount, 
        ctx.accounts.source_token_account.key(), 
        ctx.accounts.destination_token_account.key()
    );
    
    Ok(())
} 