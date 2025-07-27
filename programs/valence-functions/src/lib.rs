#![allow(unexpected_cfgs)]
#![allow(deprecated)]

// Valence Functions - Protocol-specific guard implementations and state management
// Provides reusable components for building protocol-specific authorization logic
use anchor_lang::prelude::*;

// ================================
// Program Declaration
// ================================

declare_id!("Va1enceFunc11111111111111111111111111111111");

// ================================
// Core Protocol Trait System
// ================================

/// Core trait for protocol definitions with embedded constants
/// Enables protocols to define their identity, versioning, and upgrade paths
pub trait Protocol: Clone {
    /// Unique identifier for this protocol
    /// Should be derived from protocol name and core characteristics
    fn id(&self) -> [u8; 32];

    /// Protocol version for upgrades and compatibility tracking
    /// Follows semantic versioning principles
    fn version(&self) -> u16;

    /// Human-readable name for debugging and identification
    /// Should be concise and descriptive
    fn name(&self) -> &'static str;
}

// ================================
// Protocol Upgrade System
// ================================

/// Trait for implementing safe protocol upgrades
/// Ensures data migration and compatibility validation during protocol evolution
pub trait ProtocolUpgrade<T> {
    /// Check if upgrade is allowed from old version
    /// Validates compatibility and prerequisites for migration
    fn can_upgrade_from(&self, old_version: u16, migration_data: &[u8]) -> Result<bool>;

    /// Migrate data from old session format to new format
    /// Handles data transformation during protocol upgrades
    fn migrate_from(&self, old_session_data: &[u8], migration_data: &[u8]) -> Result<Vec<u8>>;

    /// Validate that migration was successful
    /// Ensures data integrity after upgrade completion
    fn validate_migration(&self, old_data: &[u8], new_data: &[u8]) -> Result<bool>;
}

// ================================
// Runtime Environment Context
// ================================

/// Runtime context provided to all protocol functions
/// Contains essential blockchain state and caller information for guard evaluation
#[derive(Clone, Debug, Default)]
pub struct Environment {
    /// Identity of the current operation caller
    /// Used for permission checks and ownership validation
    pub caller: Pubkey,

    /// Current unix timestamp from the blockchain
    /// Essential for time-based guards and expiration logic
    pub timestamp: i64,

    /// Current slot number for block-based operations
    /// Useful for slot-based randomness and timing
    pub slot: u64,

    /// Recent blockhash for deterministic randomness
    /// Provides entropy for cryptographic operations
    pub recent_blockhash: [u8; 32],
}

impl Environment {
    /// Create a new environment from current blockchain state
    /// Convenience constructor for common use cases
    pub fn from_accounts(caller: Pubkey, clock: &Clock, recent_blockhash: [u8; 32]) -> Self {
        Self {
            caller,
            timestamp: clock.unix_timestamp,
            slot: clock.slot,
            recent_blockhash,
        }
    }

    /// Check if the environment represents a valid blockchain state
    /// Validates that timestamps and slots are reasonable
    pub fn is_valid(&self) -> bool {
        // Basic sanity checks
        self.timestamp > 0 && self.slot > 0
    }
}

// ================================
// Module Declarations
// ================================

/// Simplified escrow functionality
pub mod escrow;
/// Business logic functions
pub mod functions;
/// Guard implementations for authorization
pub mod guards;
/// State definitions for protocol-specific data structures
pub mod states;
/// Utility functions and helpers
pub mod utils;

// ================================
// Public API Re-exports
// ================================

/// Re-export all escrow functionality
#[allow(ambiguous_glob_reexports)]
pub use escrow::*;
/// Re-export function system components
pub use functions::{
    core as function_core,
    composition::*,
    common::*,
};
/// Re-export guard implementations  
pub use guards::{
    core as guard_core,
    escrow::*,
    time::*,
    multisig::*,
    state_machine::*,
};
/// Re-export state definitions
#[allow(ambiguous_glob_reexports)]
pub use states::*;
/// Re-export utilities
pub use utils::*;

