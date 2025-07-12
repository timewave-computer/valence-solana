//! Session initialization logic

use crate::{config::Config, monitor::SessionRequest};
use anyhow::{Result, anyhow};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

pub struct SessionBuilder {
    client: RpcClient,
    config: Config,
    keypair: Keypair,
}

impl SessionBuilder {
    pub fn new(config: Config) -> Result<Self> {
        let client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        // Load keypair from file
        let keypair_path = shellexpand::tilde(&config.keypair_path).to_string();
        let keypair_bytes = std::fs::read(&keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair from {}: {}", keypair_path, e))?;
        let keypair = Keypair::try_from(keypair_bytes.as_slice())
            .map_err(|e| anyhow!("Failed to parse keypair: {}", e))?;
        
        Ok(Self { client, config, keypair })
    }
    
    /// Initialize a session based on the request
    pub async fn initialize_session(&self, request: SessionRequest) -> Result<Signature> {
        println!("Initializing session {} for owner {}", request.id, request.owner);
        
        // Build initialization data based on the request
        let init_state_data = self.build_init_state(&request)?;
        
        // Verify the hash matches
        let computed_hash = self.compute_hash(&init_state_data);
        if computed_hash != request.init_state_hash {
            return Err(anyhow!("Init state hash mismatch"));
        }
        
        // Build the initialize instruction
        let instruction = self.build_initialize_instruction(request.id, init_state_data)?;
        
        // Send transaction
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );
        
        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        
        Ok(signature)
    }
    
    /// Build initialization state data for the session
    fn build_init_state(&self, request: &SessionRequest) -> Result<Vec<u8>> {
        // Build state based on requested capabilities
        let mut state_data = Vec::new();
        
        // Write version byte
        state_data.push(1u8);
        
        // Write number of capabilities
        state_data.extend_from_slice(&(request.capabilities.len() as u32).to_le_bytes());
        
        // For each capability, generate initial state
        for capability in &request.capabilities {
            // Write capability name length and name
            state_data.extend_from_slice(&(capability.len() as u32).to_le_bytes());
            state_data.extend_from_slice(capability.as_bytes());
            
            // Write capability-specific initial state
            match capability.as_str() {
                "transfer" => {
                    // Transfer capability: initialize with zero balance
                    state_data.extend_from_slice(&[0u8; 8]); // 8 bytes for u64 balance
                }
                "mint" => {
                    // Mint capability: initialize with supply limit
                    state_data.extend_from_slice(&1_000_000u64.to_le_bytes()); // Max supply
                    state_data.extend_from_slice(&0u64.to_le_bytes()); // Current supply
                }
                "vote" => {
                    // Vote capability: initialize with voting power
                    state_data.extend_from_slice(&100u64.to_le_bytes()); // Voting power
                    state_data.push(0u8); // Has voted flag
                }
                _ => {
                    // Unknown capability: no additional state
                    state_data.push(0u8); // Empty state marker
                }
            }
        }
        
        // Write timestamp
        state_data.extend_from_slice(&request.created_at.to_le_bytes());
        
        Ok(state_data)
    }
    
    /// Compute hash of initialization data
    fn compute_hash(&self, data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
    
    /// Build the initialize_session instruction
    fn build_initialize_instruction(
        &self,
        request_id: Pubkey,
        init_state_data: Vec<u8>,
    ) -> Result<Instruction> {
        // Build instruction data using proper discriminator
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"global:initialize_session");
        let hash = hasher.finalize();
        let discriminator = &hash[..8];
        
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&request_id.to_bytes());
        data.extend_from_slice(&(init_state_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&init_state_data);
        
        // Calculate PDAs
        let (session_request_pda, _) = Pubkey::find_program_address(
            &[b"session_request", request_id.as_ref()],
            &self.config.shard_program_id,
        );
        
        let (session_pda, _) = Pubkey::find_program_address(
            &[b"session", request_id.as_ref()],
            &self.config.shard_program_id,
        );
        
        Ok(Instruction {
            program_id: self.config.shard_program_id,
            accounts: vec![
                AccountMeta::new(self.keypair.pubkey(), true), // Initializer
                AccountMeta::new(session_request_pda, false),  // Request to close
                AccountMeta::new(session_pda, false),          // Session to create
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // System program
            ],
            data,
        })
    }
}