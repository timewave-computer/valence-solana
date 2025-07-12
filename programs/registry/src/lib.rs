//! Registry - Global function registry singleton
//! 
//! Stores immutable mapping of function_hash → program_id
//! Functions are purely content-addressed by their hash
//! Once registered, functions cannot be updated - only unregistered

use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

pub mod state;
pub mod instructions;
pub mod error;

pub use state::*;
pub use instructions::*;
pub use error::*;

#[program]
pub mod valence_registry {
    use super::*;

    /// Register a new function
    pub fn register(
        ctx: Context<Register>,
        hash: [u8; 32],
        program: Pubkey,
        required_capabilities: Vec<String>,
    ) -> Result<()> {
        instructions::register(ctx, hash, program, required_capabilities)
    }

    /// Unregister a function
    pub fn unregister(ctx: Context<Unregister>, hash: [u8; 32]) -> Result<()> {
        instructions::unregister(ctx, hash)
    }
}