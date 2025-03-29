use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokensParams {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump,
        constraint = single_use_account.authority == authority.key() @ SingleUseAccountError::UnauthorizedOwnerOperation,
        constraint = !single_use_account.was_used @ SingleUseAccountError::AccountAlreadyUsed
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,
    
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<TransferTokens>, params: TransferTokensParams) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Validate the account hasn't been used
    if single_use_account.was_used {
        return Err(SingleUseAccountError::AccountAlreadyUsed.into());
    }
    
    // Check that the destination meets any required destination constraints
    if !single_use_account.validate_destination(&ctx.accounts.destination_token_account.owner) {
        return Err(SingleUseAccountError::InvalidDestination.into());
    }
    
    // Get the single-use account PDA seeds for signing
    let single_use_account_bump = ctx.bumps.single_use_account;
    let seeds = &[
        b"single_use_account".as_ref(),
        ctx.accounts.authority.key().as_ref(),
        &[single_use_account_bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // Transfer tokens
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.source_token_account.to_account_info(),
                to: ctx.accounts.destination_token_account.to_account_info(),
                authority: single_use_account.to_account_info(),
            },
            signer_seeds,
        ),
        params.amount,
    ).map_err(|_| SingleUseAccountError::TokenTransferFailed.into())?;
    
    // Increment the instruction count
    single_use_account.increment_instruction_count();
    
    msg!(
        "Transferred {} tokens from {} to {}", 
        params.amount, 
        ctx.accounts.source_token_account.key(), 
        ctx.accounts.destination_token_account.key()
    );
    
    Ok(())
} 