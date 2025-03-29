use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferWithAuthorityParams {
    /// The amount of tokens to transfer
    pub amount: u64,
    /// Token owner PDA bump
    pub bump: u8,
    /// Optional memo for the transaction
    pub memo: Option<String>,
}

#[derive(Accounts)]
pub struct TransferWithAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_transfer_config", authority.key().as_ref()],
        bump,
        constraint = config.is_active @ TokenTransferError::UnauthorizedOperation,
    )]
    pub config: Account<'info, LibraryConfig>,
    
    /// The source token account
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,
    
    /// The destination token account
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    /// The owner (or delegate) of the source token account who gives permission
    /// This is typically a PDA that the program can sign for
    #[account(seeds = [b"token_authority", source_token_account.key().as_ref()], bump = params.bump)]
    pub token_authority: UncheckedAccount<'info>,
    
    /// Optional fee token account (if fees are enabled)
    #[account(mut)]
    pub fee_account: Option<Account<'info, TokenAccount>>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<TransferWithAuthority>, params: TransferWithAuthorityParams) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let source = &ctx.accounts.source_token_account;
    let destination = &ctx.accounts.destination_token_account;
    let amount = params.amount;
    
    // Validate transfer amount
    if amount == 0 {
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    if config.max_transfer_amount > 0 && amount > config.max_transfer_amount {
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    // Validate token mint consistency
    if source.mint != destination.mint {
        return Err(error!(TokenTransferError::MintMismatch));
    }
    
    // Validate allowed mints if enabled
    if config.enforce_mint_allowlist && !config.allowed_mints.contains(&source.mint) {
        return Err(error!(TokenTransferError::WrongTokenMint));
    }
    
    // Validate source allowlist if enabled
    if config.enforce_source_allowlist && !config.allowed_sources.contains(&source.owner) {
        return Err(error!(TokenTransferError::SourceOwnerMismatch));
    }
    
    // Validate destination allowlist if enabled
    if config.enforce_recipient_allowlist && !config.allowed_recipients.contains(&destination.owner) {
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
    
    // Execute the transfer with PDA signing
    let token_auth_seeds = &[
        b"token_authority",
        source.to_account_info().key.as_ref(),
        &[params.bump],
    ];
    
    let signer_seeds = &[&token_auth_seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.source_token_account.to_account_info(),
        to: ctx.accounts.destination_token_account.to_account_info(),
        authority: ctx.accounts.token_authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    
    token::transfer(cpi_ctx, transfer_amount)?;
    
    // Transfer fee if applicable
    if fee_amount > 0 && ctx.accounts.fee_account.is_some() {
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
            authority: ctx.accounts.token_authority.to_account_info(),
        };
        
        let fee_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            fee_cpi_accounts,
            signer_seeds,
        );
        
        token::transfer(fee_cpi_ctx, fee_amount)?;
        
        // Update fee statistics
        config.add_fees_collected(fee_amount);
        msg!("Fee of {} tokens collected", fee_amount);
    }
    
    // Update statistics
    config.increment_transfer_count();
    config.add_volume(amount);
    
    msg!("Transferred {} tokens from {} to {} using delegated authority", 
         transfer_amount, 
         ctx.accounts.source_token_account.key(), 
         ctx.accounts.destination_token_account.key());
    
    if let Some(memo) = &params.memo {
        msg!("Memo: {}", memo);
    }
    
    Ok(())
} 