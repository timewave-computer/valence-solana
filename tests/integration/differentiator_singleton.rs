use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    hash::Hash,
};

#[tokio::test]
async fn test_diff_singleton_initialization() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Derive diff state PDA
    let (diff_state, _bump) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );

    // Create initialization instruction
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeDiff {}.data();
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };

    // Send transaction
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Diff initialization failed");
}

#[tokio::test]
async fn test_diff_calculate_diff() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize diff first
    initialize_diff(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Calculate diff between two states
    let state_a = vec![1, 2, 3, 4, 5];
    let state_b = vec![1, 2, 4, 5, 6];
    
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );
    
    let accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::CalculateDiff {
        state_a,
        state_b,
    }.data();
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Diff calculation failed");
}

#[tokio::test]
async fn test_diff_process_diffs() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize diff
    initialize_diff(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );
    
    // Create diff operations
    use valence_kernel::diff::instructions::DiffOperation;
    let diffs = vec![
        DiffOperation::Insert { position: 0, data: vec![10, 20] },
        DiffOperation::Delete { position: 5, length: 2 },
        DiffOperation::Update { position: 10, data: vec![30, 40, 50] },
    ];
    
    let accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ProcessDiffs { diffs }.data();
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Diff processing failed");
}

#[tokio::test]
async fn test_diff_verify_integrity() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize diff
    initialize_diff(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );
    
    // Verify diff integrity with a hash
    let diff_hash = [42u8; 32]; // Example hash
    
    let accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::VerifyDiffIntegrity { diff_hash }.data();
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Diff integrity verification failed");
}

#[tokio::test]
async fn test_diff_batch_optimization() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize diff
    initialize_diff(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );
    
    // Create multiple diff operations that can be optimized
    use valence_kernel::diff::instructions::DiffOperation;
    let diffs = vec![
        DiffOperation::Insert { position: 0, data: vec![1, 2] },
        DiffOperation::Insert { position: 2, data: vec![3, 4] },  // Adjacent insert
        DiffOperation::Insert { position: 4, data: vec![5, 6] },  // Adjacent insert
        DiffOperation::Delete { position: 10, length: 2 },
        DiffOperation::Delete { position: 10, length: 3 },        // Overlapping delete
    ];
    
    let accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ProcessDiffs { diffs }.data();
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Batch diff processing failed");
}

// Helper functions

async fn initialize_diff(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    recent_blockhash: Hash,
) {
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        program_id,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeDiff {}.data();
    
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], recent_blockhash);
    
    banks_client.process_transaction(transaction).await.unwrap();
}