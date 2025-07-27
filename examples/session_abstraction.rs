// Example: Session-based account abstraction
use anchor_lang::prelude::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    commitment_config::CommitmentConfig,
};
use valence_sdk::{
    ValenceClient, SessionBuilder, OperationBuilder, AccountType,
    Guard, SessionScope,
};
use valence_core::operations::{SessionOperation, OperationBatch};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let payer = Keypair::new();
    let client = ValenceClient::new(
        "https://api.devnet.solana.com",
        payer,
        CommitmentConfig::confirmed(),
    );
    
    // Create a session for token operations
    let usdc_mint = Pubkey::new_unique(); // Your USDC mint
    let session = SessionBuilder::new(&client)
        .scope(SessionScope::Token(usdc_mint))
        .guard(Guard::And(
            Box::new(Guard::OwnerOnly),
            Box::new(Guard::UsageLimit { max: 100 })
        ))
        .build_params()?;
    
    let session_pubkey = Pubkey::new_unique(); // Derive properly in real code
    
    // Create the session on-chain
    let create_ix = client.create_session_instruction(session_pubkey, session)?;
    let sig = client.send_transaction(&[create_ix]).await?;
    println!("Session created: {}", sig);
    
    // Now use the session for account abstraction
    let alice_token = Pubkey::new_unique(); // Alice's USDC account
    let bob_token = Pubkey::new_unique();   // Bob's USDC account
    
    // Build operations off-chain
    let operations = OperationBuilder::new()
        // Register accounts off-chain for type safety
        .register_account(
            alice_token,
            AccountType::TokenAccount { mint: usdc_mint, owner: client.payer() },
            vec![] // Would fetch actual data
        )?
        .register_account(
            bob_token,
            AccountType::TokenAccount { mint: usdc_mint, owner: Pubkey::new_unique() },
            vec![]
        )?
        // Borrow accounts for the session
        .borrow_account(alice_token, 3)? // Read + Write
        .borrow_account(bob_token, 2)?   // Write only
        // Invoke token transfer
        .invoke_program(
            spl_token::ID,
            spl_token::instruction::transfer(
                &spl_token::ID,
                &alice_token,
                &bob_token,
                &client.payer(),
                &[],
                100_000_000, // 100 USDC
            )?.data,
            &[alice_token, bob_token]
        )?
        // Auto-release accounts after
        .auto_release(true)
        .build();
    
    // Execute operations through session
    let execute_ix = client.execute_operations_instruction(
        session_pubkey,
        operations,
        operations.get_all_accounts(), // Include all registered accounts
    )?;
    
    let sig = client.send_transaction(&[execute_ix]).await?;
    println!("Operations executed: {}", sig);
    
    // Example of off-chain session management with runtime
    use valence_runtime::SessionManager;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use std::sync::Arc;
    
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    ));
    
    let session_manager = SessionManager::new(rpc_client);
    
    // Load session state
    session_manager.load_session(session_pubkey).await?;
    session_manager.sync_borrowed_accounts(session_pubkey).await?;
    
    // Queue operations off-chain
    session_manager.queue_operation(
        session_pubkey,
        SessionOperation::UpdateMetadata {
            metadata: [42u8; 64],
        }
    ).await?;
    
    // Simulate before executing
    let queued_ops = vec![
        SessionOperation::UpdateMetadata { metadata: [42u8; 64] }
    ];
    let simulation = session_manager.simulate_operations(
        session_pubkey,
        &queued_ops
    ).await?;
    
    println!("Simulation result: {:?}", simulation);
    println!("Estimated compute: {} CU", simulation.compute_units);
    
    // Build batch from queued operations
    let batch = session_manager.build_batch(session_pubkey, true).await?;
    
    // Execute the batch
    let execute_ix = client.execute_operations_instruction(
        session_pubkey,
        batch,
        vec![], // No additional accounts needed for metadata update
    )?;
    
    let sig = client.send_transaction(&[execute_ix]).await?;
    println!("Batch executed: {}", sig);
    
    Ok(())
}

// Helper extension trait for client
trait ValenceClientExt {
    fn create_session_instruction(
        &self,
        session_pubkey: Pubkey,
        params: valence_core::state::CreateSessionParams,
    ) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>>;
    
    fn execute_operations_instruction(
        &self,
        session_pubkey: Pubkey,
        batch: OperationBatch,
        additional_accounts: Vec<Pubkey>,
    ) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>>;
}

impl ValenceClientExt for ValenceClient {
    fn create_session_instruction(
        &self,
        session_pubkey: Pubkey,
        params: valence_core::state::CreateSessionParams,
    ) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>> {
        // Implementation would create proper instruction
        unimplemented!("Create session instruction")
    }
    
    fn execute_operations_instruction(
        &self,
        session_pubkey: Pubkey,
        batch: OperationBatch,
        additional_accounts: Vec<Pubkey>,
    ) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>> {
        // Implementation would create proper instruction with all accounts
        unimplemented!("Execute operations instruction")
    }
}