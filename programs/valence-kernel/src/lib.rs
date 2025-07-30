#![allow(deprecated, unexpected_cfgs)]
use anchor_lang::prelude::*;

// ================================
// Module Declarations
// ================================

pub mod errors;
pub mod meter;
pub mod instructions;
pub mod namespace;
pub mod state;
pub mod validation;
pub mod bump_allocator;

// Module expected by Anchor's #[program] macro
#[doc(hidden)]
pub mod __client_accounts_crate;


// ================================
// Public API Exports
// ================================

pub use meter::*;
pub use namespace::*;
pub use state::*;
// Re-export operation types from batch_operations
pub use instructions::batch_operations::{
    KernelOperation, OperationBatch, TimestampCheck,
    ACCESS_MODE_READ, ACCESS_MODE_WRITE, ACCESS_MODE_READ_WRITE,
};

// Re-export all instruction items at crate root for Anchor's macro
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

// ================================
// Program Constants
// ================================

/// PDA seed for session accounts
pub const SESSION_ACCOUNT_SEED: &[u8] = b"session";

// ================================
// Capacity Constants
// ================================
// These constants define the maximum capacities for various kernel structures.
// They are carefully tuned to balance functionality with Solana's stack constraints.
// 
// IMPORTANT: Increasing these values may cause stack overflow errors during
// program execution. The current values ensure the program fits within Solana's
// 4KB stack limit while providing reasonable capacity for most use cases.
//
// If you need larger capacities, consider:
// 1. Deploying multiple sessions for parallel operations
// 2. Using multiple transactions for large batches
// 3. Implementing pagination patterns in your application

/// Maximum number of accounts that can be registered per category in SessionAccountLookup
/// (borrowable, programs, guards). Total capacity = 3 * MAX_REGISTERED_ACCOUNTS
pub const MAX_REGISTERED_ACCOUNTS: usize = 8;

/// Maximum number of accounts that can be referenced in a single batch operation
pub const MAX_BATCH_ACCOUNTS: usize = 12;

/// Maximum number of operations that can be executed in a single batch
pub const MAX_BATCH_OPERATIONS: usize = 5;

/// Maximum size of data payload for function calls and raw CPI operations
pub const MAX_OPERATION_DATA_SIZE: usize = 64;

/// Maximum number of account indices that can be passed to a CPI call
pub const MAX_CPI_ACCOUNT_INDICES: usize = 12;


// ================================
// Program ID Declaration
// ================================

declare_id!("Va1ence111111111111111111111111111111111111");

// Make ID accessible for tests
pub const PROGRAM_ID: Pubkey = ID;

// ================================
// Program Instruction Handlers
// ================================

#[allow(deprecated, unexpected_cfgs)]
#[program]
pub mod valence_kernel {
    use super::*;
    use crate::instructions;

    /// Initialize the valence-kernel shard program, set up global state for operation
    pub fn initialize_shard(ctx: Context<InitializeShard>) -> Result<()> {
        instructions::initialize_shard(ctx)
    }
    
    /// Creates minimal guard account for security policy configuration
    pub fn create_guard_account(
        ctx: Context<CreateGuardAccount>,
        session: Pubkey,
        allow_unregistered_cpi: bool,
    ) -> Result<()> {
        instructions::create_guard_account(ctx, session, allow_unregistered_cpi)
    }
    
    /// Establishes authorized execution context with initial registrations
    pub fn create_session_account(
        ctx: Context<CreateSession>,
        shard: Pubkey,
        params: CreateSessionParams,
        initial_borrowable: Vec<RegisteredAccount>,
        initial_programs: Vec<RegisteredProgram>,
    ) -> Result<()> {
        instructions::create_session_account(ctx, shard, params, &initial_borrowable, &initial_programs)
    }
    
    /// Unified account lookup table management
    pub fn manage_alt(
        ctx: Context<ManageALT>,
        add_borrowable: Vec<RegisteredAccount>,
        add_programs: Vec<RegisteredProgram>,
        remove_accounts: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::manage_alt(ctx, &add_borrowable, &add_programs, &remove_accounts)
    }
    
    /// Invalidate a session for move semantics
    pub fn invalidate_session(ctx: Context<InvalidateSession>) -> Result<()> {
        instructions::invalidate_session(ctx)
    }
    
    /// Execute a batch of operations using the on-chain linker
    pub fn execute_batch(
        ctx: Context<ExecuteBatch>,
        batch: OperationBatch,
    ) -> Result<()> {
        instructions::execute_batch(ctx, batch)
    }
    
    /// Create a child account within the session's namespace
    pub fn create_child_account(
        ctx: Context<CreateChildAccount>,
        namespace_suffix: String,
        initial_lamports: u64,
        space: u64,
        owner_program: Pubkey,
    ) -> Result<()> {
        instructions::create_child_account(ctx, namespace_suffix, initial_lamports, space, owner_program)
    }
    
    /// Close a child account and recover lamports
    pub fn close_child_account(ctx: Context<CloseChildAccount>) -> Result<()> {
        instructions::close_child_account(ctx)
    }
    
    /// Creates global registry of permitted CPI target programs
    pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
        instructions::initialize_allowlist(ctx)
    }
    
    /// Grants permission for sessions to invoke specified program
    pub fn add_program_to_cpi_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::add_program_to_cpi_allowlist(ctx, program_id)
    }
    
    /// Revokes permission to invoke specified program
    pub fn remove_program_from_cpi_allowlist(
        ctx: Context<ManageAllowlist>,
        program_id: Pubkey
    ) -> Result<()> {
        instructions::remove_program_from_cpi_allowlist(ctx, program_id)
    }
    
    
    // ===== DEDICATED HIGH-PERFORMANCE INSTRUCTIONS =====
    
    /// Optimized SPL token transfer
    pub fn spl_transfer(
        ctx: Context<SplTransfer>,
        amount: u64,
    ) -> Result<()> {
        instructions::spl_transfer(ctx, amount)
    }
    
}

// ================================
// Type Exports
// ================================

// Type exports
pub use crate::errors::KernelError;


