use anchor_lang::prelude::*;

declare_id!("StrgAcntpYbpLUXMvUb8ZuUjQBnMLFZnBxwxrRjrYJUY");

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod storage_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn set_item(ctx: Context<SetItem>, params: SetItemParams) -> Result<()> {
        instructions::set_item::handler(ctx, params)
    }

    pub fn get_item(ctx: Context<GetItem>, key: String) -> Result<()> {
        instructions::get_item::handler(ctx, key)
    }

    pub fn delete_item(ctx: Context<DeleteItem>, key: String) -> Result<()> {
        instructions::delete_item::handler(ctx, key)
    }

    pub fn batch_update(ctx: Context<BatchUpdate>, params: BatchUpdateParams) -> Result<()> {
        instructions::batch_update::handler(ctx, params)
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

    pub fn execute_instruction(ctx: Context<ExecuteInstruction>, params: ExecuteInstructionParams) -> Result<()> {
        instructions::execute_instruction::handler(ctx, params)
    }

    pub fn transfer_tokens(ctx: Context<TransferTokens>, params: TransferTokensParams) -> Result<()> {
        instructions::transfer_tokens::handler(ctx, params)
    }
} 