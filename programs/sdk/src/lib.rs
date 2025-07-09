/// Valence Protocol Rust SDK
/// A unified client library for capability-based execution on Solana
/// 
/// This SDK provides a high-level interface for interacting with the Valence Protocol
/// programs: kernel (with embedded eval), processor, scheduler, diff, and registry.
pub mod client;
pub mod types;
pub mod error;
pub mod kernel;
pub mod shard;
pub mod utils;

// Re-export the main client and types
pub use types::*;
pub use error::*;
pub use shard::*;

// Re-export commonly used Solana types
pub use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
    signer::Signer,
    commitment_config::CommitmentConfig,
};

// Re-export anchor types
pub use anchor_lang::prelude::*;

// Re-export utility functions
pub use utils::*;

/// The main SDK client for interacting with Valence Protocol
/// 
/// This client provides methods for:
/// - Capability management
/// - Session management
/// - Library registry operations
/// - Execution context management
pub struct ValenceClient {
    pub client: anchor_client::Client<std::rc::Rc<solana_sdk::signature::Keypair>>,
    pub program_ids: ProgramIds,
    pub payer: Keypair,
    pub commitment: CommitmentConfig,
}

impl ValenceClient {
    /// Create a new ValenceClient with the specified configuration
    pub fn new(config: ValenceConfig) -> ValenceResult<Self> {
        let cluster = config.cluster.clone();
        let commitment = config.commitment.unwrap_or(CommitmentConfig::confirmed());
        
        let client = anchor_client::Client::new_with_options(
            cluster,
            std::rc::Rc::new(config.payer.insecure_clone()),
            commitment,
        );
        
        Ok(Self {
            client,
            program_ids: config.program_ids,
            payer: config.payer,
            commitment,
        })
    }
    
    /// Get the kernel program client (includes shards with embedded eval)
    pub fn kernel_program(&self) -> anchor_client::Program<std::rc::Rc<solana_sdk::signature::Keypair>> {
        self.client.program(self.program_ids.kernel).expect("Failed to get kernel program")
    }
    
    /// Get the processor singleton program client
    pub fn processor_program(&self) -> anchor_client::Program<std::rc::Rc<solana_sdk::signature::Keypair>> {
        self.client.program(self.program_ids.processor).expect("Failed to get processor program")
    }
    
    /// Get the scheduler singleton program client
    pub fn scheduler_program(&self) -> anchor_client::Program<std::rc::Rc<solana_sdk::signature::Keypair>> {
        self.client.program(self.program_ids.scheduler).expect("Failed to get scheduler program")
    }
    
    /// Get the diff singleton program client
    pub fn diff_program(&self) -> anchor_client::Program<std::rc::Rc<solana_sdk::signature::Keypair>> {
        self.client.program(self.program_ids.diff).expect("Failed to get diff program")
    }
    
    /// Get the registry program client
    pub fn registry_program(&self) -> anchor_client::Program<std::rc::Rc<solana_sdk::signature::Keypair>> {
        self.client.program(self.program_ids.registry).expect("Failed to get registry program")
    }
    
    /// Initialize the kernel program with a shard
    pub async fn initialize_kernel(
        &self,
        _authority: &Pubkey,
        _shard_id: String,
        _max_compute_units: u64,
    ) -> ValenceResult<Signature> {
        Err(ValenceError::NotImplemented("initialize_kernel not yet implemented".to_string()))
    }
    
    /// Get the shard state PDA for a given shard ID
    pub fn get_shard_state_pda(&self, shard_id: &str) -> Pubkey {
        let (pda, _) = Pubkey::find_program_address(
            &[b"shard_state", shard_id.as_bytes()],
            &self.program_ids.kernel
        );
        pda
    }
    
    /// Initialize the registry
    pub async fn initialize_registry(&self, _authority: &Pubkey) -> ValenceResult<Signature> {
        Err(ValenceError::NotImplemented("initialize_registry not yet implemented".to_string()))
    }
} 