// Guard system for authorization and access control in valence-kernel
// This module defines the on-chain virtual machine for executing guard logic.

use anchor_lang::prelude::*;

pub mod opcodes;
pub mod evaluator;
pub mod serializer;

pub use opcodes::*;
pub use evaluator::*;
pub use serializer::*;

// ================================
// External Guard Interface
// ================================

/// Input data for external guard CPI
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExternalGuardInput {
    /// The session being evaluated
    pub session: Pubkey,
    /// The caller requesting access
    pub caller: Pubkey,
    /// Current timestamp
    pub timestamp: i64,
    /// Operation context (custom data)
    pub operation: Vec<u8>,
}

/// Output data from external guard CPI
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExternalGuardOutput {
    /// Whether the guard passed
    pub passed: bool,
}

// ================================
// Required Account Definition
// ================================

/// Defines an account required by an external guard
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CpiAccountRequirement {
    /// Expected public key of the account
    pub pubkey: Pubkey,
    /// Whether the account should be writable
    pub is_writable: bool,
    /// Whether the account should be a signer
    pub is_signer: bool,
    /// Role description for documentation
    pub role: CpiAccountRole,
}

/// Role of an account in guard execution
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum CpiAccountRole {
    /// Oracle providing price or other data
    Oracle,
    /// Token mint
    TokenMint,
    /// Token account
    TokenAccount,
    /// Authority account
    Authority,
    /// State account
    State,
    /// Custom role with description
    Custom([u8; 32]),
}

// ================================
// Guard System Constants
// ================================

/// Maximum nesting depth for composite guards to prevent stack overflow
pub const MAX_GUARD_COMPILER_DEPTH: u8 = 3;

/// Maximum size for external guard data to prevent DoS
pub const MAX_CPI_DATA_SIZE: usize = 256;

/// Maximum number of accounts an external guard can require
pub const MAX_CPI_ACCOUNTS: usize = 8;

// ================================
// Core Guard Enum
// ================================

/// Unified guard system for all authorization logic
#[derive(Clone, Debug)]
pub enum Guard {
    /// Always allows access (useful for testing)
    AlwaysTrue,
    /// Always denies access (useful for pausing)
    AlwaysFalse,
    /// Only allows the session owner to access
    OwnerOnly,
    /// Time-based access control with expiration timestamp
    Expiration { expires_at: i64 },
    /// Permission-based access requiring specific authorization level
    Permission { required: u64 },
    /// Account-based permission check (more secure than global Permission)
    PermissionAccount { account: Pubkey, required_level: u64 },
    /// Usage limit guard for rate limiting
    UsageLimit { max: u64 },
    /// Logical AND - both guards must pass
    And(Box<Guard>, Box<Guard>),
    /// Logical OR - either guard can pass
    Or(Box<Guard>, Box<Guard>),
    /// Logical NOT - inverts the guard result
    Not(Box<Guard>),
    /// External guard implemented by another program
    External {
        /// Program ID of the external guard implementation
        program: Pubkey,
        /// Custom data to pass to the guard
        data: Vec<u8>,
        /// Required accounts for secure execution
        required_accounts: Vec<CpiAccountRequirement>,
    },
}

impl Guard {
    /// Validate guard data to prevent DoS attacks
    pub fn validate(&self) -> Result<()> {
        match self {
            Guard::External { data, required_accounts, .. } => {
                if data.len() > MAX_CPI_DATA_SIZE {
                    return Err(crate::errors::KernelError::GuardDataTooLarge.into());
                }
                if required_accounts.len() > MAX_CPI_ACCOUNTS {
                    return Err(crate::errors::KernelError::GuardDataTooLarge.into());
                }
            }
            Guard::And(a, b) | Guard::Or(a, b) => {
                a.validate()?;
                b.validate()?;
            }
            Guard::Not(g) => {
                g.validate()?;
            }
            _ => {} // Simple guards are always valid
        }
        Ok(())
    }

    /// Check if this guard requires external evaluation
    pub fn requires_external(&self) -> bool {
        match self {
            Guard::External { .. } => true,
            Guard::PermissionAccount { .. } => true,
            Guard::And(a, b) => a.requires_external() || b.requires_external(),
            Guard::Or(a, b) => a.requires_external() || b.requires_external(),
            Guard::Not(g) => g.requires_external(),
            _ => false,
        }
    }
}