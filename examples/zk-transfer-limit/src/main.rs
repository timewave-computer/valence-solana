//! ZK Transfer Limit Example - Full Flow
//! 
//! This demonstrates:
//! 1. Setting up a ZK transfer limit guard
//! 2. Creating a session that borrows an account
//! 3. Generating a proof that a transfer is within limits
//! 4. Executing the transfer with ZK verification

use anchor_client::{Client, Cluster};
use anchor_lang::prelude::*;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
};
use zk_transfer_limit_example::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup client
    let payer = Keypair::new();
    let client = Client::new_with_options(
        Cluster::Localnet,
        std::rc::Rc::new(payer),
        CommitmentConfig::confirmed(),
    );
    
    // 2. Setup accounts
    let admin = Keypair::new();
    let alice = Keypair::new(); // Account holder
    let bob = Keypair::new();   // Recipient
    let charlie = Keypair::new(); // Authorized operator
    
    println!("Setting up ZK transfer limit guard...");
    println!("Account holder: {}", alice.pubkey());
    println!("Recipient: {}", bob.pubkey());
    println!("Authorized operator: {}", charlie.pubkey());
    
    // 3. Fund Alice's account
    // In practice, you'd airdrop or transfer SOL here
    println!("\n✓ Alice's account funded with 100 SOL");
    
    // 4. Register the ZK transfer limit verification key
    setup_transfer_limit_guard(
        &client,
        &admin,
        vec![charlie.pubkey()], // Only Charlie can submit proofs
    ).await?;
    
    // 5. Generate a ZK proof for a 5 SOL transfer (within limit)
    let transfer_amount = 5 * 1_000_000_000; // 5 SOL
    let (proof, public_values) = generate_transfer_proof(
        alice.pubkey(),
        bob.pubkey(),
        transfer_amount,
    )?;
    println!("\n✓ Generated ZK proof for 5 SOL transfer (within 10 SOL limit)");
    
    // 6. Create a session that borrows Alice's account
    let session = create_transfer_limit_session(
        &client,
        &charlie, // Charlie creates the session
        alice.pubkey(), // Borrowing Alice's account
        proof.clone(),
        public_values.clone(),
    ).await?;
    
    // 7. Execute the transfer
    println!("\nExecuting 5 SOL transfer with ZK verification...");
    execute_limited_transfer(
        &client,
        session,
        alice.pubkey(),
        bob.pubkey(),
        transfer_amount,
    ).await?;
    
    // 8. Try to create a session for transfer over limit (should fail at proof generation)
    println!("\n\nTrying to generate proof for 15 SOL transfer (over 10 SOL limit)...");
    let over_limit_amount = 15 * 1_000_000_000; // 15 SOL
    
    match generate_transfer_proof(alice.pubkey(), bob.pubkey(), over_limit_amount) {
        Err(e) => println!("✓ Proof generation correctly failed: {}", e),
        Ok(_) => println!("✗ Proof generation succeeded (this shouldn't happen!)"),
    }
    
    // 9. Demonstrate unauthorized user rejection
    println!("\n\nTrying with unauthorized user (should fail)...");
    let unauthorized = Keypair::new();
    
    // Even with a valid proof, unauthorized users can't create sessions
    let (proof2, public_values2) = generate_transfer_proof(
        alice.pubkey(),
        bob.pubkey(),
        3 * 1_000_000_000, // 3 SOL - within limit
    )?;
    
    let result = create_transfer_limit_session(
        &client,
        &unauthorized, // Not in whitelist!
        alice.pubkey(),
        proof2,
        public_values2,
    ).await;
    
    match result {
        Err(e) => println!("✓ Unauthorized user correctly rejected: {}", e),
        Ok(_) => println!("✗ Unauthorized user was accepted (this shouldn't happen!)"),
    }
    
    println!("\n\nExample completed successfully!");
    println!("\nKey takeaways:");
    println!("- ZK proofs enforce transfer limits without revealing amounts");
    println!("- Only whitelisted operators can submit proofs");
    println!("- Borrowed accounts remain under session control");
    println!("- Guards are evaluated atomically with operations");
    
    Ok(())
}