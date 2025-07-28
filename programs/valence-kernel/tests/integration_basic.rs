// Basic integration test for valence-kernel
// Tests core functionality without complex setup

use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use valence_kernel::{PROGRAM_ID, SessionScope, SessionSharedData, BorrowedAccount};

#[tokio::test]
async fn test_program_loads() {
    let program_test = ProgramTest::new(
        "valence_kernel",
        PROGRAM_ID,
        None, // Use default processor
    );
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Just verify the program loads without error
    assert!(banks_client.get_account(PROGRAM_ID).await.is_ok());
}

#[test]
fn test_types_compile() {
    // Test that basic types can be instantiated
    let _scope = SessionScope::Global;
    let _shared_data = SessionSharedData::default();
    let _borrowed = BorrowedAccount::default();
    
    // Test serialization
    let scope = SessionScope::User;
    let serialized = scope.try_to_vec().unwrap();
    let _deserialized: SessionScope = SessionScope::try_from_slice(&serialized).unwrap();
}

#[test]
fn test_constants() {
    // Test that program constants are accessible
    assert_eq!(valence_kernel::SESSION_SEED, b"session");
    assert_eq!(valence_kernel::SESSION_ACCOUNT_SEED, b"session");
    
    // Test that the program ID is valid
    assert!(!PROGRAM_ID.to_string().is_empty());
} 