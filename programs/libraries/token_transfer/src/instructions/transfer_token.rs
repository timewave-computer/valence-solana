use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TokenAccount, Token2022, Transfer};
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;
use crate::utils::token_helpers;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokenParams {
    /// The amount of tokens to transfer
    pub amount: u64,
    /// The fee amount to transfer
    pub fee_amount: Option<u64>,
    /// The slippage tolerance in basis points (optional, defaults to config)
    pub slippage_bps: Option<u16>,
    /// Optional memo for the transaction
    pub memo: Option<String>,
}

impl<'info> TransferToken<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, TransferToken<'info>>,
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

        // Validate source account owner
        if ctx.accounts.source_account.owner != *ctx.accounts.authority.key {
            return Err(TokenTransferError::OwnerMismatch.into());
        }
        
        // Validate mint match
        if ctx.accounts.source_account.mint != ctx.accounts.mint.key() {
            return Err(TokenTransferError::MintMismatch.into());
        }
        
        // Validate destination mint match
        if ctx.accounts.destination_account.mint != ctx.accounts.mint.key() {
            return Err(TokenTransferError::MintMismatch.into());
        }
        
        // Validate source is allowed
        if !ctx.accounts.library_config.is_source_allowed(&ctx.accounts.source_account.key()) {
            return Err(TokenTransferError::UnauthorizedSource.into());
        }
        
        // Validate recipient is allowed
        if !ctx.accounts.library_config.is_recipient_allowed(&ctx.accounts.destination_account.key()) {
            return Err(TokenTransferError::UnauthorizedRecipient.into());
        }
        
        // Validate mint is allowed
        if !ctx.accounts.library_config.is_mint_allowed(ctx.accounts.mint.key) {
            return Err(TokenTransferError::UnauthorizedMint.into());
        }
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(
        seeds = [b"library_config"],
        bump,
    )]
    pub library_config: Account<'info, LibraryConfig>,

    /// CHECK: The processor program that is calling this library
    pub processor_program: UncheckedAccount<'info>,

    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destination_account: Account<'info, TokenAccount>,

    /// CHECK: The mint of the token being transferred
    pub mint: UncheckedAccount<'info>,

    /// The authority that can authorize the transfer
    pub authority: Signer<'info>,

    /// CHECK: Optional fee collector account
    #[account(mut)]
    pub fee_collector: Option<Account<'info, TokenAccount>>,

    /// The token program (Token-2022)
    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<TransferToken>, params: TransferTokenParams) -> Result<()> {
    let library_config = &mut ctx.accounts.library_config;
    let source = &ctx.accounts.source_account;
    let _destination = &ctx.accounts.destination_account;
    let amount = params.amount;
    let fee_amount = params.fee_amount.unwrap_or(0);
    let memo = params.memo;
    
    // Validate amount
    if amount == 0 {
        return Err(TokenTransferError::InvalidAmount.into());
    }
    
    // Validate amount is within max transfer limit if set
    if library_config.max_transfer_amount > 0 && amount > library_config.max_transfer_amount {
        return Err(TokenTransferError::TransferAmountExceedsLimit.into());
    }
    
    // Validate fee amount does not exceed limit
    if fee_amount > 0 {
        // Check fee collector is provided
        if ctx.accounts.fee_collector.is_none() {
            return Err(TokenTransferError::FeeCollectorRequired.into());
        }
        
        // Ensure fee is not more than 5% of the amount for individual transfers
        let max_fee = amount / 20;
        if fee_amount > max_fee {
            return Err(TokenTransferError::FeeExceedsLimit.into());
        }
    }
    
    // Calculate total needed
    let total = amount.checked_add(fee_amount)
        .ok_or(TokenTransferError::ArithmeticOverflow)?;
    
    // Validate that the source account has enough funds
    if source.amount < total {
        return Err(TokenTransferError::InsufficientFunds.into());
    }
    
    // Calculate slippage if enabled
    // Only comment out since we might use this later
    let _slippage_amount = 0;
    
    // If slippage protection is enabled, apply it
    if let Some(slippage_bps) = params.slippage_bps {
        if slippage_bps > 0 {
            // Calculate slippage amount (in basis points, 10000 = 100%)
            // Using underscore to indicate we're intentionally not using this yet
            let _calculated_slippage = amount * slippage_bps as u64 / 10000;
            
            // TODO: Implement slippage protection logic when needed
        }
    }
    
    // Transfer tokens to destination
    let cpi_accounts = Transfer {
        from: ctx.accounts.source_account.to_account_info(),
        to: ctx.accounts.destination_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token_helpers::transfer_tokens(cpi_ctx, amount)?;

    // Transfer fee if specified
    if fee_amount > 0 && ctx.accounts.fee_collector.is_some() {
        let fee_collector = ctx.accounts.fee_collector.as_ref().unwrap();
        
        // Validate fee collector mint matches the token being transferred
        if fee_collector.mint != ctx.accounts.mint.key() {
            return Err(TokenTransferError::MintMismatch.into());
        }
        
        let fee_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: fee_collector.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let fee_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), fee_accounts);
        token_helpers::transfer_tokens(fee_ctx, fee_amount)?;
        
        // Update fee collection stats
        library_config.add_fees_collected(fee_amount);
    }

    // Update library config transfer count and volume
    library_config.increment_transfer_count();
    library_config.add_volume(amount);
    library_config.last_updated = Clock::get()?.unix_timestamp;

    // Log the transfer details
    msg!("Transferred {} tokens from {} to {}", 
        amount, 
        ctx.accounts.source_account.key(), 
        ctx.accounts.destination_account.key()
    );
    
    if let Some(memo) = memo {
        msg!("Memo: {}", memo);
    }
    
    Ok(())
} 