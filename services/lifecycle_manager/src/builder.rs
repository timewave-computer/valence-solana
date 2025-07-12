//! Builds and initializes accounts based on requests

use anyhow::Result;
use anchor_client::Client;
use sha2::{Sha256, Digest};
use tracing::{info, error, debug};
use crate::{config::Config, db::DbPool, types::*};
use std::sync::Arc;

pub async fn run_builder(
    config: Config,
    db_pool: DbPool,
    anchor_client: Arc<Client>,
) -> Result<()> {
    info!("Starting account builder");
    
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(config.poll_interval)
    );
    
    loop {
        interval.tick().await;
        
        // Process pending account requests
        let pending_requests = crate::db::get_pending_account_requests(&db_pool).await?;
        
        for request in pending_requests {
            if let Err(e) = process_account_request(&config, &db_pool, &anchor_client, &request).await {
                error!("Error processing account request {}: {}", request.id, e);
            }
        }
    }
}

async fn process_account_request(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
    request: &AccountRequest,
) -> Result<()> {
    info!("Processing account request: {}", request.id);
    
    // Build initial state based on capabilities
    let init_state_data = build_initial_state(&request.capabilities)?;
    
    // Verify state hash matches
    let computed_hash = compute_state_hash(&init_state_data);
    if computed_hash != request.init_state_hash {
        error!("State hash mismatch for request {}", request.id);
        return Ok(()); // Skip this request
    }
    
    // Initialize account on-chain
    let program = anchor_client.program(config.shard_program_id)?;
    
    // Build initialize_account instruction
    let account_id = solana_sdk::pubkey::Pubkey::new_unique(); // Would derive properly
    
    let tx = program
        .request()
        .accounts(shard_accounts::InitializeAccount {
            initializer: program.payer(),
            account_request: request.id,
            account: account_id,
            system_program: solana_sdk::system_program::id(),
        })
        .args(shard_instruction::InitializeAccount {
            request_id: request.id,
            init_state_data: init_state_data.clone(),
        })
        .send()?;
    
    info!("Initialized account {} from request {} in tx {}", 
        account_id, request.id, tx);
    
    // Update database
    crate::db::mark_account_initialized(db_pool, &request.id, &account_id).await?;
    
    // Create initial linear progression
    let progression = LinearProgression {
        id: format!("account:{}", account_id),
        current_state: LinearState::Account { id: account_id },
        history: vec![],
        pending_operations: vec![],
    };
    crate::db::store_linear_progression(db_pool, &progression).await?;
    
    // Emit event
    let event = LifecycleEvent::AccountInitialized {
        account_id,
        request_id: request.id,
    };
    crate::db::store_lifecycle_event(db_pool, &event).await?;
    
    Ok(())
}

fn build_initial_state(capabilities: &[String]) -> Result<Vec<u8>> {
    // Build appropriate initial state based on capabilities
    let mut state = InitialState {
        version: 1,
        capabilities: capabilities.to_vec(),
        balances: Default::default(),
        metadata: Default::default(),
    };
    
    // Add capability-specific initialization
    for cap in capabilities {
        match cap.as_str() {
            "transfer" => {
                // Initialize transfer-related state
                state.balances.insert("native".to_string(), 0);
            }
            "mint" => {
                // Initialize minting state
                state.metadata.insert("mint_authority".to_string(), vec![]);
            }
            "admin" => {
                // Initialize admin state
                state.metadata.insert("admin_keys".to_string(), vec![]);
            }
            _ => {
                debug!("Unknown capability during init: {}", cap);
            }
        }
    }
    
    // Serialize state
    Ok(bincode::serialize(&state)?)
}

fn compute_state_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InitialState {
    version: u8,
    capabilities: Vec<String>,
    balances: std::collections::HashMap<String, u64>,
    metadata: std::collections::HashMap<String, Vec<u8>>,
}

// Mock account structures for the example
mod shard_accounts {
    use anchor_lang::prelude::*;
    
    #[derive(Accounts)]
    pub struct InitializeAccount<'info> {
        pub initializer: Signer<'info>,
        pub account_request: UncheckedAccount<'info>,
        pub account: UncheckedAccount<'info>,
        pub system_program: Program<'info, System>,
    }
}

mod shard_instruction {
    use anchor_lang::prelude::*;
    
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct InitializeAccount {
        pub request_id: Pubkey,
        pub init_state_data: Vec<u8>,
    }
}