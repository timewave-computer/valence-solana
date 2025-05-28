use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use crate::state::{LibraryConfig, TransferDestination};
use crate::error::TokenTransferError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchTransferParams {
    pub destinations: Vec<TransferDestination>,
    pub total_amount: u64,
    pub fee_amount: Option<u64>,
}

pub fn handler<'a>(
    _ctx: Context<'_, '_, '_, 'a, BatchTransfer<'a>>,
    _params: BatchTransferParams,
) -> Result<()> {
    // TODO: Implement batch transfer functionality
    // Currently disabled due to lifetime complexity with remaining_accounts
    // Individual transfers should be used instead
    return Err(TokenTransferError::BatchTransfersDisabled.into());
}

#[derive(Accounts)]
#[instruction(params: BatchTransferParams)]
pub struct BatchTransfer<'info> {
    #[account(
        mut,
        constraint = source_account.owner == authority.key() @ TokenTransferError::OwnerMismatch
    )]
    pub source_account: InterfaceAccount<'info, TokenAccount>,
    
    /// CHECK: Authority for the source account - validated by token program
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [b"library_config".as_ref()],
        bump,
        constraint = library_config.is_active @ TokenTransferError::LibraryInactive
    )]
    pub library_config: Account<'info, LibraryConfig>,
    
    /// CHECK: Processor program - validated against library config
    pub processor_program: UncheckedAccount<'info>,
    
    /// Optional fee collector account
    #[account(mut)]
    pub fee_collector: Option<InterfaceAccount<'info, TokenAccount>>,
    
    pub token_program: Interface<'info, TokenInterface>,
    
    // Remaining accounts should be destination token accounts in order
    // This allows for dynamic number of destinations while maintaining type safety
} 