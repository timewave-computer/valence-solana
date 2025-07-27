// Structured operation types for session-based execution
use crate::validation;
use anchor_lang::prelude::*;

// Access mode constants
pub const MODE_READ: u8 = 1;
pub const MODE_WRITE: u8 = 2;
pub const MODE_READ_WRITE: u8 = 3;

/// Structured operations that can be executed through sessions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SessionOperation {
    /// Borrow an account for use in this session
    BorrowAccount {
        /// Account to borrow
        account: Pubkey,
        /// Access mode (1 = read, 2 = write, 3 = read+write)
        mode: u8,
    },
    
    /// Release a borrowed account
    ReleaseAccount {
        /// Account to release
        account: Pubkey,
    },
    
    /// Execute a CPI to another program
    InvokeProgram {
        /// Index into the program manifest
        manifest_index: u8,
        /// Instruction data
        data: Vec<u8>,
        /// Account indices from borrowed accounts
        account_indices: Vec<u8>,
    },
    
    /// Update session metadata
    UpdateMetadata {
        /// New metadata
        metadata: [u8; 64],
    },
    
    /// Custom operation for protocol extensions
    /// Performs a CPI to the specified program with the discriminator and data.
    /// The target program receives:
    /// - Account 0: The session account (read-only)
    /// - Accounts 1+: All remaining_accounts from the instruction
    /// - Data: discriminator (8 bytes) + custom data
    Custom {
        /// Program to dispatch custom operation to (must be in CPI allowlist)
        program_id: Pubkey,
        /// Operation discriminator (8-byte anchor discriminator)
        discriminator: [u8; 8],
        /// Operation data (appended after discriminator)
        data: Vec<u8>,
    },
}

impl SessionOperation {
    /// Validate operation data sizes
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::InvokeProgram { data, account_indices, .. } => {
                validation::validate_cpi_data(data)?;
                validation::validate_account_indices(account_indices, 7)?; // Max 8 accounts (0-7)
            }
            Self::Custom { data, .. } => {
                validation::validate_custom_data(data)?;
            }
            _ => {} // Other operations have fixed-size data
        }
        Ok(())
    }
    
    /// Get the required signer for this operation
    pub fn required_signer(&self) -> Option<Pubkey> {
        None // All operations use session authorization
    }
    
    /// Check if this operation requires write access
    pub fn requires_write(&self) -> bool {
        !matches!(self, Self::InvokeProgram { .. }) // All except InvokeProgram need write
    }
    
    /// Estimate compute units for this operation
    pub fn compute_estimate(&self) -> u64 {
        match self {
            Self::BorrowAccount { .. } => 5_000,
            Self::ReleaseAccount { .. } => 3_000,
            Self::InvokeProgram { .. } => 50_000,
            Self::UpdateMetadata { .. } => 2_000,
            Self::Custom { .. } => 10_000,
        }
    }
}

/// Represents a program that can be invoked by operations
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct ProgramManifestEntry {
    /// Program ID
    pub program_id: Pubkey,
}

/// Batch of operations to execute atomically
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OperationBatch {
    /// Operations to execute
    pub operations: Vec<SessionOperation>,
    /// Whether to auto-release all accounts at end
    pub auto_release: bool,
    /// Manifest of programs that can be invoked by operations
    pub program_manifest: Vec<ProgramManifestEntry>,
}

impl OperationBatch {
    /// Maximum number of operations in a batch
    pub const MAX_OPERATIONS: usize = 16;
    
    /// Maximum number of programs in manifest
    pub const MAX_MANIFEST_SIZE: usize = 8;
    
    /// Create a new operation batch
    pub fn new(operations: Vec<SessionOperation>, program_manifest: Vec<ProgramManifestEntry>) -> Self {
        Self {
            operations,
            auto_release: true,
            program_manifest,
        }
    }
    
    /// Validate the entire batch
    pub fn validate(&self) -> Result<()> {
        // Validate batch size
        validation::validate_vec_count(&self.operations, Self::MAX_OPERATIONS)?;
        validation::validate_vec_count(&self.program_manifest, Self::MAX_MANIFEST_SIZE)?;
        
        // Validate each operation
        for op in &self.operations {
            op.validate()?;
        }
        
        Ok(())
    }
    
    /// Estimate total compute units
    pub fn compute_estimate(&self) -> u64 {
        self.operations
            .iter()
            .map(|op| op.compute_estimate())
            .sum()
    }
    
    /// Check if batch requires write access
    pub fn requires_write(&self) -> bool {
        self.operations.iter().any(|op| op.requires_write())
    }
}