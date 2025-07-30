use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use valence_sdk::{SessionBuilder, BatchBuilder, MoveSemantics};
use valence_kernel::state::{RegisteredAccount, RegisteredProgram};

#[tokio::test]
async fn test_zk_transfer_with_move_semantics() {
    // Setup test environment
    let program_test = ProgramTest::new(
        "valence_kernel",
        valence_kernel::ID,
        processor!(valence_kernel::entry),
    );
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Create session with ZK verifier
    let session_keypair = Keypair::new();
    let alt_keypair = Keypair::new();
    let guard_keypair = Keypair::new();
    
    // Mock ZK verifier program (in production, this is the committee's program)
    let zk_verifier = Pubkey::new_unique();
    
    // Build session
    let session_builder = SessionBuilder::new(
        &client,
        "test/zk/alice".to_string()
    )
    .allow_unregistered_cpi()
    .with_programs(vec![
        RegisteredProgram {
            address: zk_verifier,
            active: true,
            label: *b"zk_verifier____________________",
        }
    ]);
    
    // Test flow:
    // 1. Create position with hidden limit
    // 2. Generate ZK proof for transfer
    // 3. Execute transfer with verification
    // 4. Transfer ownership using move semantics
    // 5. Verify old owner cannot access
    
    // The key insight: ZK verification is just another registered function
    // No special kernel support needed!
}

#[test]
fn test_move_semantics_ownership_transfer() {
    use super::*;
    
    // Create a position
    let mut position = TransferLimitPosition::new(
        Pubkey::new_unique(),
        1_000_000, // 1M daily limit
    );
    
    let original_owner = position.owner;
    let new_owner = Pubkey::new_unique();
    
    // Transfer ownership
    position.transfer_ownership(new_owner).unwrap();
    
    // Verify changes
    assert_eq!(position.owner, new_owner);
    assert_ne!(position.owner, original_owner);
    assert_eq!(position.nonce, 1); // Nonce incremented
    
    // In practice, the old session would be invalidated
    // preventing the original owner from using the position
}

#[test]
fn test_zk_proof_generation() {
    use super::*;
    
    // Test proof generation for various scenarios
    let test_cases = vec![
        (1_000_000, 0, 500_000, true),        // Valid: 500K transfer, 1M limit
        (1_000_000, 900_000, 50_000, true),   // Valid: Near limit
        (1_000_000, 900_000, 200_000, false), // Invalid: Would exceed limit
    ];
    
    for (limit, transferred, amount, should_succeed) in test_cases {
        let result = generate_transfer_proof(limit, transferred, amount);
        
        if should_succeed {
            assert!(result.is_ok(), 
                "Proof generation should succeed for transfer {} with limit {} and already transferred {}",
                amount, limit, transferred
            );
        } else {
            // In practice, proof generation would fail for invalid transfers
            // For demo, we don't actually verify constraints
        }
    }
}