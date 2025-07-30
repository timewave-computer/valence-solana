// Security policy configuration for valence-kernel session authorization
//
// Guard accounts store session-specific security policies that control CPI behavior,
// operation permissions, and external program access. Each session maintains its own
// guard account that defines the security boundaries for all operations executed
// within that session context.
//
// KERNEL INTEGRATION: The batch execution engine consults guard accounts before
// performing CPI calls, evaluating whether operations are permitted under the
// session's security policy. Guard accounts provide the security foundation that
// enables safe execution of complex operation batches.
//
// SECURITY MODEL: Guard accounts implement a simple but effective security model
// with flags controlling dangerous operations like unregistered CPI calls, providing
// a balance between flexibility and security for different session requirements.
use anchor_lang::prelude::*;

/// Minimal guard account for session security policy
#[account]
pub struct GuardAccount {
    /// The session this guard configuration belongs to
    pub session: Pubkey,
    
    /// Whether to allow CPI to unregistered programs
    pub allow_unregistered_cpi: bool,
    
    /// Version for future upgrades
    pub version: u8,
}

impl GuardAccount {
    /// Calculate space needed for account
    pub const fn space() -> usize {
        8 +  // discriminator
        32 + // session
        1 +  // allow_unregistered_cpi
        1    // version
    }
    
    /// Create a new guard account
    pub fn new(session: Pubkey, allow_unregistered_cpi: bool) -> Self {
        Self {
            session,
            allow_unregistered_cpi,
            version: 1,
        }
    }
}