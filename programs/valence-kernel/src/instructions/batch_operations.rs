// Primary batch execution engine for valence-kernel operation processing
//
// This module implements the core "on-chain linker" execution model that enables
// clients to submit complex operation batches without managing low-level account
// details. The batch system processes atomic sequences of account borrowing, CPI
// calls, and state transitions while maintaining security through comprehensive
// validation and guard evaluation.
//
// EXECUTION MODEL: Clients provide flat account lists and operation sequences with
// indices into those accounts. The kernel handles account linking, permission
// validation, guard evaluation, and atomic execution, abstracting away Solana's
// complex account model while maintaining security and performance.
//
// SECURITY INTEGRATION: All operations flow through validation layers, guard
// evaluations, and allowlist checks before execution. The batch engine ensures
// atomic success/failure across entire operation sequences and prevents partial
// state corruption through comprehensive rollback mechanisms.
//
// PERFORMANCE OPTIMIZATION: The linker model eliminates remaining_accounts patterns
// and reduces transaction size through index-based account references. Batch
// processing amortizes validation costs across multiple operations.

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use crate::{
    errors::KernelError,
    validation,
    state::{Session, GuardAccount, AllowlistAccount, SessionAccountLookup},
    namespace::NamespacePath,
    MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS, MAX_OPERATION_DATA_SIZE, MAX_CPI_ACCOUNT_INDICES,
};

// ================================
// Execution Context
// ================================

/// Enhanced execution context with full transaction metadata
#[derive(Clone, Debug)]
pub struct ExecutionContext {
    // Transaction metadata
    pub slot: u64,
    pub epoch: u64,
    pub tx_submitter: Pubkey,
    
    // Session context
    pub session: Pubkey,
    pub namespace: NamespacePath,
    pub caller: Pubkey,
    pub timestamp: i64,
}

// ================================
// Core Operation Types
// ================================

/// Timestamp comparison types for assertions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampCheck {
    Before,
    After,
    Equal,
}

/// Operation enum for async/dynamic use cases
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum KernelOperation {
    // ===== ACCOUNT MANAGEMENT =====
    
    /// Borrow an account for session use
    BorrowAccount {
        account_index: u8,
        mode: u8, // ACCESS_MODE_READ, WRITE, or READ_WRITE
    },
    
    /// Release a previously borrowed account
    ReleaseAccount {
        account_index: u8,
    },
    

    // ===== REGISTERED FUNCTION CPI =====
    
    /// Call a registered function via the on-chain registry
    CallRegisteredFunction {
        /// Registry ID of the function to call
        registry_id: u64,
        /// Fixed-size array of account indices for CPI
        account_indices: [u8; MAX_CPI_ACCOUNT_INDICES],
        /// Number of actual account indices
        account_indices_len: u8,
        /// Fixed-size function-specific data
        data: [u8; MAX_OPERATION_DATA_SIZE],
        /// Actual data length
        data_len: u16,
    },

    // ===== UNSAFE RAW CPI =====
    
    /// Raw CPI to arbitrary program (requires `allow_unregistered_cpi` flag)
    UnsafeRawCpi {
        /// Index of the program ID in accounts array
        program_index: u8,
        /// Fixed-size array of account indices for CPI
        account_indices: [u8; MAX_CPI_ACCOUNT_INDICES],
        /// Number of actual account indices
        account_indices_len: u8,
        /// Fixed-size raw instruction data
        data: [u8; MAX_OPERATION_DATA_SIZE],
        /// Actual data length
        data_len: u16,
    },
}

impl KernelOperation {
    /// Validate operation parameters
    /// 
    /// # Errors
    /// Returns validation errors for invalid parameters
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::BorrowAccount { mode, .. } => {
                require!(
                    *mode == ACCESS_MODE_READ || 
                    *mode == ACCESS_MODE_WRITE || 
                    *mode == ACCESS_MODE_READ_WRITE,
                    KernelError::InvalidParameters
                );
            }
            
            
            Self::CallRegisteredFunction { account_indices, account_indices_len, data, data_len, .. } |
            Self::UnsafeRawCpi { account_indices, account_indices_len, data, data_len, .. } => {
                let indices_slice = &account_indices[..*account_indices_len as usize];
                validation::validate_account_indices(indices_slice, 255)?;
                let data_slice = &data[..*data_len as usize];
                validation::validate_cpi_data(data_slice)?;
            }
            
            Self::ReleaseAccount{ .. } => {} // Other operations have no variable parameters to validate
        }
        Ok(())
    }
    
    /// Check if operation requires write access to session
    #[must_use]
    pub const fn requires_session_write(&self) -> bool {
        matches!(self, 
            Self::BorrowAccount { .. } |
            Self::ReleaseAccount { .. }
        )
    }
    
    /// Get estimated compute units for this operation
    #[must_use]
    pub const fn compute_estimate(&self) -> u64 {
        match self {
            // Account management
            Self::BorrowAccount { .. } => 3_000,
            Self::ReleaseAccount { .. } => 2_000,
            
            // CPI operations are expensive
            Self::CallRegisteredFunction { .. } | Self::UnsafeRawCpi { .. } => 50_000,
        }
    }
}

// ================================
// Operation Batch
// ================================


/// A batch of operations to execute with account linking
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OperationBatch {
    /// Fixed-size array of accounts involved
    pub accounts: [Pubkey; MAX_BATCH_ACCOUNTS],
    /// Number of actual accounts
    pub accounts_len: u8,
    /// Fixed-size array of operations to execute
    pub operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS],
    /// Number of actual operations
    pub operations_len: u8,
}

impl OperationBatch {
    /// Validate the batch
    /// 
    /// # Errors
    /// Returns validation errors for invalid batch parameters
    pub fn validate(&self) -> Result<()> {
        // Validate account list size
        require!(
            self.accounts_len > 0 && self.accounts_len as usize <= MAX_BATCH_ACCOUNTS,
            KernelError::InvalidParameters
        );
        
        // Validate operations
        require!(
            self.operations_len > 0 && self.operations_len as usize <= MAX_BATCH_OPERATIONS,
            KernelError::InvalidParameters
        );
        
        // Validate each operation
        for i in 0..self.operations_len as usize {
            let op = self.operations[i].as_ref()
                .ok_or(KernelError::InvalidParameters)?;
            op.validate()?;
            
            // Validate account indices are within bounds
            match op {
                KernelOperation::BorrowAccount { account_index, .. } |
                KernelOperation::ReleaseAccount { account_index } => {
                    require!(
                        (*account_index as usize) < self.accounts_len as usize,
                        KernelError::InvalidParameters
                    );
                }
                
                
                KernelOperation::UnsafeRawCpi { program_index, account_indices, account_indices_len, .. } => {
                    require!(
                        (*program_index as usize) < self.accounts_len as usize,
                        KernelError::InvalidParameters
                    );
                    for &account_index in account_indices.iter().take(*account_indices_len as usize) {
                        require!(
                            (account_index as usize) < self.accounts_len as usize,
                            KernelError::InvalidParameters
                        );
                    }
                }
                
                KernelOperation::CallRegisteredFunction { account_indices, account_indices_len, .. } => {
                    for &account_index in account_indices.iter().take(*account_indices_len as usize) {
                        require!(
                            (account_index as usize) < self.accounts_len as usize,
                            KernelError::InvalidParameters
                        );
                    }
                }
                
            }
        }
        
        Ok(())
    }
    
    /// Estimate total compute units
    #[must_use]
    pub fn compute_estimate(&self) -> u64 {
        let mut total = 0u64;
        for i in 0..self.operations_len as usize {
            if let Some(op) = &self.operations[i] {
                total += op.compute_estimate();
            }
        }
        total
    }
}

// ================================
// Constants
// ================================

pub const ACCESS_MODE_READ: u8 = 1;
pub const ACCESS_MODE_WRITE: u8 = 2;
pub const ACCESS_MODE_READ_WRITE: u8 = 3;

// ================================
// Batch Execution Handler
// ================================

/// Execute a batch of operations using the on-chain linker model
/// 
/// This is the batch execution interface for async/dynamic use cases.
/// Clients provide:
/// 1. A flat list of account pubkeys
/// 2. A list of operations with indices into that account list
/// 
/// The kernel handles all linking, validation, and execution.
/// Execute a batch of kernel operations
/// 
/// # Errors
/// Returns execution errors for invalid operations or failed validation
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
pub fn execute_batch(
    ctx: Context<ExecuteBatch>,
    batch: OperationBatch,
) -> Result<()> {
    // Validate the batch
    batch.validate()?;
    
    // Extract key accounts and session key first to avoid borrow checker issues
    let session_key = ctx.accounts.session.key();
    let session = &mut ctx.accounts.session;
    let guard_account = &ctx.accounts.guard_account;
    let cpi_allowlist = &ctx.accounts.cpi_allowlist;
    let alt = &ctx.accounts.account_lookup;
    let clock = &ctx.accounts.clock;
    let caller = ctx.accounts.caller.key();
    
    // Verify relationships
    require!(
        guard_account.session == session_key,
        KernelError::InvalidSessionConfig
    );
    require!(
        alt.session == session_key,
        KernelError::InvalidSessionConfig
    );
    
    // Create execution context with full transaction metadata
    let _execution_ctx = ExecutionContext {
        // Transaction metadata
        slot: clock.slot,
        epoch: clock.epoch,
        tx_submitter: ctx.accounts.tx_submitter.key(),
        
        // Session context
        session: session_key,
        namespace: session.namespace.clone(),
        caller,
        timestamp: clock.unix_timestamp,
    };
    
    // Authorization checks
    require!(
        caller == session.owner,
        KernelError::Unauthorized
    );
    
    // Ensure session is still active
    require!(
        session.active,
        KernelError::SessionInactive
    );
    
    // Process each operation
    for i in 0..batch.operations_len as usize {
        let operation = batch.operations[i].as_ref()
            .ok_or(KernelError::InvalidParameters)?;
        match operation {
            KernelOperation::BorrowAccount { account_index, mode } => {
                require!(
                    (*account_index as usize) < batch.accounts_len as usize,
                    KernelError::InvalidParameters
                );
                let account = &batch.accounts[*account_index as usize];
                
                // Check if account is a child account of this session
                let is_child_account = session.is_child_account(account);
                
                // Verify account is either registered in ALT or is a child account
                if is_child_account {
                    // For child accounts, we allow borrowing with the requested mode
                    // as long as the session created them
                    msg!("Allowing borrow of child account {}", account);
                } else {
                    alt.validate_borrowable(account, *mode)?;
                }
                
                // Borrow the account
                session.borrow_account(*account, *mode, clock)?;
                
                msg!("Borrowed account {} with mode {}", account, mode);
            }
            
            KernelOperation::ReleaseAccount { account_index } => {
                require!(
                    (*account_index as usize) < batch.accounts_len as usize,
                    KernelError::InvalidParameters
                );
                let account = &batch.accounts[*account_index as usize];
                
                session.release_account(account)?;
                
                msg!("Released account {}", account);
            }
            
            
            KernelOperation::CallRegisteredFunction { registry_id, account_indices, account_indices_len, data, data_len } => {
                msg!("Calling registered function {} with {} accounts", 
                    registry_id, account_indices_len);
                
                // Look up function in registry
                let function_info = crate::state::function_registry::FunctionInfo::get_registry_entry(*registry_id)
                    .ok_or(KernelError::InvalidParameters)?;
                
                // Verify function is active
                require!(
                    function_info.is_active,
                    KernelError::InvalidParameters
                );
                
                // Verify program is on allowlist
                require!(
                    cpi_allowlist.is_allowed(&function_info.program_id),
                    KernelError::ProgramNotAllowed
                );
                
                // Build account metas for CPI
                let mut account_infos = Vec::with_capacity(*account_indices_len as usize);
                let mut account_metas = Vec::with_capacity(*account_indices_len as usize);
                
                // Collect accounts for CPI
                for &idx in account_indices.iter().take(*account_indices_len as usize) {
                    require!(
                        (idx as usize) < batch.accounts_len as usize,
                        KernelError::InvalidParameters
                    );
                    let account_key = &batch.accounts[idx as usize];
                    
                    // Find this account in remaining_accounts
                    let mut found = false;
                    for remaining_account in ctx.remaining_accounts {
                        if remaining_account.key() == *account_key {
                            account_infos.push(remaining_account.clone());
                            account_metas.push(solana_program::instruction::AccountMeta {
                                pubkey: *account_key,
                                is_signer: remaining_account.is_signer,
                                is_writable: remaining_account.is_writable,
                            });
                            found = true;
                            break;
                        }
                    }
                    require!(found, KernelError::MissingRequiredAccount);
                }
                
                // Increment CPI depth
                session.check_and_increment_cpi_depth()?;
                
                // Build and invoke instruction
                let ix = solana_program::instruction::Instruction {
                    program_id: function_info.program_id,
                    accounts: account_metas,
                    data: data[..*data_len as usize].to_vec(),
                };
                
                solana_program::program::invoke(&ix, &account_infos)?;
                
                // Decrement CPI depth
                session.decrement_cpi_depth();
                
                msg!("Successfully called registered function {}", registry_id);
            }
            
            KernelOperation::UnsafeRawCpi { program_index, account_indices, account_indices_len, data, data_len } => {
                require!(
                    (*program_index as usize) < batch.accounts_len as usize,
                    KernelError::InvalidParameters
                );
                let program_id = &batch.accounts[*program_index as usize];
                
                // CRITICAL SECURITY CHECK
                let is_allowed = cpi_allowlist.is_allowed(program_id);
                
                if !is_allowed {
                    // Program not on allowlist - check developer opt-in
                    require!(
                        guard_account.allow_unregistered_cpi,
                        KernelError::ProgramNotAllowed
                    );
                    
                    msg!("WARNING: Executing unsafe CPI to {} via developer opt-in", program_id);
                }
                
                // Build account metas for CPI
                let mut account_infos = Vec::with_capacity(*account_indices_len as usize);
                let mut account_metas = Vec::with_capacity(*account_indices_len as usize);
                
                // Collect remaining accounts that match our indices
                for &needed_idx in account_indices.iter().take(*account_indices_len as usize) {
                    require!(
                        (needed_idx as usize) < batch.accounts_len as usize,
                        KernelError::InvalidParameters
                    );
                    let needed_key = &batch.accounts[needed_idx as usize];
                    
                    // Find this account in remaining_accounts
                    let mut found = false;
                    for remaining_account in ctx.remaining_accounts {
                        if remaining_account.key() == *needed_key {
                            account_infos.push(remaining_account.clone());
                            account_metas.push(solana_program::instruction::AccountMeta {
                                pubkey: *needed_key,
                                is_signer: remaining_account.is_signer,
                                is_writable: remaining_account.is_writable,
                            });
                            found = true;
                            break;
                        }
                    }
                    require!(found, KernelError::MissingRequiredAccount);
                }
                
                require!(
                    account_infos.len() == *account_indices_len as usize,
                    KernelError::InvalidParameters
                );
                
                // Increment CPI depth
                session.check_and_increment_cpi_depth()?;
                
                // Build and invoke instruction
                let ix = solana_program::instruction::Instruction {
                    program_id: *program_id,
                    accounts: account_metas,
                    data: data[..*data_len as usize].to_vec(),
                };
                
                solana_program::program::invoke(&ix, &account_infos)?;
                
                // Decrement CPI depth
                session.decrement_cpi_depth();
                
                msg!("Executed CPI to {}", program_id);
            }
        }
    }
    
    // Increment usage counter
    session.increment_usage(clock)?;
    
    Ok(())
}

// ================================
// Account Context
// ================================

#[derive(Accounts)]
pub struct ExecuteBatch<'info> {
    /// The session being used for execution
    #[account(mut)]
    pub session: Box<Account<'info, Session>>,
    
    /// The guard configuration for this session
    pub guard_account: Box<Account<'info, GuardAccount>>,
    
    /// The session's account lookup table
    #[account(
        constraint = account_lookup.session == session.key() @ KernelError::InvalidSessionConfig
    )]
    pub account_lookup: Box<Account<'info, SessionAccountLookup>>,
    
    /// Global CPI allowlist for security checks
    pub cpi_allowlist: Box<Account<'info, AllowlistAccount>>,
    
    /// The caller executing the batch
    pub caller: Signer<'info>,
    
    /// Transaction fee payer (who submitted the transaction)
    #[account(mut)]
    pub tx_submitter: Signer<'info>,
    
    /// Clock for timestamp operations
    pub clock: Sysvar<'info, Clock>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
    
    /// Rent sysvar for calculating rent-exempt balances
    pub rent: Sysvar<'info, Rent>,
}