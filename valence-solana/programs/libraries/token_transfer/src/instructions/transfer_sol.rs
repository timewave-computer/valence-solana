use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferSolParams {
    /// The amount of lamports to transfer
    pub amount: u64,
    /// Optional memo for the transaction
    pub memo: Option<String>,
}

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_transfer_config", authority.key().as_ref()],
        bump,
        constraint = config.is_active @ TokenTransferError::UnauthorizedOperation,
    )]
    pub config: Account<'info, LibraryConfig>,
    
    /// The source wallet (must be a signer)
    #[account(mut)]
    pub source_wallet: Signer<'info>,
    
    /// The destination wallet
    #[account(mut)]
    pub destination_wallet: SystemAccount<'info>,
    
    /// Optional fee receiver (if fees are enabled)
    #[account(mut)]
    pub fee_receiver: Option<SystemAccount<'info>>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TransferSol>, params: TransferSolParams) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let source = &ctx.accounts.source_wallet;
    let destination = &ctx.accounts.destination_wallet;
    let amount = params.amount;
    
    // Validate transfer amount
    if amount == 0 {
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    if config.max_transfer_amount > 0 && amount > config.max_transfer_amount {
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    // Validate source allowlist if enabled
    if config.enforce_source_allowlist && !config.allowed_sources.contains(&source.key()) {
        return Err(error!(TokenTransferError::SourceOwnerMismatch));
    }
    
    // Validate destination allowlist if enabled
    if config.enforce_recipient_allowlist && !config.allowed_recipients.contains(&destination.key()) {
        return Err(error!(TokenTransferError::RecipientNotAllowed));
    }
    
    // Calculate fee if applicable
    let mut fee_amount: u64 = 0;
    let transfer_amount: u64;
    
    if config.fee_bps > 0 {
        fee_amount = amount.saturating_mul(config.fee_bps as u64) / 10000;
        transfer_amount = amount.saturating_sub(fee_amount);
    } else {
        transfer_amount = amount;
    }
    
    // Execute the transfer
    let cpi_accounts = Transfer {
        from: source.to_account_info(),
        to: destination.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts,
    );
    
    system_program::transfer(cpi_ctx, transfer_amount)?;
    
    // Transfer fee if applicable
    if fee_amount > 0 && ctx.accounts.fee_receiver.is_some() {
        let fee_account = ctx.accounts.fee_receiver.as_ref().unwrap();
        let fee_receiver_key = fee_account.key();
        let config_fee_receiver = config.fee_collector.unwrap_or(config.authority);
        
        // Validate fee receiver matches expected collector
        if fee_receiver_key != config_fee_receiver {
            return Err(error!(TokenTransferError::DestinationOwnerMismatch));
        }
        
        // Execute fee transfer
        let fee_cpi_accounts = Transfer {
            from: source.to_account_info(),
            to: fee_account.to_account_info(),
        };
        
        let fee_cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            fee_cpi_accounts,
        );
        
        system_program::transfer(fee_cpi_ctx, fee_amount)?;
        
        // Update fee statistics
        config.add_fees_collected(fee_amount);
        msg!("Fee of {} lamports collected", fee_amount);
    }
    
    // Update statistics
    config.increment_transfer_count();
    config.add_volume(amount);
    
    msg!("Transferred {} lamports from {} to {}", 
         transfer_amount, 
         source.key(), 
         destination.key());
    
    if let Some(memo) = &params.memo {
        msg!("Memo: {}", memo);
    }
    
    Ok(())
} 