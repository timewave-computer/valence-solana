// Integration tests for Valence Protocol singleton architecture

pub mod processor_singleton;
pub mod scheduler_singleton;
pub mod diff_singleton;
pub mod end_to_end;
pub mod performance;

// Re-export common test utilities
pub use solana_program_test::*;
pub use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

/// Common test setup function
pub async fn setup_test_environment() -> (BanksClient, Keypair, Hash, Pubkey) {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );
    
    let (banks_client, payer, recent_blockhash) = program_test.start().await;
    (banks_client, payer, recent_blockhash, program_id)
}

/// Helper to create a transaction with single instruction
pub fn create_transaction(
    instruction: Instruction,
    payer: &Pubkey,
    signers: &[&Keypair],
    recent_blockhash: Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(payer),
    );
    transaction.sign(signers, recent_blockhash);
    transaction
}