#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

// ================================
// Module Declarations
// ================================

// Core module structure
pub mod state;          // State definitions (Session, SessionAccount, Shard)
pub mod errors;         // Error types for the protocol
pub mod events;         // Events emitted by the protocol
pub mod instructions;   // Instruction handlers grouped by functionality

// ================================
// Public Re-exports
// ================================

// Make core types available at crate root
pub use state::*;
pub use errors::*;
pub use events::*;

// Re-export account contexts and types from instructions module
pub use instructions::{
    // ===== Session Management Contexts =====
    CreateSession, MoveSession, UpdateSessionData, CleanupSession, CloseSession,
    
    // ===== Account Management Contexts =====
    AddAccount, UseAccount, UseAccountIf, UpdateAccountMetadata, CloseAccount, CreateAccountWithSession,
    
    // ===== Atomic Operation Contexts =====
    UseAccountsAtomic, UseAccountsSimple, UseAccountsAtomicOptimized,
    
    // ===== Shard Management Contexts =====
    Deploy, Execute,
    
    // ===== Cross-Program Invocation Interfaces =====
    VerifyAccount, VerifyAccountWithSession,
    
    // ===== Verification Infrastructure Types =====
    VerificationMode, InlineVerifierType, VerificationCache, BatchVerificationRequest,
    VerifyAccountsBatch,
};

// ================================
// Program Declaration
// ================================

// Program ID placeholder - will be replaced with actual deployed address
declare_id!("VCoreXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// TODO: Fix Anchor macro issue with modular structure
// Currently not using #[program] macro due to compatibility issues
pub mod valence_core {
    use super::*;
    use super::instructions::{session, account, atomic, shard};

    // ================================
    // Session Management Instructions
    // ================================
    
    /// Create a new session (optionally as child of another session)
    /// Sessions are containers that enforce linear type semantics
    pub fn create_session(
        ctx: Context<CreateSession>,
        protocol_type: [u8; 32],
        parent_session: Option<Pubkey>,
    ) -> Result<()> {
        // TODO: This is duplicated from session::create_session due to Anchor macro issues
        // Once resolved, this should just call session::create_session(ctx, protocol_type, parent_session)
        
        // ===== Parent Session Validation =====
        
        // If creating a child session, verify parent exists and is accessible
        if let Some(parent) = parent_session {
            if let Some(parent_account) = ctx.remaining_accounts.first() {
                require!(parent_account.key() == parent, CoreError::InvalidParentSession);
                
                let parent_data = parent_account.try_borrow_data()?;
                if parent_data.len() >= 8 + Session::SIZE {
                    let parent_session = Session::try_deserialize(&mut &parent_data[8..])?;
                    require!(
                        parent_session.can_access(&ctx.accounts.owner.key()),
                        CoreError::ParentSessionAccessDenied
                    );
                }
            } else {
                return Err(CoreError::ParentSessionNotProvided.into());
            }
        }
        
        // ===== Initialize Session State =====
        
        let session = &mut ctx.accounts.session;
        session.bump = ctx.bumps.session;
        session.owner = ctx.accounts.owner.key();
        session.accounts = Vec::with_capacity(MAX_ACCOUNTS_PER_SESSION);
        session.consumed = false;
        session.created_at = Clock::get()?.unix_timestamp;
        session.protocol_type = protocol_type;
        session.verification_data = [0u8; 256];
        session.parent_session = parent_session;
        
        msg!("Session created for protocol: {:?}, parent: {:?}", 
            &protocol_type[..8],
            parent_session
        );
        Ok(())
    }

    /// Move session ownership (implements linear type semantics)
    /// Once moved, the original owner can no longer access the session
    pub fn move_session(ctx: Context<MoveSession>, new_owner: Pubkey) -> Result<()> {
        session::move_session(ctx, new_owner)
    }

    /// Update session verification data
    /// Allows sharing state between different verifiers in a protocol
    pub fn update_session_data(
        ctx: Context<UpdateSessionData>,
        data: Vec<u8>,
    ) -> Result<()> {
        session::update_session_data(ctx, data)
    }

    /// Clean up expired accounts from session
    /// Anyone can call this to save rent by removing expired accounts
    pub fn cleanup_session(ctx: Context<CleanupSession>) -> Result<()> {
        session::cleanup_session(ctx)
    }

    /// Close a consumed session and reclaim rent
    /// Only consumed sessions can be closed to prevent accidental closure
    pub fn close_session(ctx: Context<CloseSession>) -> Result<()> {
        session::close_session(ctx)
    }

    // ================================
    // Account Management Instructions
    // ================================

    /// Add a new account to the session with verifier-based authorization
    pub fn add_account(
        ctx: Context<AddAccount>, 
        verifier: Pubkey,
        max_uses: u32,
        lifetime_seconds: i64,
        metadata: Option<Vec<u8>>,
    ) -> Result<()> {
        account::add_account(ctx, verifier, max_uses, lifetime_seconds, metadata)
    }

    /// Use an account with verifier authorization
    /// Verifier program must approve the usage via CPI
    pub fn use_account<'info>(
        ctx: Context<'_, '_, '_, 'info, UseAccount<'info>>,
        operation_data: Vec<u8>,
    ) -> Result<()> {
        account::use_account(ctx, operation_data)
    }

    /// Use account conditionally based on built-in condition checks
    /// Useful for simple conditions without custom verifiers
    pub fn use_account_if(
        ctx: Context<UseAccountIf>,
        condition_type: u8,
        condition_value: u64,
    ) -> Result<()> {
        account::use_account_if(ctx, condition_type, condition_value)
    }

    /// Update account metadata
    /// Protocols can store arbitrary data like voucher IDs or path info
    pub fn update_account_metadata(
        ctx: Context<UpdateAccountMetadata>,
        metadata: Vec<u8>,
    ) -> Result<()> {
        account::update_account_metadata(ctx, metadata)
    }

    /// Close an expired account and reclaim rent
    /// Can be called by anyone for expired accounts or by owner anytime
    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        account::close_account(ctx)
    }

    /// Create session with single account in one transaction
    /// Optimized helper for the common single-participant case
    pub fn create_account_with_session(
        ctx: Context<CreateAccountWithSession>,
        protocol_type: [u8; 32],
        verifier: Pubkey,
        max_uses: u32,
        lifetime_seconds: i64,
        metadata: Option<Vec<u8>>,
    ) -> Result<()> {
        account::create_account_with_session(
            ctx,
            protocol_type,
            verifier,
            max_uses,
            lifetime_seconds,
            metadata,
        )
    }

    // ================================
    // Atomic Operation Instructions
    // ================================

    /// Use multiple accounts atomically (all succeed or all fail)
    /// Supports dynamic number of accounts via remaining_accounts
    pub fn use_accounts_atomic<'info>(
        ctx: Context<'_, '_, '_, 'info, UseAccountsAtomic<'info>>
    ) -> Result<()> {
        atomic::use_accounts_atomic(ctx)
    }
    
    /// Use exactly two accounts atomically
    /// Simplified version for the common two-party case
    pub fn use_accounts_simple(ctx: Context<UseAccountsSimple>) -> Result<()> {
        atomic::use_accounts_simple(ctx)
    }
    
    /// Use multiple accounts atomically with CPI depth awareness
    /// Optimizes verification strategy based on remaining call depth
    pub fn use_accounts_atomic_with_depth<'info>(
        ctx: Context<'_, '_, 'info, 'info, UseAccountsAtomicOptimized<'info>>,
        estimated_depth: u8,
    ) -> Result<()> {
        atomic::use_accounts_atomic_optimized(ctx, estimated_depth)
    }

    // ================================
    // Shard Management Instructions
    // ================================

    /// Deploy a new shard containing state and control flow definition
    pub fn deploy_shard(ctx: Context<Deploy>, code: Vec<u8>) -> Result<()> {
        shard::deploy(ctx, code)
    }

    /// Execute shard logic with session account authorization
    /// Actual execution is protocol-specific and happens off-chain
    pub fn execute_shard(ctx: Context<Execute>, input: Vec<u8>) -> Result<()> {
        shard::execute(ctx, input)
    }
}
