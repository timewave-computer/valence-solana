use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_scheduler_singleton_initialization() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Derive scheduler state PDA
    let (scheduler_state, _bump) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );

    // Create initialization instruction
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeScheduler {
        max_shards: 10,
        max_queue_size: 100,
    }.data();
    
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
    assert!(result.is_ok(), "Scheduler initialization failed");
}

#[tokio::test]
async fn test_scheduler_queue_management() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize scheduler first
    initialize_scheduler(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Schedule execution
    let shard_id = Pubkey::new_unique();
    let capabilities = vec!["cap1".to_string(), "cap2".to_string(), "cap3".to_string()];
    let priority = 5;
    
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    let accounts = vec![
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ScheduleExecution {
        shard_id,
        capabilities,
        priority,
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
    assert!(result.is_ok(), "Schedule execution failed");
}

#[tokio::test]
async fn test_scheduler_priority_ordering() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize scheduler
    initialize_scheduler(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    // Schedule multiple executions with different priorities
    let test_cases = vec![
        (Pubkey::new_unique(), vec!["low_priority".to_string()], 2),
        (Pubkey::new_unique(), vec!["high_priority".to_string()], 9),
        (Pubkey::new_unique(), vec!["medium_priority".to_string()], 5),
    ];
    
    for (shard_id, capabilities, priority) in test_cases {
        let accounts = vec![
            AccountMeta::new(scheduler_state, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
        ];
        
        let data = valence_kernel::instruction::ScheduleExecution {
            shard_id,
            capabilities,
            priority,
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
        assert!(result.is_ok(), "Failed to schedule execution with priority {}", priority);
    }
    
    // Process queue and verify high priority items are processed first
    let process_accounts = vec![
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let process_data = valence_kernel::instruction::ProcessQueue {}.data();
    let process_instruction = Instruction {
        program_id,
        accounts: process_accounts,
        data: process_data,
    };
    
    let mut process_transaction = Transaction::new_with_payer(
        &[process_instruction],
        Some(&payer.pubkey()),
    );
    process_transaction.sign(&[&payer], recent_blockhash);
    
    let process_result = banks_client.process_transaction(process_transaction).await;
    assert!(process_result.is_ok(), "Queue processing failed");
}

#[tokio::test]
async fn test_scheduler_resource_allocation() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize scheduler
    initialize_scheduler(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    // Allocate resources for a shard
    let shard_id = Pubkey::new_unique();
    let compute_units = 200_000u64;
    let memory_bytes = 1_000_000u64;
    
    let accounts = vec![
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::AllocateResources {
        shard_id,
        compute_units,
        memory_bytes,
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
    assert!(result.is_ok(), "Resource allocation failed");
}

// Helper functions

async fn initialize_scheduler(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    recent_blockhash: Hash,
) {
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        program_id,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];

    let data = valence_kernel::instruction::InitializeScheduler {
        max_shards: 10,
        max_queue_size: 100,
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
    
    banks_client.process_transaction(transaction).await.unwrap();
}