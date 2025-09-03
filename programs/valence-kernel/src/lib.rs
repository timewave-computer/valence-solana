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
/// (borrowable, programs, guards). Reduced for stack optimization.
pub const MAX_REGISTERED_ACCOUNTS: usize = 4;

/// Maximum number of accounts that can be referenced in a single batch operation
pub const MAX_BATCH_ACCOUNTS: usize = 12;

/// Maximum number of operations that can be executed in a single batch
pub const MAX_BATCH_OPERATIONS: usize = 5;

/// Maximum size of data payload for function calls and raw CPI operations
pub const MAX_OPERATION_DATA_SIZE: usize = 64;

/// Maximum number of account indices that can be passed to a CPI call
pub const MAX_CPI_ACCOUNT_INDICES: usize = 12;

/// Maximum direct children per session - aligned with EVM
pub const MAX_DIRECT_CHILDREN: u8 = 8;

/// Maximum depth for cascading invalidation to prevent stack overflow - aligned with EVM
pub const MAX_CASCADE_DEPTH: u8 = 4;

/// Minimum compute units required to continue cascading
pub const MIN_COMPUTE_UNITS_FOR_CASCADE: u64 = 50_000;

/// Maximum sessions to invalidate in a single batch
pub const MAX_BATCH_INVALIDATION_SIZE: usize = 10;


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
        // Limit parameter sizes to prevent stack overflow (use truncated slices)
        let borrowable_slice = if initial_borrowable.len() > MAX_REGISTERED_ACCOUNTS { 
            &initial_borrowable[..MAX_REGISTERED_ACCOUNTS] 
        } else { 
            &initial_borrowable 
        };
        let programs_slice = if initial_programs.len() > MAX_REGISTERED_ACCOUNTS { 
            &initial_programs[..MAX_REGISTERED_ACCOUNTS] 
        } else { 
            &initial_programs 
        };
        
        instructions::create_session_account(ctx, shard, params, borrowable_slice, programs_slice)
    }
    
    /// Unified account lookup table management (fixed for Anchor compatibility)
    pub fn manage_alt(
        ctx: Context<ManageAlt>,
        add_borrowable: Vec<RegisteredAccount>,
        add_programs: Vec<RegisteredProgram>,
        remove_accounts: Vec<Pubkey>,
    ) -> Result<()> {
        // Limit sizes to prevent stack overflow and respect contract constraints
        let borrowable_slice = if add_borrowable.len() > MAX_REGISTERED_ACCOUNTS { 
            &add_borrowable[..MAX_REGISTERED_ACCOUNTS] 
        } else { 
            &add_borrowable 
        };
        let programs_slice = if add_programs.len() > MAX_REGISTERED_ACCOUNTS { 
            &add_programs[..MAX_REGISTERED_ACCOUNTS] 
        } else { 
            &add_programs 
        };
        let remove_slice = if remove_accounts.len() > MAX_REGISTERED_ACCOUNTS { 
            &remove_accounts[..MAX_REGISTERED_ACCOUNTS] 
        } else { 
            &remove_accounts 
        };
        
        instructions::manage_alt(ctx, borrowable_slice, programs_slice, remove_slice)
    }
    
    /// Invalidate a session for move semantics
    pub fn invalidate_session(ctx: Context<InvalidateSession>) -> Result<()> {
        instructions::invalidate_session(ctx)
    }
    
    /// Invalidate multiple sessions in a single batch operation
    pub fn invalidate_session_batch(
        ctx: Context<InvalidateSessionBatch>,
        session_keys: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::invalidate_session_batch(ctx, &session_keys)
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

// Error types
pub use crate::errors::KernelError;

// State types
pub use crate::state::{Session, SessionBorrowedAccount, GuardAccount, SessionAccountLookup};
pub use crate::state::{RegisteredAccount, RegisteredProgram, CreateSessionParams};

// Namespace types
pub use crate::namespace::{NamespacePath, Namespace};


