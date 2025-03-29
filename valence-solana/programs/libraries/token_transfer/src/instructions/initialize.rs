use anchor_lang::prelude::*;
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeParams {
    /// The processor program ID (if restricted)
    pub processor_program_id: Option<Pubkey>,
    /// Maximum transfer size in tokens (0 for unlimited)
    pub max_transfer_amount: u64,
    /// Maximum batch transfer size (0 for unlimited)
    pub max_batch_size: u8,
    /// Whether to enforce recipient allowlisting
    pub enforce_recipient_allowlist: bool,
    /// List of allowed recipient addresses (if enforce_recipient_allowlist is true)
    pub allowed_recipients: Vec<Pubkey>,
    /// Whether to enforce source allowlisting
    pub enforce_source_allowlist: bool,
    /// List of allowed source addresses (if enforce_source_allowlist is true)
    pub allowed_sources: Vec<Pubkey>,
    /// Whether to enforce token mint allowlisting
    pub enforce_mint_allowlist: bool,
    /// List of allowed token mints (if enforce_mint_allowlist is true)
    pub allowed_mints: Vec<Pubkey>,
    /// Whether to validate that token accounts belong to the provided owner
    pub validate_account_ownership: bool,
    /// Whether to enable slippage protection
    pub enable_slippage_protection: bool,
    /// Default slippage tolerance in basis points (e.g., 100 = 1%)
    pub default_slippage_bps: u16,
    /// Fee in basis points (e.g., 25 = 0.25%)
    pub fee_bps: u16,
    /// Fee collector address
    pub fee_collector: Option<Pubkey>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = LibraryConfig::size(
            params.allowed_recipients.len(),
            params.allowed_sources.len(),
            params.allowed_mints.len(),
        ),
        seeds = [b"token_transfer_config", authority.key().as_ref()],
        bump
    )]
    pub config: Account<'info, LibraryConfig>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let authority = ctx.accounts.authority.key();
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate parameters
    if params.fee_bps > 1000 {
        // Limit fee to a maximum of 10%
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    if params.default_slippage_bps > 5000 {
        // Limit default slippage to a maximum of 50%
        return Err(error!(TokenTransferError::InvalidAmount));
    }
    
    // Initialize config
    config.authority = authority;
    config.is_active = true;
    config.processor_program_id = params.processor_program_id;
    config.max_transfer_amount = params.max_transfer_amount;
    config.max_batch_size = params.max_batch_size;
    config.enforce_recipient_allowlist = params.enforce_recipient_allowlist;
    config.allowed_recipients = params.allowed_recipients;
    config.enforce_source_allowlist = params.enforce_source_allowlist;
    config.allowed_sources = params.allowed_sources;
    config.enforce_mint_allowlist = params.enforce_mint_allowlist;
    config.allowed_mints = params.allowed_mints;
    config.validate_account_ownership = params.validate_account_ownership;
    config.enable_slippage_protection = params.enable_slippage_protection;
    config.default_slippage_bps = params.default_slippage_bps;
    config.fee_bps = params.fee_bps;
    config.fee_collector = params.fee_collector;
    config.transfer_count = 0;
    config.total_volume = 0;
    config.total_fees_collected = 0;
    config.last_updated = current_time;
    config.reserved = [0; 64];
    
    msg!("Token Transfer Library initialized with authority: {}", config.authority);
    
    if config.fee_bps > 0 {
        if let Some(fee_collector) = config.fee_collector {
            msg!("Fee set to {} basis points, collector: {}", config.fee_bps, fee_collector);
        } else {
            msg!("Fee set to {} basis points, collector: authority", config.fee_bps);
        }
    }
    
    // Log security settings
    if config.enforce_recipient_allowlist {
        msg!("Recipient allowlist enforced with {} allowed addresses", config.allowed_recipients.len());
    }
    
    if config.enforce_source_allowlist {
        msg!("Source allowlist enforced with {} allowed addresses", config.allowed_sources.len());
    }
    
    if config.enforce_mint_allowlist {
        msg!("Token mint allowlist enforced with {} allowed mints", config.allowed_mints.len());
    }
    
    if config.max_transfer_amount > 0 {
        msg!("Maximum transfer amount: {}", config.max_transfer_amount);
    }
    
    if config.max_batch_size > 0 {
        msg!("Maximum batch size: {}", config.max_batch_size);
    }
    
    Ok(())
} 