//! Shard state structures (models)

use anchor_lang::prelude::*;

/// Shard configuration
#[account]
pub struct ShardConfig {
    /// Authority that controls this shard
    pub authority: Pubkey,
    /// Whether the shard is paused
    pub is_paused: bool,
    /// Maximum operations per bundle
    pub max_operations_per_bundle: u16,
    /// Default policy for respecting function deregistrations
    pub default_respect_deregistration: bool,
    /// Reserved for future use
    pub _reserved: [u8; 31],
}

/// Account request (awaiting initialization) - renamed from SessionRequest
#[account]
pub struct AccountRequest {
    /// Unique request ID
    pub id: Pubkey,
    /// Who requested the account
    pub owner: Pubkey,
    /// Requested capabilities
    pub capabilities: Vec<String>,
    /// Hash of expected initialization state
    pub init_state_hash: [u8; 32],
    /// When the request was created
    pub created_at: i64,
}

/// Account with capabilities - renamed from Session
#[account]
pub struct ValenceAccount {
    /// Account ID
    pub id: Pubkey,
    /// Account owner
    pub owner: Pubkey,
    /// Granted capabilities
    pub capabilities: Vec<String>,
    /// Current state hash
    pub state_hash: [u8; 32],
    /// Whether account is active
    pub is_active: bool,
    /// When account was created
    pub created_at: i64,
}

/// Session - Linear type containing multiple accounts in shared namespace (new concept)
#[account]
pub struct Session {
    /// Session ID
    pub id: Pubkey,
    /// Session owner
    pub owner: Pubkey,
    /// INTERNAL: Backing accounts - DO NOT ACCESS DIRECTLY
    /// This field is for internal use only. Developers should interact with Sessions
    /// through the public API without needing to know about underlying accounts.
    #[doc(hidden)]
    pub _internal_accounts: Vec<Pubkey>,
    /// Shared namespace for this session
    pub namespace: String,
    /// Whether this session has been consumed (UTXO-like semantics)
    pub is_consumed: bool,
    /// Session nonce for uniqueness
    pub nonce: u64,
    /// When session was created
    pub created_at: i64,
    /// Optional session metadata
    pub metadata: Vec<u8>,
    /// Pre-aggregated capabilities (bitmap)
    pub capabilities: u64,
    /// Pre-aggregated state root (XOR of all account state hashes)
    pub state_root: [u8; 32],
}

impl Session {
    /// Check if session has a specific capability
    pub fn has_capability(&self, cap: crate::capabilities::Capability) -> bool {
        (self.capabilities & cap.to_mask()) != 0
    }
    
    /// Check if session has all required capabilities
    pub fn has_all_capabilities(&self, caps: &[crate::capabilities::Capability]) -> bool {
        caps.iter().all(|cap| self.has_capability(*cap))
    }
    
    /// Check if session has any of the required capabilities
    pub fn has_any_capability(&self, caps: &[crate::capabilities::Capability]) -> bool {
        caps.iter().any(|cap| self.has_capability(*cap))
    }
    
    /// Get capabilities as a Capabilities struct
    pub fn get_capabilities(&self) -> crate::capabilities::Capabilities {
        crate::capabilities::Capabilities(self.capabilities)
    }
    
    /// Update state root after execution
    pub fn update_state_root(&mut self, new_state_root: [u8; 32]) {
        self.state_root = new_state_root;
    }
    
    /// Compute new state root by applying a diff
    pub fn apply_state_diff(&mut self, diff: &[u8]) -> [u8; 32] {
        // Simple XOR-based diff application for now
        let mut new_state = self.state_root;
        for (i, &byte) in diff.iter().enumerate() {
            if i < 32 {
                new_state[i] ^= byte;
            }
        }
        self.state_root = new_state;
        new_state
    }
    
    // Internal account management functions (not for public use)
    
    /// INTERNAL: Get backing accounts (only for internal use by shard program)
    #[doc(hidden)]
    pub fn get_internal_accounts(&self) -> &Vec<Pubkey> {
        &self._internal_accounts
    }
    
    /// INTERNAL: Set backing accounts (only for internal use during session creation)
    #[doc(hidden)]
    pub fn set_internal_accounts(&mut self, accounts: Vec<Pubkey>) {
        self._internal_accounts = accounts;
    }
    
    /// Get the number of backing accounts (safe for public use)
    pub fn account_count(&self) -> usize {
        self._internal_accounts.len()
    }
}

/// Session consumption record - tracks UTXO-like consumption
#[account]
pub struct SessionConsumption {
    /// The session that was consumed
    pub consumed_session: Pubkey,
    /// The new sessions created from consumption
    pub created_sessions: Vec<Pubkey>,
    /// Transaction signature that performed the consumption
    pub transaction_signature: [u8; 64],
    /// When the consumption occurred
    pub consumed_at: i64,
}

/// Bundle definition
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Bundle {
    /// Bundle operations
    pub operations: Vec<Operation>,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Session this bundle operates within
    pub session: Pubkey,
}

/// Single operation in a bundle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Operation {
    /// Function hash (lookup in registry)
    pub function_hash: [u8; 32],
    /// Arguments for the function
    pub args: Vec<u8>,
    /// Expected diff hash after execution
    pub expected_diff: Option<[u8; 32]>,
    /// Which account in the session this operation targets
    pub target_account: Option<Pubkey>,
}

/// Bundle execution mode
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExecutionMode {
    /// Execute all operations in single transaction
    Sync,
    /// Execute across multiple transactions with checkpoints
    Async,
}

/// Async bundle execution state
#[account]
pub struct ExecutionState {
    /// Bundle being executed
    pub bundle_id: Pubkey,
    /// Current operation index
    pub current_operation: u16,
    /// Total operations
    pub total_operations: u16,
    /// Current state hash
    pub state_hash: [u8; 32],
    /// Whether execution is complete
    pub is_complete: bool,
    /// The bundle operations (stored for continuation)
    pub operations: Vec<Operation>,
    /// Session this bundle belongs to
    pub session: Pubkey,
}

/// Bundle queue for pending bundles
#[account]
pub struct BundleQueue {
    /// Shard this queue belongs to
    pub shard: Pubkey,
    /// Pending bundles
    pub bundles: Vec<QueuedBundle>,
}

/// Bundle waiting in queue
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueuedBundle {
    /// Bundle ID
    pub id: Pubkey,
    /// Bundle data
    pub bundle: Bundle,
    /// Who submitted this bundle
    pub submitter: Pubkey,
    /// When it was queued
    pub queued_at: i64,
}

/// Function import record - tracks imported functions
#[account]
pub struct FunctionImport {
    /// Shard that imported this function
    pub shard: Pubkey,
    /// Function hash
    pub function_hash: [u8; 32],
    /// Program ID at time of import (cached)
    pub program: Pubkey,
    /// Whether this import respects deregistration
    pub respect_deregistration: bool,
    /// When this function was imported
    pub imported_at: i64,
}

// ===== Simplified API Types =====

/// Simplified operation for clean bundle API
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SimpleOperation {
    /// Function to execute
    pub function_hash: [u8; 32],
    /// Required capabilities (bitmap)
    pub required_capabilities: u64,
    /// Arguments for the function
    pub args: Vec<u8>,
}

/// Simplified bundle for clean API
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SimpleBundle {
    /// Operations to execute
    pub operations: Vec<SimpleOperation>,
    /// Session to execute on
    pub session: Pubkey,
}