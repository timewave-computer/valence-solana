use anchor_lang::prelude::*;

declare_id!("SngUse1uMxJR2VbJQw7zYSQBVUBPRYStCgSZxSxQVyy");

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod single_use_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn execute(ctx: Context<Execute>, params: ExecuteParams) -> Result<()> {
        instructions::execute::handler(ctx, params)
    }

    pub fn emergency_recover(ctx: Context<EmergencyRecover>) -> Result<()> {
        instructions::emergency_recover::handler(ctx)
    }

    // Base Account Program passthrough instructions
    pub fn register_library(ctx: Context<RegisterLibrary>, params: RegisterLibraryParams) -> Result<()> {
        instructions::register_library::handler(ctx, params)
    }

    pub fn approve_library(ctx: Context<ApproveLibrary>, library: Pubkey) -> Result<()> {
        instructions::approve_library::handler(ctx, library)
    }

    pub fn create_token_account(ctx: Context<CreateTokenAccount>, mint: Pubkey) -> Result<()> {
        instructions::create_token_account::handler(ctx, mint)
    }

    pub fn transfer_tokens(ctx: Context<TransferTokens>, params: TransferTokensParams) -> Result<()> {
        instructions::transfer_tokens::handler(ctx, params)
    }
} 