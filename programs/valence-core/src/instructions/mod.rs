// ================================
// Instruction Modules
// ================================

// Core instruction implementations
pub mod session;                          // Session lifecycle management
pub mod account;                          // Account creation and usage
pub mod atomic;                           // Atomic multi-account operations
pub mod shard;                            // Shard deployment and execution

// ================================
// Public Exports
// ================================

// Re-export all instruction handlers and contexts for external use
pub use session::*;
pub use account::*;
pub use atomic::*;
pub use shard::*;

// ================================
// CPI Interface Definitions
// ================================

use anchor_lang::prelude::*;

/// CPI context for basic account verification
/// Used by verifier programs to validate account usage
#[derive(Accounts)]
pub struct VerifyAccount<'info> {
    // Account being verified
    pub account: AccountInfo<'info>,
    // Caller requesting verification
    pub caller: AccountInfo<'info>,
}

/// CPI context for account verification with session state
/// Allows verifiers to access and update shared session data
#[derive(Accounts)]
pub struct VerifyAccountWithSession<'info> {
    // Account being verified
    pub account: AccountInfo<'info>,
    // Caller requesting verification
    pub caller: AccountInfo<'info>,
    // Mutable session for shared verification data
    /// CHECK: Session validation handled by verifier
    #[account(mut)]
    pub session: AccountInfo<'info>,
}

// ===== CPI Function Stubs =====

/// Verify account usage authorization
/// Implemented by external verifier programs
pub fn verify_account<'info>(_ctx: CpiContext<'_, '_, '_, 'info, VerifyAccount<'info>>) -> Result<()> {
    // This stub is overridden by actual verifier program implementations
    Ok(())
}

/// Verify account with access to session state
/// Implemented by external verifier programs that need shared context
pub fn verify_account_with_session<'info>(_ctx: CpiContext<'_, '_, '_, 'info, VerifyAccountWithSession<'info>>) -> Result<()> {
    // This stub is overridden by actual verifier program implementations
    Ok(())
}