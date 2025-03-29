use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferInfo {
    /// The amount of tokens to transfer
    pub amount: u64,
    /// Index of the destination token account in the accounts array
    pub destination_index: u8,
    /// Optional memo for the specific transfer
    pub memo: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchTransferParams {
    /// Vector of transfer information
    pub transfers: Vec<TransferInfo>,
}

#[derive(Accounts)]
pub struct BatchTransfer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_transfer_config", authority.key().as_ref()],
        bump,
        constraint = config.is_active @ TokenTransferError::UnauthorizedOperation,
    )]
    pub config: Account<'info, LibraryConfig>,
    
    /// The source token account for all transfers
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,
    
    /// Multiple destination token accounts
    #[account(mut)]
    pub destination_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// Optional fee token account (if fees are enabled)
    #[account(mut)]
    pub fee_account: Option<Account<'info, TokenAccount>>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BatchTransfer>, params: BatchTransferParams) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let source = &ctx.accounts.source_token_account;
    let destination_accounts = &ctx.accounts.destination_token_accounts;
    
    // Validate batch size
    if params.transfers.is_empty() {
        return Err(error!(TokenTransferError::InvalidBatchSize));
    }
    
    if config.max_batch_size > 0 && params.transfers.len() > config.max_batch_size as usize {
        return Err(error!(TokenTransferError::InvalidBatchSize));
    }
    
    // Validate source allowlist if enabled
    if config.enforce_source_allowlist && !config.allowed_sources.contains(&source.owner) {
        return Err(error!(TokenTransferError::SourceOwnerMismatch));
    }
    
    // Calculate total amount to transfer and validate mints
    let mut total_amount: u64 = 0;
    
    for transfer in &params.transfers {
        // Validate transfer amount
        if transfer.amount == 0 {
            return Err(error!(TokenTransferError::InvalidAmount));
        }
        
        // Avoid overflow
        total_amount = total_amount.checked_add(transfer.amount)
            .ok_or(error!(TokenTransferError::CalculationOverflow))?;
            
        // Validate destination index
        if transfer.destination_index as usize >= destination_accounts.len() {
            return Err(error!(TokenTransferError::InvalidDestinationIndex));
        }
        
        let destination = &destination_accounts[transfer.destination_index as usize];
        
        // Validate token mint consistency
        if source.mint != destination.mint {
            return Err(error!(TokenTransferError::MintMismatch));
        }
        
        // Validate destination allowlist if enabled
        if config.enforce_recipient_allowlist && !config.allowed_recipients.contains(&destination.owner) {
            return Err(error!(TokenTransferError::RecipientNotAllowed));
        }
    }
    
    // Validate allowed mints if enabled
    if config.enforce_mint_allowlist && !config.allowed_mints.contains(&source.mint) {
        return Err(error!(TokenTransferError::WrongTokenMint));
    }
    
    // Validate total transfer amount
    if config.max_transfer_amount > 0 && total_amount > config.max_transfer_amount {
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    // Calculate total fee if applicable
    let mut total_fee: u64 = 0;
    
    if config.fee_bps > 0 {
        total_fee = total_amount.saturating_mul(config.fee_bps as u64) / 10000;
    }
    
    // Validate we have a fee account if fees are enabled
    if total_fee > 0 && ctx.accounts.fee_account.is_none() {
        return Err(error!(TokenTransferError::MissingFeeAccount));
    }
    
    // Perform transfers
    for transfer in &params.transfers {
        let destination_index = transfer.destination_index as usize;
        let destination = &destination_accounts[destination_index];
        let transfer_amount = transfer.amount;
        
        // Execute the transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: destination.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, transfer_amount)?;
        
        msg!("Transferred {} tokens to {}", 
             transfer_amount, 
             destination.key());
             
        if let Some(memo) = &transfer.memo {
            msg!("Memo: {}", memo);
        }
    }
    
    // Transfer fee if applicable
    if total_fee > 0 && ctx.accounts.fee_account.is_some() {
        let fee_receiver = config.fee_collector.unwrap_or(config.authority);
        let fee_account = ctx.accounts.fee_account.as_ref().unwrap();
        
        // Validate fee account
        if fee_account.mint != source.mint {
            return Err(error!(TokenTransferError::MintMismatch));
        }
        
        if fee_account.owner != fee_receiver {
            return Err(error!(TokenTransferError::DestinationOwnerMismatch));
        }
        
        // Execute fee transfer
        let fee_cpi_accounts = Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: fee_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let fee_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            fee_cpi_accounts,
        );
        
        token::transfer(fee_cpi_ctx, total_fee)?;
        
        // Update fee statistics
        config.add_fees_collected(total_fee);
        msg!("Fee of {} tokens collected", total_fee);
    }
    
    // Update statistics
    config.increment_transfer_count();
    config.add_volume(total_amount);
    
    msg!("Batch transfer completed: {} transfers for total amount {}", 
         params.transfers.len(), 
         total_amount);
    
    Ok(())
} 