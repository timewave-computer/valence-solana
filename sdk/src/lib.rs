use anchor_client::{Client, Cluster, Program};
use solana_sdk::{
    signature::{Keypair, Signer},
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use sha2::{Sha256, Digest};
use anyhow::Result;
use std::rc::Rc;
use valence_domain_clients::clients::solana::SolanaClient;
use valence_domain_clients::solana::{SolanaBaseClient, SolanaSigningClient};
use valence_domain_clients::common::transaction::TransactionResponse;

pub struct ValenceClient {
    #[allow(dead_code)]
    client: Client<Rc<Keypair>>,
    registry_program: Program<Rc<Keypair>>,
    shard_program: Program<Rc<Keypair>>,
    payer_pubkey: Pubkey,
    registry_id: Pubkey,
    shard_id: Pubkey,
    #[allow(dead_code)]
    solana_client: Option<SolanaClient>,
}

impl ValenceClient {
    pub fn new(
        cluster: Cluster,
        payer: Rc<Keypair>,
        registry_id: Pubkey,
        shard_id: Pubkey,
    ) -> Result<Self> {
        let payer_pubkey = payer.pubkey();
        let client = Client::new_with_options(
            cluster.clone(),
            payer.clone(),
            CommitmentConfig::confirmed(),
        );
        
        let registry_program = client.program(registry_id)?;
        let shard_program = client.program(shard_id)?;
        
        Ok(Self {
            client,
            registry_program,
            shard_program,
            payer_pubkey,
            registry_id,
            shard_id,
            solana_client: None,
        })
    }
    
    pub fn payer(&self) -> Pubkey {
        self.payer_pubkey
    }
    
    /// Create a new ValenceClient with valence-domain-clients SolanaClient
    pub fn new_with_domain_client(
        rpc_url: &str,
        payer: Keypair,
        registry_id: Pubkey,
        shard_id: Pubkey,
    ) -> Result<Self> {
        let payer_pubkey = payer.pubkey();
        let payer_rc = Rc::new(payer.insecure_clone());
        
        // Create anchor client for program interactions
        let cluster = Cluster::Custom(rpc_url.to_string(), rpc_url.to_string());
        let client = Client::new_with_options(
            cluster,
            payer_rc.clone(),
            CommitmentConfig::confirmed(),
        );
        
        let registry_program = client.program(registry_id)?;
        let shard_program = client.program(shard_id)?;
        
        // Create valence-domain-clients SolanaClient
        let solana_client = SolanaClient::new(rpc_url, &payer.to_base58_string())?;
        
        Ok(Self {
            client,
            registry_program,
            shard_program,
            payer_pubkey,
            registry_id,
            shard_id,
            solana_client: Some(solana_client),
        })
    }
    
    // Registry operations
    pub fn register_function(
        &self,
        program_id: Pubkey,
        bytecode_hash: [u8; 32],
    ) -> Result<[u8; 32]> {
        // Compute content hash
        let content_hash = compute_content_hash(&program_id, &bytecode_hash);
        
        // Derive PDA
        let (function_entry, _) = Pubkey::find_program_address(
            &[registry::FUNCTION_SEED, &content_hash],
            &self.registry_id,
        );
        
        self.registry_program
            .request()
            .accounts(registry::accounts::RegisterFunction {
                function_entry,
                program: program_id,
                authority: self.payer(),
                system_program: anchor_lang::system_program::ID,
            })
            .args(registry::instruction::RegisterFunction {
                bytecode_hash,
            })
            .send()?;
            
        Ok(content_hash)
    }
    
    // Session operations
    pub fn create_session(
        &self,
        capabilities: u64,
        metadata: Vec<u8>,
    ) -> Result<Pubkey> {
        let owner = self.payer();
        
        // First get the session counter account
        let (session_counter_pda, _) = Pubkey::find_program_address(
            &[shard::SESSION_COUNTER_SEED, owner.as_ref()],
            &self.shard_id,
        );
        
        // Check if counter exists, if not initialize it
        let counter = match self.shard_program.account::<shard::SessionCounter>(session_counter_pda) {
            Ok(counter_account) => counter_account.counter,
            Err(_) => {
                // Account doesn't exist or is invalid, so initialize it
                self.shard_program
                    .request()
                    .accounts(shard::accounts::InitializeSessionCounter {
                        session_counter: session_counter_pda,
                        owner,
                        system_program: anchor_lang::system_program::ID,
                    })
                    .args(shard::instruction::InitializeSessionCounter {})
                    .send()?;
                0 // The counter for the new session will be 0
            }
        };
        
        // Derive session PDA using the counter
        let (session, _) = Pubkey::find_program_address(
            &[
                shard::SESSION_SEED,
                owner.as_ref(),
                &counter.to_le_bytes(),
            ],
            &self.shard_id,
        );
        
        self.shard_program
            .request()
            .accounts(shard::accounts::CreateSession {
                session,
                session_counter: session_counter_pda,
                owner,
                system_program: anchor_lang::system_program::ID,
            })
            .args(shard::instruction::CreateSession {
                capabilities,
                metadata,
            })
            .send()?;
            
        Ok(session)
    }
    
    pub fn execute_function(
        &self,
        session: Pubkey,
        function_program: Pubkey,
        function_bytecode_hash: [u8; 32],
        input_data: Vec<u8>,
    ) -> Result<()> {
        // Compute function hash
        let function_hash = compute_content_hash(
            &function_program,
            &function_bytecode_hash,
        );
        
        // Derive function entry PDA
        let (function_entry, _) = Pubkey::find_program_address(
            &[registry::FUNCTION_SEED, &function_hash],
            &self.registry_id,
        );
        
        self.shard_program
            .request()
            .accounts(shard::accounts::ExecuteFunction {
                session,
                owner: self.payer(),
                registry_program: self.registry_id,
                function_entry,
                function_program,
            })
            .args(shard::instruction::ExecuteFunction {
                function_hash,
                input_data,
            })
            .send()?;
            
        Ok(())
    }
    
    pub fn consume_session(&self, session: Pubkey) -> Result<()> {
        self.shard_program
            .request()
            .accounts(shard::accounts::ConsumeSession {
                session,
                owner: self.payer(),
            })
            .args(shard::instruction::ConsumeSession {})
            .send()?;
            
        Ok(())
    }
    
    /// Get access to the underlying SolanaClient for general Solana operations
    pub fn solana_client(&self) -> Option<&SolanaClient> {
        self.solana_client.as_ref()
    }
    
    /// Helper method to get SOL balance using valence-domain-clients
    pub async fn get_balance(&self) -> Result<f64> {
        match &self.solana_client {
            Some(client) => client.get_sol_balance_as_sol().await,
            None => Err(anyhow::anyhow!("SolanaClient not available. Use new_with_domain_client constructor")),
        }
    }
    
    /// Helper method to transfer SOL using valence-domain-clients
    pub async fn transfer_sol(&self, to: &str, amount_sol: f64) -> Result<TransactionResponse> {
        match &self.solana_client {
            Some(client) => client.transfer_sol_amount(to, amount_sol).await,
            None => Err(anyhow::anyhow!("SolanaClient not available. Use new_with_domain_client constructor")),
        }
    }
}

// Utility functions
pub fn compute_content_hash(
    program_id: &Pubkey,
    bytecode_hash: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(program_id.as_ref());
    hasher.update(bytecode_hash);
    hasher.finalize().into()
}

// Session builder for fluent API
pub struct SessionBuilder {
    pub capabilities: u64,
    pub metadata: Vec<u8>,
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self {
            capabilities: 0,
            metadata: Vec::new(),
        }
    }
    
    pub fn with_read(mut self) -> Self {
        self.capabilities |= shard::Capabilities::READ;
        self
    }
    
    pub fn with_write(mut self) -> Self {
        self.capabilities |= shard::Capabilities::WRITE;
        self
    }
    
    pub fn with_execute(mut self) -> Self {
        self.capabilities |= shard::Capabilities::EXECUTE;
        self
    }
    
    pub fn with_transfer(mut self) -> Self {
        self.capabilities |= shard::Capabilities::TRANSFER;
        self
    }
    
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn build(self, client: &ValenceClient) -> Result<Pubkey> {
        client.create_session(self.capabilities, self.metadata)
    }
}