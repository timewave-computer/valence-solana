use anchor_lang::prelude::*;
use crate::state::LibraryConfig;
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub authority: Pubkey,
    pub processor_program_id: Pubkey,
    pub max_transfer_amount: u64,
    pub max_batch_size: u8,
    pub fee_collector: Option<Pubkey>,
    pub enforce_recipient_allowlist: bool,
    pub allowed_recipients: Option<Vec<Pubkey>>,
    pub enforce_source_allowlist: bool,
    pub allowed_sources: Option<Vec<Pubkey>>,
    pub enforce_mint_allowlist: bool,
    pub allowed_mints: Option<Vec<Pubkey>>,
}

impl<'info> Initialize<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, Initialize<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // No additional validation needed for initialization
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = LibraryConfig::size(
            params.allowed_recipients.as_ref().map_or(0, |v| v.len()),
            params.allowed_sources.as_ref().map_or(0, |v| v.len()),
            params.allowed_mints.as_ref().map_or(0, |v| v.len()),
        ),
        seeds = [b"library_config"],
        bump
    )]
    pub library_config: Account<'info, LibraryConfig>,

    /// CHECK: The processor program that will be using this library
    pub processor_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let library_config = &mut ctx.accounts.library_config;
    
    // Validate parameters
    if params.max_batch_size > 0 && params.max_batch_size > 100 {
        return Err(TokenTransferError::InvalidBatchSize.into());
    }
    
    if params.enforce_recipient_allowlist && params.allowed_recipients.is_none() {
        return Err(TokenTransferError::EmptyAllowlist.into());
    }
    
    if params.enforce_source_allowlist && params.allowed_sources.is_none() {
        return Err(TokenTransferError::EmptyAllowlist.into());
    }
    
    if params.enforce_mint_allowlist && params.allowed_mints.is_none() {
        return Err(TokenTransferError::EmptyAllowlist.into());
    }
    
    // Initialize library config
    library_config.authority = params.authority;
    library_config.processor_program_id = Some(params.processor_program_id);
    library_config.is_active = true;
    library_config.max_transfer_amount = params.max_transfer_amount;
    library_config.max_batch_size = params.max_batch_size;
    library_config.enforce_recipient_allowlist = params.enforce_recipient_allowlist;
    library_config.allowed_recipients = params.allowed_recipients.unwrap_or_default();
    library_config.enforce_source_allowlist = params.enforce_source_allowlist;
    library_config.allowed_sources = params.allowed_sources.unwrap_or_default();
    library_config.enforce_mint_allowlist = params.enforce_mint_allowlist;
    library_config.allowed_mints = params.allowed_mints.unwrap_or_default();
    library_config.fee_collector = params.fee_collector;
    library_config.transfer_count = 0;
    library_config.total_volume = 0;
    library_config.total_fees_collected = 0;
    library_config.last_updated = Clock::get()?.unix_timestamp;
    
    // Log initialization details
    msg!("Token Transfer Library initialized");
    msg!("Authority: {}", library_config.authority);
    
    if let Some(fee_collector) = library_config.fee_collector {
        msg!("Fee collector: {}", fee_collector);
    }
    
    if library_config.max_transfer_amount > 0 {
        msg!("Max transfer amount: {}", library_config.max_transfer_amount);
    } else {
        msg!("No max transfer amount set");
    }
    
    if library_config.max_batch_size > 0 {
        msg!("Max batch size: {}", library_config.max_batch_size);
    } else {
        msg!("Batch transfers disabled");
    }
    
    if library_config.enforce_recipient_allowlist {
        msg!("Recipient allowlist enforced with {} recipients", library_config.allowed_recipients.len());
    }
    
    if library_config.enforce_source_allowlist {
        msg!("Source allowlist enforced with {} sources", library_config.allowed_sources.len());
    }
    
    if library_config.enforce_mint_allowlist {
        msg!("Mint allowlist enforced with {} mints", library_config.allowed_mints.len());
    }
    
    Ok(())
} 