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
use solana_program_test::BanksClientError;

#[tokio::test]
async fn test_processor_singleton_initialization() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Derive processor state PDA
    let (processor_state, _bump) = Pubkey::find_program_address(
        &[b"processor_state"],
        &program_id,
    );

    // Create initialization instruction
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeProcessor {}.data();
    
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
    assert!(result.is_ok(), "Processor initialization failed");
}

#[tokio::test]
async fn test_processor_capability_execution() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize processor first
    initialize_processor(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Process capability
    let capability_id = "test_capability".to_string();
    let input_data = vec![1, 2, 3, 4];
    
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        &program_id,
    );
    
    let accounts = vec![
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ProcessCapability {
        capability_id,
        input_data,
        session: None,
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
    assert!(result.is_ok(), "Capability processing failed");
}

#[tokio::test]
async fn test_processor_pause_resume() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize processor
    initialize_processor(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        &program_id,
    );
    
    // Pause processor
    let pause_accounts = vec![
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let pause_data = valence_kernel::instruction::PauseProcessor {}.data();
    let pause_instruction = Instruction {
        program_id,
        accounts: pause_accounts.clone(),
        data: pause_data,
    };
    
    let mut pause_transaction = Transaction::new_with_payer(
        &[pause_instruction],
        Some(&payer.pubkey()),
    );
    pause_transaction.sign(&[&payer], recent_blockhash);
    
    let pause_result = banks_client.process_transaction(pause_transaction).await;
    assert!(pause_result.is_ok(), "Processor pause failed");
    
    // Try to process capability while paused (should fail)
    let process_result = process_capability_with_state(
        &mut banks_client,
        &payer,
        &program_id,
        recent_blockhash,
        "test_capability",
        vec![1, 2, 3],
    ).await;
    assert!(process_result.is_err(), "Processing should fail when paused");
    
    // Resume processor
    let resume_data = valence_kernel::instruction::ResumeProcessor {}.data();
    let resume_instruction = Instruction {
        program_id,
        accounts: pause_accounts,
        data: resume_data,
    };
    
    let mut resume_transaction = Transaction::new_with_payer(
        &[resume_instruction],
        Some(&payer.pubkey()),
    );
    resume_transaction.sign(&[&payer], recent_blockhash);
    
    let resume_result = banks_client.process_transaction(resume_transaction).await;
    assert!(resume_result.is_ok(), "Processor resume failed");
}

// Helper functions

async fn initialize_processor(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    recent_blockhash: Hash,
) {
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        program_id,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeProcessor {}.data();
    
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

async fn process_capability_with_state(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    recent_blockhash: Hash,
    capability_id: &str,
    input_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        program_id,
    );
    
    let accounts = vec![
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ProcessCapability {
        capability_id: capability_id.to_string(),
        input_data,
        session: None,
    }.data();
    
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
    
    banks_client.process_transaction(transaction).await
}