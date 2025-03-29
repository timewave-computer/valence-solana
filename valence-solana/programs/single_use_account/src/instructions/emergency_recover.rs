use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(Accounts)]
pub struct EmergencyRecover<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump,
        constraint = single_use_account.authority == authority.key() @ SingleUseAccountError::UnauthorizedOwnerOperation,
        constraint = single_use_account.is_expired() @ SingleUseAccountError::AccountNotExpired
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,

    // Optional token accounts for recovery
    #[account(mut)]
    pub source_token_account: Option<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub destination_token_account: Option<Account<'info, TokenAccount>>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<EmergencyRecover>) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Validate the account has an expiration time and has expired
    if single_use_account.expiration_time.is_none() {
        return Err(SingleUseAccountError::NoExpirationTime.into());
    }
    
    let current_time = Clock::get()?.unix_timestamp;
    let expiration_time = single_use_account.expiration_time.unwrap();
    
    if current_time <= expiration_time {
        return Err(SingleUseAccountError::AccountNotExpired.into());
    }
    
    // If token accounts are provided, transfer any remaining tokens to the destination
    if let (Some(source), Some(destination)) = (&ctx.accounts.source_token_account, &ctx.accounts.destination_token_account) {
        let amount = source.amount;
        
        if amount > 0 {
            // Transfer tokens from the source to destination
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
                        from: source.to_account_info(),
                        to: destination.to_account_info(),
                        authority: single_use_account.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount,
            ).map_err(|_| SingleUseAccountError::TokenTransferFailed.into())?;
            
            msg!("Recovered {} tokens from {} to {}", 
                amount, 
                source.key(), 
                destination.key()
            );
        }
    }
    
    // Mark the account as used if it wasn't already
    if !single_use_account.was_used {
        single_use_account.mark_as_used();
    }
    
    msg!(
        "Emergency recovery completed for expired account at time: {} (expiration: {})", 
        current_time,
        expiration_time
    );
    
    Ok(())
} 