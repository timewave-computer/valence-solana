use anchor_lang::prelude::*;

// ================================
// Constants and Configuration
// ================================

// PDA seeds for deriving program addresses
pub const SESSION_SEED: &[u8] = b"session";
pub const ACCOUNT_SEED: &[u8] = b"account";
pub const SHARD_SEED: &[u8] = b"shard";
pub const CODE_SEED: &[u8] = b"code";

// Size limits for predictable resource usage and abuse prevention
pub const MAX_ACCOUNTS_PER_SESSION: usize = 16;
pub const MAX_CODE_SIZE: usize = 10240; // 10KB max shard code size

// ================================
// Core State Structures
// ================================

/// A session manages a set of accounts with move semantics and provides
/// a shared verification context. Sessions enforce linear type semantics
/// they can be used multiple times but consumed (moved) only once
#[account]
#[derive(Debug)]
pub struct Session {
    // Account discriminator bump for PDA derivation
    pub bump: u8,
    
    // Ownership and access control
    pub owner: Pubkey,                     // Current owner of the session
    pub consumed: bool,                    // Move semantics - true after ownership transfer
    
    // Account management
    pub accounts: Vec<Pubkey>,             // Collection of managed SessionAccounts
    
    // Protocol identification and timing
    pub protocol_type: [u8; 32],           // Hash identifying the protocol choreography
    pub created_at: i64,                   // Unix timestamp of creation
    
    // Verification context for shared state between verifiers
    pub verification_data: [u8; 256],      // Arbitrary data shared across verifications
    pub parent_session: Option<Pubkey>,    // Optional parent for hierarchical sessions
}

impl Session {
    // Size calculation for account allocation
    pub const SIZE: usize = 1 + 32 + 4 + (32 * MAX_ACCOUNTS_PER_SESSION) + 1 + 8 + 32 + 256 + 33;
    
    // ===== State Queries =====
    
    /// Check if this session is a child of another session
    pub fn is_child_session(&self) -> bool {
        self.parent_session.is_some()
    }
    
    /// Determine if an address can access this session
    /// Access is granted to the owner or if the session hasn't been consumed
    pub fn can_access(&self, accessor: &Pubkey) -> bool {
        self.owner == *accessor || !self.consumed
    }
    
    /// Check if the session has reached its account capacity
    pub fn is_full(&self) -> bool {
        self.accounts.len() >= MAX_ACCOUNTS_PER_SESSION
    }
    
    /// Calculate remaining account slots available
    pub fn remaining_capacity(&self) -> usize {
        MAX_ACCOUNTS_PER_SESSION.saturating_sub(self.accounts.len())
    }
}

// ===== SessionAccount - Verifier-Delegated Authorization =====

/// Account with verifier-based authorization
/// Each account delegates its authorization logic to an external verifier program
#[account]
#[derive(Debug)]
pub struct SessionAccount {
    // Account discriminator bump for PDA derivation
    pub bump: u8,
    
    // Relationship to parent session
    pub session: Pubkey,                   // Parent session that manages this account
    pub verifier: Pubkey,                  // External program that authorizes usage
    
    // Security and replay protection
    pub nonce: u64,                        // Monotonic counter preventing replay attacks
    
    // Usage lifecycle management
    pub uses: u32,                         // Current number of times account has been used
    pub max_uses: u32,                     // Maximum allowed uses before expiration
    pub expires_at: i64,                   // Unix timestamp when account becomes invalid
    pub created_at: i64,                   // Unix timestamp of account creation
    
    // Extensible metadata for verifier-specific data
    pub metadata: [u8; 64],                // Arbitrary data interpreted by verifier
}

impl SessionAccount {
    // Size calculation for account allocation
    pub const SIZE: usize = 1 + 32 + 32 + 8 + 4 + 4 + 8 + 8 + 64;
    
    // ===== Serialization Constants =====
    
    // Offsets for direct field access during deserialization
    pub const DISCRIMINATOR_SIZE: usize = 8;
    pub const EXPIRES_AT_OFFSET: usize = Self::DISCRIMINATOR_SIZE + 1 + 32 + 32 + 8 + 4 + 4;
    
    // ===== State Queries =====
    
    /// Check if account is currently usable
    /// Account is active if it hasn't exceeded usage limits and hasn't expired
    pub fn is_active(&self) -> bool {
        let clock = Clock::get().unwrap();
        self.uses < self.max_uses && clock.unix_timestamp < self.expires_at
    }
    
    /// Check if account has expired based on provided timestamp
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time >= self.expires_at
    }
    
    /// Calculate remaining uses before account reaches limit
    pub fn remaining_uses(&self) -> u32 {
        self.max_uses.saturating_sub(self.uses)
    }
    
    /// Comprehensive check if account can be used at given time
    /// Combines usage limit and expiration checks
    pub fn can_be_used(&self, current_time: i64) -> bool {
        self.uses < self.max_uses && current_time < self.expires_at
    }
}

// ================================
// Shard System State
// ================================

/// Shard represents a developer workspace with initial state and program layout
/// Shards define the portion of a cross-chain program that executes on this chain
#[account]
#[derive(Debug)]
pub struct Shard {
    // Account discriminator bump for PDA derivation
    pub bump: u8,
    
    // Integrity and ownership
    pub code_hash: [u8; 32],      // Hash of deployed code for integrity verification
    pub owner: Pubkey,            // Developer who deployed this shard
}

impl Shard {
    // Size calculation for account allocation
    pub const SIZE: usize = 1 + 32 + 32;
}

// ===== CodeStorage - Shard Executable Storage =====

/// CodeStorage holds the actual code/state definition for a shard
/// Stored separately to allow for larger code payloads
#[account]
pub struct CodeStorage {
    // Variable-length code storage (limited by MAX_CODE_SIZE)
    pub code: Vec<u8>,            // Serialized code/DSL for shard execution
}