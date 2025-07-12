//! Monitor for session requests

use crate::config::Config;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use std::time::Duration;

/// Session request data structure (matches on-chain)
#[derive(Debug, Clone)]
pub struct SessionRequest {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub capabilities: Vec<String>,
    pub init_state_hash: [u8; 32],
    pub created_at: i64,
}

pub struct SessionMonitor {
    client: RpcClient,
    config: Config,
}

impl SessionMonitor {
    pub fn new(config: Config) -> Result<Self> {
        let client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        Ok(Self { client, config })
    }
    
    pub fn poll_interval(&self) -> Duration {
        self.config.poll_interval()
    }
    
    /// Poll for new session requests
    pub async fn poll_requests(&self) -> Result<Vec<SessionRequest>> {
        // Get all accounts owned by the shard program
        let accounts = self.client.get_program_accounts(&self.config.shard_program_id)?;
        
        let mut requests = Vec::new();
        
        for (pubkey, account) in accounts {
            // Check if this is a SessionRequest account by checking discriminator
            if let Some(request) = self.parse_session_request(&pubkey, &account)? {
                requests.push(request);
            }
        }
        
        Ok(requests)
    }
    
    /// Parse account data into SessionRequest
    fn parse_session_request(&self, _pubkey: &Pubkey, account: &Account) -> Result<Option<SessionRequest>> {
        let data = &account.data;
        
        // Check minimum size for discriminator
        if data.len() < 8 {
            return Ok(None);
        }
        
        // Check discriminator (set by Anchor)
        let discriminator = &data[0..8];
        
        // Skip if not a SessionRequest
        if !self.is_session_request_discriminator(discriminator) {
            return Ok(None);
        }
        
        // Parse the data using borsh
        let mut offset = 8; // Skip discriminator
        
        // Parse id (32 bytes)
        if data.len() < offset + 32 {
            return Ok(None);
        }
        let id_bytes = &data[offset..offset + 32];
        let id = Pubkey::new_from_array(id_bytes.try_into().unwrap());
        offset += 32;
        
        // Parse owner (32 bytes)
        if data.len() < offset + 32 {
            return Ok(None);
        }
        let owner_bytes = &data[offset..offset + 32];
        let owner = Pubkey::new_from_array(owner_bytes.try_into().unwrap());
        offset += 32;
        
        // Parse capabilities length (4 bytes)
        if data.len() < offset + 4 {
            return Ok(None);
        }
        let cap_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        
        // Parse capabilities
        let mut capabilities = Vec::new();
        for _ in 0..cap_len {
            if data.len() < offset + 4 {
                return Ok(None);
            }
            let str_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            
            if data.len() < offset + str_len {
                return Ok(None);
            }
            let capability = String::from_utf8_lossy(&data[offset..offset + str_len]).to_string();
            capabilities.push(capability);
            offset += str_len;
        }
        
        // Parse init_state_hash (32 bytes)
        if data.len() < offset + 32 {
            return Ok(None);
        }
        let init_state_hash: [u8; 32] = data[offset..offset + 32].try_into().unwrap();
        offset += 32;
        
        // Parse created_at (8 bytes)
        if data.len() < offset + 8 {
            return Ok(None);
        }
        let created_at = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        
        Ok(Some(SessionRequest {
            id,
            owner,
            capabilities,
            init_state_hash,
            created_at,
        }))
    }
    
    fn is_session_request_discriminator(&self, discriminator: &[u8]) -> bool {
        // Anchor discriminator is sha256("account:SessionRequest")[..8]
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"account:SessionRequest");
        let hash = hasher.finalize();
        let expected = &hash[..8];
        discriminator == expected
    }
}