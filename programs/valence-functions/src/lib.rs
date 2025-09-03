#![allow(unexpected_cfgs)]
#![allow(deprecated)]

// Valence Functions - Simplified function implementations aligned with valence-kernel
// 
// This crate has been simplified to focus on core function definitions that work
// with the valence-kernel's hardcoded registry system.

use anchor_lang::prelude::*;

// ================================
// Program Declaration
// ================================

declare_id!("Va1enceFunc11111111111111111111111111111111");

// ================================
// Simplified Runtime Environment
// ================================

/// Runtime context aligned with valence-kernel's ExecutionContext
#[derive(Clone, Debug, Default)]
pub struct Environment {
    /// Transaction metadata
    pub slot: u64,
    pub epoch: u64,
    pub tx_submitter: Pubkey,
    
    /// Session context
    pub session: Pubkey,
    pub caller: Pubkey,
    pub timestamp: i64,
    
    /// For backward compatibility (will be removed)
    pub recent_blockhash: [u8; 32],
}

impl Environment {
    /// Create environment from kernel's execution context
    pub fn from_kernel_context(
        slot: u64,
        epoch: u64,
        tx_submitter: Pubkey,
        session: Pubkey,
        caller: Pubkey,
        timestamp: i64,
    ) -> Self {
        Self {
            slot,
            epoch,
            tx_submitter,
            session,
            caller,
            timestamp,
            recent_blockhash: [0u8; 32], // Placeholder
        }
    }

    /// Check if the environment represents a valid state
    pub fn is_valid(&self) -> bool {
        self.timestamp > 0 && self.slot > 0
    }
}

// ================================
// Module Declarations (simplified)
// ================================

/// Core function trait and infrastructure
/// Individual function implementations
pub mod functions;

/// State definitions for function data structures
pub mod states;

// Removed: Complex escrow functionality (if not used by kernel)
// Removed: Complex shard trait system (if not used by kernel)

// ================================
// Public API Re-exports (simplified)
// ================================

/// Re-export function system components
pub use functions::*;

/// Re-export state definitions
pub use states::*;

// ================================
// Removed Complex Abstractions
// ================================

// The following have been removed to align with kernel's simplified approach:
// - Shard trait system (unused by kernel's hardcoded registry)
// - ShardUpgrade system (complex upgrade logic not needed)
// - Complex environment fields (kernel uses ExecutionContext)
// - Escrow module (if not used by kernel)

