use anchor_lang::prelude::*;

declare_id!("TknTrfzGGGtAL3YxG8TKvoa2Yo2uXxn6SYgZ5C3fHNB");

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod token_transfer {
    use super::*;

    pub fn initialize(ctx: Context<initialize::Initialize>, params: initialize::InitializeParams) -> Result<()> {
        initialize::handler(ctx, params)
    }

    pub fn transfer_token(ctx: Context<transfer_token::TransferToken>, params: transfer_token::TransferTokenParams) -> Result<()> {
        transfer_token::handler(ctx, params)
    }

    pub fn transfer_sol(ctx: Context<transfer_sol::TransferSol>, params: transfer_sol::TransferSolParams) -> Result<()> {
        transfer_sol::handler(ctx, params)
    }

    pub fn batch_transfer(ctx: Context<batch_transfer::BatchTransfer>, params: batch_transfer::BatchTransferParams) -> Result<()> {
        batch_transfer::handler(ctx, params)
    }

    pub fn transfer_with_authority(ctx: Context<transfer_with_authority::TransferWithAuthority>, params: transfer_with_authority::TransferWithAuthorityParams) -> Result<()> {
        transfer_with_authority::handler(ctx, params)
    }
} 