#![allow(deprecated)]
use anchor_lang::prelude::*;

// ================================
// Module Declarations
// ================================

pub mod errors;
pub mod compute_meter;
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
pub use compute_meter::*;
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
pub const SESSION_ACCOUNT_SEED: &[u8] = b"session";

/// Session seed (alias for backwards compatibility)
pub const SESSION_SEED: &[u8] = SESSION_ACCOUNT_SEED;

/// Maximum operation size to prevent DoS attacks
/// @deprecated Use validation module constants instead
pub const MAX_SESSION_OPERATION_DATA_SIZE: usize = 1024;

// ================================
// Program ID Declaration
// ================================

declare_id!("Va1ence111111111111111111111111111111111111");

// Make ID accessible for tests
pub const PROGRAM_ID: Pubkey = ID;

// ================================
// Program Instruction Handlers
// ================================

#[program]
pub mod valence_kernel {
    use super::*;
    use crate::instructions;

    /// Initialize the valence-kernel program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }
    
    pub fn create_guard_data(
        ctx: Context<CreateGuardData>,
        session: Pubkey,
        serialized_guard: SerializedGuard,
    ) -> Result<()> {
        instructions::create_guard_data(ctx, session, serialized_guard)
    }
    
    pub fn create_session_account(
        ctx: Context<CreateSession>,
        shard: Pubkey,
        params: CreateSessionParams,
    ) -> Result<()> {
        instructions::create_session_account(ctx, shard, params)
    }
    
    pub fn execute_session_operations(
        ctx: Context<ExecuteOperations>,
        batch: KernelOperationBatch,
    ) -> Result<()> {
        instructions::execute_session_operations(ctx, batch)
    }
    
    pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
        instructions::initialize_allowlist(ctx)
    }
    
    pub fn add_program_to_cpi_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::add_program_to_cpi_allowlist(ctx, program_id)
    }
    
    pub fn remove_program_from_cpi_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::remove_program_from_cpi_allowlist(ctx, program_id)
    }
    
    pub fn execute_with_guard(
        ctx: Context<ExecuteWithGuard>,
        operation: Vec<u8>
    ) -> Result<()> {
        instructions::execute_with_guard(ctx, operation)
    }
}

// ================================
// Type Aliases
// ================================

// Create type aliases to avoid naming conflicts
pub type ValenceResult<T> = Result<T>;
pub type KernelError = errors::KernelError;

// Backwards compatibility aliases
pub type SessionScope = state::SessionContextScope;
pub type BorrowedAccount = state::SessionBorrowedAccount;
pub type OperationBatch = operations::KernelOperationBatch;
pub type SessionOperation = operations::KernelOperation;
pub type ProgramManifestEntry = operations::CpiProgramEntry;
pub type CPIManifestEntry = guards::CpiCallEntry;

