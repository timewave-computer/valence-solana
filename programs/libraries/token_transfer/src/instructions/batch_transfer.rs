use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TokenAccount, Token2022, Transfer};
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;
use crate::utils::token_helpers;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferDestination {
    pub destination: Pubkey,
    pub amount: u64,
    pub memo: Option<String>,
}

impl<'info> BatchTransfer<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, BatchTransfer<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Validate library is active
        if !ctx.accounts.library_config.is_active {
            return Err(TokenTransferError::LibraryInactive.into());
        }
        
        // Validate processor program
        if ctx.accounts.processor_program.key() != ctx.accounts.library_config.processor_program_id.expect("processor not set") {
            return Err(TokenTransferError::InvalidProcessorProgram.into());
        }
        
        // Validate batch transfers are enabled
        if ctx.accounts.library_config.max_batch_size == 0 {
            return Err(TokenTransferError::BatchTransfersDisabled.into());
        }
        
        // Validate source account owner
        if ctx.accounts.source_account.owner != *ctx.accounts.authority.key {
            return Err(TokenTransferError::OwnerMismatch.into());
        }
        
        // Validate source is allowed
        if !ctx.accounts.library_config.is_source_allowed(&ctx.accounts.source_account.key()) {
            return Err(TokenTransferError::UnauthorizedSource.into());
        }
        
        Ok(())
    }
}


/// Batch transfer parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchTransferParams {
    pub destinations: Vec<TransferDestination>,
    pub fee_amount: Option<u64>,
}

/// Accounts required for batch transfer operation
#[derive(Accounts)]
pub struct BatchTransfer<'info> {
    #[account(
        seeds = [b"library_config"],
        bump,
    )]
    pub library_config: Account<'info, LibraryConfig>,

    /// CHECK: The processor program that is calling this library
    pub processor_program: UncheckedAccount<'info>,

    /// The source token account for all transfers
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,

    /// The authority that can authorize the transfers
    pub authority: Signer<'info>,

    /// CHECK: Optional fee collector account
    #[account(mut)]
    pub fee_collector: Option<Account<'info, TokenAccount>>,

    /// The token program (Token-2022)
    pub token_program: Program<'info, Token2022>,
}

// The remaining accounts represent destination token accounts
// They must be passed in the same order as destinations in params
// Each destination account is validated inside the instruction handler

pub fn handler(ctx: Context<BatchTransfer>, params: BatchTransferParams) -> Result<()> {
    let library_config = &mut ctx.accounts.library_config;
    let destinations = &params.destinations;
    let source = &ctx.accounts.source_account;
    let fee_amount = params.fee_amount.unwrap_or(0);
    
    // Validate number of destinations doesn't exceed max batch size
    if destinations.len() > library_config.max_batch_size as usize {
        return Err(TokenTransferError::BatchSizeExceeded.into());
    }
    
    // Validate number of destinations matches number of remaining accounts
    if destinations.len() != ctx.remaining_accounts.len() {
        return Err(TokenTransferError::AccountMismatch.into());
    }
    
    // Calculate total amount to transfer
    let mut total_amount: u64 = 0;
    for dest in destinations {
        if dest.amount == 0 {
            return Err(TokenTransferError::InvalidAmount.into());
        }
        
        // Validate individual amount doesn't exceed max transfer limit if set
        if library_config.max_transfer_amount > 0 && dest.amount > library_config.max_transfer_amount {
            return Err(TokenTransferError::TransferAmountExceedsLimit.into());
        }
        
        // Check for overflow
        total_amount = total_amount.checked_add(dest.amount)
            .ok_or(TokenTransferError::ArithmeticOverflow)?;
    }
    
    // Validate fee amount does not exceed limit
    if fee_amount > 0 {
        // Check fee collector is provided if fee amount is set
        if ctx.accounts.fee_collector.is_none() {
            return Err(TokenTransferError::FeeCollectorRequired.into());
        }
        
        // Ensure fee is not more than 5% of the total amount for batch transfers
        let max_fee = total_amount / 20;
        if fee_amount > max_fee {
            return Err(TokenTransferError::FeeExceedsLimit.into());
        }
    }
    
    // Calculate total needed
    let total_needed = total_amount.checked_add(fee_amount)
        .ok_or(TokenTransferError::ArithmeticOverflow)?;
    
    // Validate that the source account has enough funds
    if source.amount < total_needed {
        return Err(TokenTransferError::InsufficientFunds.into());
    }
    
    // Transfer tokens to all destinations
    for (i, dest) in destinations.iter().enumerate() {
        let destination_account_info = &ctx.remaining_accounts[i];
        
        // Deserialize as TokenAccount to validate
        let destination_account = Account::<TokenAccount>::try_from(destination_account_info)?;
        
        // Validate destination account is correct
        if destination_account.key() != dest.destination {
            return Err(TokenTransferError::AccountMismatch.into());
        }
        
        // Validate mint matches source account
        if destination_account.mint != source.mint {
            return Err(TokenTransferError::MintMismatch.into());
        }
        
        // Validate allowed recipient if enforced
        if !library_config.is_recipient_allowed(&destination_account.key()) {
            return Err(TokenTransferError::UnauthorizedRecipient.into());
        }
        
        // Execute transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info().clone(),
            to: destination_account_info.clone(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token_helpers::transfer_tokens(cpi_ctx, dest.amount)?;
        
        // Log transfer
        msg!("Transferred {} tokens to {}", dest.amount, dest.destination);
        if let Some(memo) = &dest.memo {
            msg!("Memo: {}", memo);
        }
        
        // Update library volume stats
        library_config.add_volume(dest.amount);
        library_config.increment_transfer_count();
    }
    
    // Transfer fee if specified
    if fee_amount > 0 && ctx.accounts.fee_collector.is_some() {
        let fee_collector = ctx.accounts.fee_collector.as_ref().unwrap();
        
        // Validate fee collector mint matches the source account
        if fee_collector.mint != source.mint {
            return Err(TokenTransferError::MintMismatch.into());
        }
        
        let fee_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info().clone(),
            to: fee_collector.to_account_info().clone(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        };
        
        let fee_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), fee_accounts);
        token_helpers::transfer_tokens(fee_ctx, fee_amount)?;
        
        // Update fee collection stats
        library_config.add_fees_collected(fee_amount);
        msg!("Fee of {} tokens collected", fee_amount);
    }
    
    library_config.last_updated = Clock::get()?.unix_timestamp;
    msg!("Batch transfer completed with {} destinations", destinations.len());
    
    Ok(())
} 