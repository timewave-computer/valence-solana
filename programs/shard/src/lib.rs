//! Shard - User-deployed contracts that define control flow and state
//! 
//! Shards compose functions from the registry and use verifier
//! for permission checks. All state management happens in shards.

use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111114");

pub mod state;
pub mod error;
pub mod instructions;
pub mod validation;
pub mod capabilities;
mod internal;

pub use state::*;
pub use error::*;
// Import all instruction contexts
pub use instructions::*;

#[program]
pub mod valence_shard {
    use super::*;

    /// Initialize shard configuration
    pub fn initialize(
        ctx: Context<Initialize>,
        max_operations_per_bundle: u16,
        default_respect_deregistration: bool,
    ) -> Result<()> {
        instructions::admin::initialize(ctx, max_operations_per_bundle, default_respect_deregistration)
    }

    /// Request a new account (renamed from request_session)
    pub fn request_account(
        ctx: Context<RequestAccount>,
        capabilities: Vec<String>,
        init_state_hash: [u8; 32],
    ) -> Result<()> {
        instructions::account::request_account(ctx, capabilities, init_state_hash)
    }

    /// Initialize an account (renamed from initialize_session)
    pub fn initialize_account(
        ctx: Context<InitializeAccount>,
        request_id: Pubkey,
        init_state_data: Vec<u8>,
    ) -> Result<()> {
        instructions::account::initialize_account(ctx, request_id, init_state_data)
    }

    /// Create a new session from multiple accounts (new linear session concept)
    pub fn create_session(
        ctx: Context<CreateSession>,
        accounts: Vec<Pubkey>,
        namespace: String,
        nonce: u64,
        metadata: Vec<u8>,
    ) -> Result<()> {
        instructions::session::create_session(ctx, accounts, namespace, nonce, metadata)
    }

    /// Consume a session and create new sessions (UTXO-like semantics)
    pub fn consume_session(
        ctx: Context<ConsumeSession>,
        new_sessions_data: Vec<(Vec<Pubkey>, String, u64, Vec<u8>)>,
    ) -> Result<()> {
        instructions::session::consume_session(ctx, new_sessions_data)
    }

    /// Execute a synchronous bundle (atomic in single transaction)
    pub fn execute_sync_bundle(
        ctx: Context<ExecuteSyncBundle>,
        bundle: Bundle,
    ) -> Result<()> {
        instructions::bundle::execute_sync(ctx, bundle)
    }

    /// Start an asynchronous bundle execution
    pub fn start_async_bundle(
        ctx: Context<StartAsyncBundle>,
        bundle: Bundle,
    ) -> Result<()> {
        instructions::bundle::start_async(ctx, bundle)
    }

    /// Continue async bundle execution from checkpoint
    pub fn continue_async_bundle(
        ctx: Context<ContinueAsyncBundle>,
        bundle_id: Pubkey,
    ) -> Result<()> {
        instructions::bundle::continue_async(ctx, bundle_id)
    }
    
    /// Import a function from the registry
    pub fn import_function(
        ctx: Context<ImportFunction>,
        function_hash: [u8; 32],
        respect_deregistration: Option<bool>,
    ) -> Result<()> {
        instructions::functions::import_function(ctx, function_hash, respect_deregistration)
    }
    
    /// Update function import policy
    pub fn update_import_policy(
        ctx: Context<UpdateImportPolicy>,
        function_hash: [u8; 32],
        respect_deregistration: bool,
    ) -> Result<()> {
        instructions::functions::update_import_policy(ctx, function_hash, respect_deregistration)
    }

    /// Admin: Update shard configuration
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_config: ShardConfig,
    ) -> Result<()> {
        instructions::admin::update_config(ctx, new_config)
    }

    /// Admin: Pause/unpause shard
    pub fn set_paused(
        ctx: Context<SetPaused>,
        paused: bool,
    ) -> Result<()> {
        instructions::admin::set_paused(ctx, paused)
    }
    
    // ===== Simplified Session API (V2) =====
    
    /// Create a session with direct capability specification (simplified API)
    pub fn create_session_v2(
        ctx: Context<CreateSessionV2>,
        capabilities: u64,
        initial_state: Vec<u8>,
        namespace: String,
        nonce: u64,
        metadata: Vec<u8>,
    ) -> Result<()> {
        instructions::session_v2::create_session_v2(ctx, capabilities, initial_state, namespace, nonce, metadata)
    }
    
    /// Execute operations directly on a session (simplified API)
    pub fn execute_on_session(
        ctx: Context<ExecuteOnSession>,
        function_hash: [u8; 32],
        args: Vec<u8>,
    ) -> Result<()> {
        instructions::session_v2::execute_on_session(ctx, function_hash, args)
    }
    
    /// Execute a simplified bundle on a session (simplified API)
    pub fn execute_bundle_v2(
        ctx: Context<ExecuteBundleV2>,
        bundle: SimpleBundle,
    ) -> Result<()> {
        instructions::session_v2::execute_bundle_v2(ctx, bundle)
    }
}