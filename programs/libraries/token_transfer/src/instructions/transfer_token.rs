use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token_2022::{self, Token2022, Transfer};
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

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(
        seeds = [b"library_config"],
        bump,
        constraint = library_config.is_active @ TokenTransferError::LibraryInactive,
        constraint = processor_program.key() == library_config.processor_program_id.expect("processor not set") @ TokenTransferError::InvalidProcessorProgram,
    )]
    pub library_config: Account<'info, LibraryConfig>,

    /// CHECK: The processor program that is calling this library
    pub processor_program: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = source_account.owner == *authority.key @ TokenTransferError::OwnerMismatch,
        constraint = source_account.mint == mint.key() @ TokenTransferError::MintMismatch,
    )]
    pub source_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = destination_account.mint == mint.key() @ TokenTransferError::MintMismatch,
    )]
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
    let mut _minimum_amount_out = amount;
    
    // If slippage protection is enabled, apply it
    if let Some(slippage_bps) = params.slippage_bps {
        if slippage_bps > 0 {
            // Validate slippage is within reasonable bounds (max 5% = 500 bps)
            if slippage_bps > 500 {
                return Err(TokenTransferError::InvalidAmount.into());
            }
            
            // Calculate slippage amount (in basis points, 10000 = 100%)
            let slippage_amount = amount * slippage_bps as u64 / 10000;
            
            // Set minimum amount out accounting for slippage
            _minimum_amount_out = amount.saturating_sub(slippage_amount);
            
            msg!("Slippage protection enabled: {}bps, minimum amount out: {}", 
                 slippage_bps, _minimum_amount_out);
        }
    }
    
    // TODO: Implement actual slippage validation in future swap integration
    
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
    if fee_amount > 0 {
        if let Some(fee_collector) = &ctx.accounts.fee_collector {
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
            library_config.add_fees_collected(fee_amount)?;
        }
    }

    // Update library config transfer count and volume
    library_config.increment_transfer_count()?;
    library_config.add_volume(amount)?;

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