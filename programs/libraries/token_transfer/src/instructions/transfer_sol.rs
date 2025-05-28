use anchor_lang::prelude::*;
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferSolParams {
    pub amount: u64,
    pub fee_amount: Option<u64>,
    pub memo: Option<String>,
}

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(
        seeds = [b"library_config"],
        bump,
        constraint = library_config.is_active @ TokenTransferError::LibraryInactive,
        constraint = library_config.processor_program_id == Some(processor_program.key()) @ TokenTransferError::InvalidProcessorProgram,
    )]
    pub library_config: Account<'info, LibraryConfig>,

    /// CHECK: The processor program that is calling this library
    pub processor_program: UncheckedAccount<'info>,

    #[account(mut)]
    pub source: Signer<'info>,

    /// CHECK: The SOL recipient
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Optional fee collector account
    #[account(mut)]
    pub fee_collector: Option<UncheckedAccount<'info>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TransferSol>, params: TransferSolParams) -> Result<()> {
    let library_config = &mut ctx.accounts.library_config;
    let amount = params.amount;
    let fee_amount = params.fee_amount.unwrap_or(0);

    // Validate amount does not exceed max transfer limit if set
    if library_config.max_transfer_amount > 0 && amount > library_config.max_transfer_amount {
        return Err(TokenTransferError::TransferAmountExceedsLimit.into());
    }

    // Validate fee collector is not the same as recipient if provided
    if let Some(fee_collector) = &ctx.accounts.fee_collector {
        if fee_collector.key() == ctx.accounts.recipient.key() {
            return Err(TokenTransferError::AccountMismatch.into());
        }
    }

    // Validate fee amount does not exceed limit
    if fee_amount > 0 {
        // Check fee collector is provided if fee amount is set
        if ctx.accounts.fee_collector.is_none() {
            return Err(TokenTransferError::FeeCollectorRequired.into());
        }

        // Ensure fee is not more than 10% of the total amount
        let max_fee = amount / 10;
        if fee_amount > max_fee {
            return Err(TokenTransferError::FeeExceedsLimit.into());
        }
    }

    // Calculate total amount needed
    let total_needed = amount + fee_amount;

    // Check sender has enough SOL
    let sender_balance = ctx.accounts.source.try_lamports()?;
    if sender_balance < total_needed {
        return Err(TokenTransferError::InsufficientFunds.into());
    }

    // Transfer SOL to recipient
    let sol_transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        ctx.accounts.source.key,
        ctx.accounts.recipient.key,
        amount,
    );

    anchor_lang::solana_program::program::invoke(
        &sol_transfer_ix,
        &[
            ctx.accounts.source.to_account_info(),
            ctx.accounts.recipient.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Transfer fee if applicable
    if fee_amount > 0 {
        if let Some(fee_collector) = &ctx.accounts.fee_collector {
            let fee_transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.source.key,
                fee_collector.key,
                fee_amount,
            );

            anchor_lang::solana_program::program::invoke(
                &fee_transfer_ix,
                &[
                    ctx.accounts.source.to_account_info(),
                    fee_collector.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
            
            // Update fee collection stats
            library_config.add_fees_collected(fee_amount)?;
        }
    }

    // Update library config transfer count and volume
    library_config.increment_transfer_count()?;
    library_config.add_volume(amount)?;

    // Log the transfer details
    msg!("Transferred {} SOL from {} to {}", 
        amount, 
        ctx.accounts.source.key(), 
        ctx.accounts.recipient.key()
    );
    
    if let Some(memo) = params.memo {
        msg!("Memo: {}", memo);
    }
    
    Ok(())
} 