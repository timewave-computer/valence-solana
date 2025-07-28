// Separate account for storing guard data
// Enables precise sizing and efficient rent usage
use crate::guards::SerializedGuard;
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
    pub serialized_guard: SerializedGuard,
    /// Version for future upgrades
    pub version: u8,
}

impl GuardData {
    /// Calculate exact space needed for a compiled guard
    pub fn calculate_space_for_apu_program(serialized_guard: &SerializedGuard) -> usize {
        8 + // discriminator
        32 + // session pubkey
        serialized_guard.calculate_space() + // compiled guard data
        1 // version
    }
    
    /// Create new guard data
    #[must_use]
    pub fn create(session: Pubkey, serialized_guard: SerializedGuard) -> Self {
        Self {
            session,
            serialized_guard,
            version: 1,
        }
    }
}