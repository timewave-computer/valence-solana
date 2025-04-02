use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod error;
pub mod state;
pub mod instructions;
pub mod cpi;
pub mod instruction;

// Re-export the InitializeParams for external use
pub use cpi::InitializeParams;

// Define program for CPI
anchor_lang::declare_program!(
    crate::base_account,
    BaseAccountProgram,
    "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
);

#[program]
mod base_account {
    use super::*;
    use crate::instructions::{
        initialize,
        approve_library,
        revoke_library,
        create_token_account,
        close_token_account,
        transfer_ownership,
        create_approval_nonce,
        execute_instruction,
    };

    pub fn initialize(
        ctx: Context<initialize::Initialize>,
        max_libraries: u8,
        max_token_accounts: u8,
    ) -> Result<()> {
        initialize::handler(ctx, max_libraries, max_token_accounts)
    }

    pub fn approve_library(
        ctx: Context<approve_library::ApproveLibrary>,
    ) -> Result<()> {
        approve_library::handler(ctx)
    }

    pub fn revoke_library(
        ctx: Context<revoke_library::RevokeLibrary>,
    ) -> Result<()> {
        revoke_library::handler(ctx)
    }

    pub fn create_token_account(
        ctx: Context<create_token_account::CreateTokenAccount>,
    ) -> Result<()> {
        create_token_account::handler(ctx)
    }

    pub fn close_token_account(
        ctx: Context<close_token_account::CloseTokenAccount>,
    ) -> Result<()> {
        close_token_account::handler(ctx)
    }
    
    pub fn transfer_ownership(
        ctx: Context<transfer_ownership::TransferOwnership>,
    ) -> Result<()> {
        transfer_ownership::handler(ctx)
    }

    pub fn create_approval_nonce(
        ctx: Context<create_approval_nonce::CreateApprovalNonce>,
        expiration: i64,
    ) -> Result<()> {
        create_approval_nonce::handler(ctx, expiration)
    }

    pub fn execute_instruction(
        ctx: Context<execute_instruction::ExecuteInstruction>,
        ix_data: Vec<u8>,
    ) -> Result<()> {
        execute_instruction::handler(ctx, ix_data)
    }
}
