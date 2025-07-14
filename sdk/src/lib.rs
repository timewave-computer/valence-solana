use anchor_client::{Client, Cluster, Program};
use solana_sdk::{
    signature::{Keypair, Signer},
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use sha2::{Sha256, Digest};
use anyhow::Result;
use std::rc::Rc;

pub struct ValenceClient {
    #[allow(dead_code)]
    client: Client<Rc<Keypair>>,
    registry_program: Program<Rc<Keypair>>,
    shard_program: Program<Rc<Keypair>>,
    payer_pubkey: Pubkey,
    registry_id: Pubkey,
    shard_id: Pubkey,
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
        })
    }
    
    pub fn payer(&self) -> Pubkey {
        self.payer_pubkey
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
        let counter = match self.shard_program.rpc().get_account(&session_counter_pda) {
            Ok(account) => {
                // Account exists, read the counter value
                // Skip discriminator (8 bytes) and owner (32 bytes) to get counter
                if account.data.len() >= 48 {
                    u64::from_le_bytes(account.data[40..48].try_into().unwrap_or([0; 8]))
                } else {
                    0
                }
            }
            Err(_) => {
                // Account doesn't exist, initialize it first
                self.shard_program
                    .request()
                    .accounts(shard::accounts::InitializeSessionCounter {
                        session_counter: session_counter_pda,
                        owner,
                        system_program: anchor_lang::system_program::ID,
                    })
                    .args(shard::instruction::InitializeSessionCounter {})
                    .send()?;
                0 // Start with counter = 0
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