//! RPC client interface for Valence programs

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

/// Main client for interacting with Valence programs
pub struct ValenceClient {
    rpc: RpcClient,
}

impl ValenceClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url),
        }
    }
    
    // Placeholder methods - will be implemented in later phases
    pub async fn route_to_gateway(&self, _target: Pubkey, _data: Vec<u8>) -> Result<()> {
        // Use the rpc field to avoid dead code warning
        let _url = self.rpc.url();
        todo!("Implement gateway routing")
    }
}