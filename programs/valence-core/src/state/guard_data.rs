// Separate account for storing guard data
// Enables precise sizing and efficient rent usage
use crate::guards::CompiledGuard;
use anchor_lang::prelude::*;

// ================================
// Guard Data Account
// ================================

/// Stores guard configuration separately from session
/// Allows for dynamic sizing and efficient rent management
#[account]
#[derive(Debug)]
pub struct GuardData {
    /// The session this guard data belongs to
    pub session: Pubkey,
    /// The compiled guard program (flattened opcodes)
    pub compiled_guard: CompiledGuard,
    /// Version for future upgrades
    pub version: u8,
}

impl GuardData {
    /// Calculate exact space needed for a compiled guard
    pub fn space_for_compiled_guard(compiled_guard: &CompiledGuard) -> usize {
        8 + // discriminator
        32 + // session pubkey
        compiled_guard.space() + // compiled guard data
        1 // version
    }
    
    /// Create new guard data
    pub fn new(session: Pubkey, compiled_guard: CompiledGuard) -> Self {
        Self {
            session,
            compiled_guard,
            version: 1,
        }
    }
}