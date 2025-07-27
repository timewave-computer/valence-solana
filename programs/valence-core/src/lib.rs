#![allow(deprecated)]
use anchor_lang::prelude::*;

// ================================
// Module Declarations
// ================================

pub mod errors;
pub mod compute_budget;
pub mod guards;
pub mod instructions;
pub mod operations;
pub mod state;
pub mod validation;
pub mod tests;

// Module expected by Anchor's #[program] macro
#[doc(hidden)]
pub mod __client_accounts_crate;


// ================================
// Public API Exports
// ================================

pub use errors::*;
pub use compute_budget::*;
pub use guards::*;
pub use operations::*;
pub use state::*;

// Re-export all instruction items at crate root for Anchor's macro
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

// ================================
// Program Constants
// ================================

/// PDA seed for session accounts
pub const SESSION_SEED: &[u8] = b"session";

/// Maximum operation size to prevent DoS attacks
/// @deprecated Use validation module constants instead
pub const MAX_OPERATION_SIZE: usize = 1024;

// ================================
// Program ID Declaration
// ================================

declare_id!("Va1ence111111111111111111111111111111111111");

// ================================
// Program Instruction Handlers
// ================================

#[program]
pub mod valence_core {
    use super::*;
    use crate::instructions;

    /// Initialize the valence-core program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }
    
    pub fn create_guard_data(
        ctx: Context<CreateGuardData>,
        session: Pubkey,
        compiled_guard: CompiledGuard,
    ) -> Result<()> {
        instructions::create_guard_data(ctx, session, compiled_guard)
    }
    
    pub fn create_session(
        ctx: Context<CreateSession>,
        protocol: Pubkey,
        params: CreateSessionParams,
    ) -> Result<()> {
        instructions::create_session(ctx, protocol, params)
    }
    
    pub fn execute_operations(
        ctx: Context<ExecuteOperations>,
        batch: OperationBatch,
    ) -> Result<()> {
        instructions::execute_operations(ctx, batch)
    }
    
    pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
        instructions::initialize_allowlist(ctx)
    }
    
    pub fn add_to_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::add_to_allowlist(ctx, program_id)
    }
    
    pub fn remove_from_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::remove_from_allowlist(ctx, program_id)
    }
}

// ================================
// Type Aliases
// ================================

// Create type aliases to avoid naming conflicts
pub type ValenceResult<T> = Result<T>;
pub type ValenceError = errors::ValenceError;

