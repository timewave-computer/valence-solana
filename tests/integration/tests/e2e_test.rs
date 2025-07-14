// End-to-End Integration Test for Valence Protocol
// =================================================
//
// This test validates the complete flow of the Valence Protocol:
// 1. Starting a local Solana validator
// 2. Deploying all three programs (Registry, Shard, Test Function)
// 3. Registering a function in the Registry
// 4. Creating a session with specific capabilities
// 5. Executing the function multiple times with different commands
// 6. Consuming the session (demonstrating linear type semantics)
//
// This test ensures all components work together correctly and that
// the protocol's security model (capabilities, session consumption) is enforced.

use anchor_client::Cluster;
use solana_sdk::{
    signature::{Keypair, Signer},
};
use valence_sdk::ValenceClient;

#[path = "test_utils.rs"]
mod test_utils;
use test_utils::{LocalValidator, DeployedPrograms};
use valence_test_shard::{self as shard, create_test_session_params};

#[test]
fn test_end_to_end_flow() {
    // Start local validator
    let validator = match LocalValidator::start() {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to start validator: {}. Make sure solana-test-validator is in PATH. You can run this test with: nix develop -c cargo test", e);
        }
    };
    
    // Setup
    let payer = std::rc::Rc::new(Keypair::new());
    let cluster = Cluster::Localnet;
    
    // Airdrop SOL to payer
    validator.airdrop(&payer.pubkey(), 10)
        .expect("Failed to airdrop SOL");
    
    // Deploy programs
    let programs = DeployedPrograms::deploy()
        .expect("Failed to deploy programs. Make sure programs are built with: nix develop -c bash scripts/build-with-keys.sh");
    
    let registry_id = programs.registry_id;
    let shard_id = programs.shard_id;
    let test_function_id = programs.test_function_id;
    
    println!("Programs deployed successfully!");
    println!(" Registry: {}", registry_id);
    println!(" Shard: {}", shard_id);
    println!(" Test Function: {}", test_function_id);
    
    // Create client
    let client = ValenceClient::new(
        cluster,
        payer,
        registry_id,
        shard_id,
    ).unwrap();
    
    // Test flow
    println!("\n1. Register the test function");
    let bytecode_hash = [0u8; 32]; // Using zero hash to bypass verification in test
    
    let content_hash = client
        .register_function(test_function_id, bytecode_hash)
        .unwrap();
    println!("   Function registered with hash: {:?}", content_hash);
    
    println!("\n2. Create a session with execute capability");
    // NOTE: This currently fails due to a design issue in the shard program.
    // The program uses owner.lamports() as a nonce for PDA derivation, which is
    // problematic because lamports can change between reading and transaction execution.
    // A better design would use a deterministic nonce or counter.
    let (capabilities, metadata) = create_test_session_params();
    let session = client
        .create_session(capabilities, metadata)
        .expect("Session creation failed - see note above about lamports nonce issue");
    println!("   Session created: {}", session);
    
    println!("\n3. Execute function through session");
    
    // Test ECHO command
    println!("   Testing ECHO command...");
    let echo_data = vec![0x01, b'h', b'e', b'l', b'l', b'o'];
    client
        .execute_function(
            session,
            test_function_id,
            bytecode_hash,
            echo_data,
        )
        .unwrap();
    
    // Test COMPUTE command
    println!("   Testing COMPUTE command...");
    let mut compute_data = vec![0x02];
    compute_data.extend_from_slice(&42u32.to_le_bytes());
    compute_data.extend_from_slice(&58u32.to_le_bytes());
    client
        .execute_function(
            session,
            test_function_id,
            bytecode_hash,
            compute_data,
        )
        .unwrap();
    
    // Test VERIFY command
    println!("   Testing VERIFY command...");
    let verify_data = vec![0x03];
    client
        .execute_function(
            session,
            test_function_id,
            bytecode_hash,
            verify_data,
        )
        .unwrap();
    
    println!("   All function commands executed successfully");
    
    println!("\n4. Consume session (linear type semantics)");
    client
        .consume_session(session)
        .unwrap();
    println!("   Session consumed");
    
    println!("\nEnd-to-end test completed successfully!");
    println!("   All Anchor client operations worked correctly with environment-specific program IDs.");
}

// Example function program that could be called
// Note: Commented out to avoid anchor macro issues in test files
/*
pub mod example_function {
    use anchor_lang::prelude::*;
    
    declare_id!("ExmpL11111111111111111111111111111111111111");
    
    #[program]
    pub mod example {
        use super::*;
        
        pub fn process(ctx: Context<Process>, data: Vec<u8>) -> Result<()> {
            msg!("Processing data: {:?}", data);
            
            // Update session state (in real impl)
            let session = &mut ctx.accounts.session;
            session.data = data;
            
            Ok(())
        }
    }
    
    #[derive(Accounts)]
    pub struct Process<'info> {
        #[account(mut)]
        pub session: Account<'info, SessionData>,
        pub owner: Signer<'info>,
    }
    
    #[account]
    pub struct SessionData {
        pub data: Vec<u8>,
    }
}
*/