use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use template_shard::*;
use borsh::BorshSerialize;

#[tokio::test]
async fn test_shard_initialization() {
    // Create program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "template_shard",
        program_id,
        None, // Use default processor
    );

    // Start test
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive shard PDA
    let (shard_pda, _bump) = Pubkey::find_program_address(&[b"shard"], &program_id);

    // Create initialize instruction
    // In a real test, you would use the actual instruction builder
    println!("Shard PDA: {}", shard_pda);
    println!("Authority: {}", payer.pubkey());
    
    // Test would continue with actual transaction...
}

#[tokio::test]
async fn test_function_registration_and_execution() {
    // Test registering a function with the shard
    // Then test executing it
    
    // This would:
    // 1. Initialize shard
    // 2. Deploy hello_world function
    // 3. Register hello_world with shard
    // 4. Execute hello_world through shard
    // 5. Verify results
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    
    /// Helper to create test keypairs
    pub fn create_test_keypair() -> Keypair {
        Keypair::new()
    }
    
    /// Helper to create function hash
    pub fn create_function_hash(name: &str) -> [u8; 32] {
        let mut hash = [0u8; 32];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(32);
        hash[..len].copy_from_slice(&name_bytes[..len]);
        hash
    }
}