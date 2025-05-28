#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("TknTrfzGGGtAL3YxG8TKvoa2Yo2uXxn6SYgZ5C3fHNB");

pub mod error;
pub mod state;
pub mod instructions;
pub mod utils;

use instructions::*;

#[program]
pub mod token_transfer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn transfer_token(ctx: Context<TransferToken>, params: TransferTokenParams) -> Result<()> {
        instructions::transfer_token::handler(ctx, params)
    }

    pub fn transfer_sol(ctx: Context<TransferSol>, params: TransferSolParams) -> Result<()> {
        instructions::transfer_sol::handler(ctx, params)
    }

    pub fn batch_transfer<'info>(ctx: Context<'_, '_, '_, 'info, BatchTransfer<'info>>, params: BatchTransferParams) -> Result<()> {
        instructions::batch_transfer::handler(ctx, params)
    }

    pub fn transfer_with_authority(ctx: Context<TransferWithAuthority>, params: TransferWithAuthorityParams) -> Result<()> {
        instructions::transfer_with_authority::handler(ctx, params)
    }
} 