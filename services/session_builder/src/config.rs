//! Service configuration

use solana_sdk::pubkey::Pubkey;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    /// RPC endpoint to connect to
    pub rpc_url: String,
    
    /// Keypair for the session builder service
    pub keypair_path: String,
    
    /// Shard program ID to monitor
    pub shard_program_id: Pubkey,
    
    /// How often to poll for new requests (in seconds)
    pub poll_interval_secs: u64,
    
    /// Maximum retries for failed initializations
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            keypair_path: "~/.config/solana/id.json".to_string(),
            shard_program_id: Pubkey::default(),
            poll_interval_secs: 5,
            max_retries: 3,
        }
    }
}

impl Config {
    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_secs)
    }
}