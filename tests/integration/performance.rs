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
use std::time::Instant;

#[tokio::test]
async fn test_processor_performance_baseline() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize processor
    initialize_processor(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Measure capability processing time
    let iterations = 10;
    let mut total_time = 0u128;
    
    for i in 0..iterations {
        let start = Instant::now();
        
        let capability_id = format!("perf_test_{}", i);
        let input_data = vec![1u8; 1000]; // 1KB of data
        
        process_capability(
            &mut banks_client,
            &payer,
            &program_id,
            &capability_id,
            input_data,
            recent_blockhash,
        ).await;
        
        let elapsed = start.elapsed();
        total_time += elapsed.as_millis();
        println!("Iteration {}: {:?}", i, elapsed);
    }
    
    let avg_time = total_time / iterations as u128;
    println!("Average processing time: {}ms", avg_time);
    
    // Baseline: Should be under 100ms for local tests
    assert!(avg_time < 100, "Processing time exceeds baseline");
}

#[tokio::test]
async fn test_scheduler_queue_performance() {
    let program_id = valence_kernel::id();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize scheduler
    initialize_scheduler(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    // Measure queue operations
    let queue_size = 50;
    let start = Instant::now();
    
    // Enqueue many items
    for i in 0..queue_size {
        let shard_id = Pubkey::new_unique();
        let capabilities = vec![format!("cap_{}", i)];
        let priority = (i % 10) as u8;
        
        schedule_execution(
            &mut banks_client,
            &payer,
            &program_id,
            scheduler_state,
            shard_id,
            capabilities,
            priority,
            recent_blockhash,
        ).await;
    }
    
    let enqueue_time = start.elapsed();
    println!("Time to enqueue {} items: {:?}", queue_size, enqueue_time);
    
    // Process queue
    let process_start = Instant::now();
    
    for _ in 0..queue_size {
        process_queue(
            &mut banks_client,
            &payer,
            &program_id,
            scheduler_state,
            recent_blockhash,
        ).await;
    }
    
    let process_time = process_start.elapsed();
    println!("Time to process {} items: {:?}", queue_size, process_time);
    
    // Baseline: Should handle 50 items in under 5 seconds
    assert!(enqueue_time.as_secs() < 5, "Enqueue time exceeds baseline");
    assert!(process_time.as_secs() < 5, "Process time exceeds baseline");
}

#[tokio::test]
async fn test_diff_diff_performance() {
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
    
    // Test different data sizes
    let test_sizes = vec![100, 1000, 5000];
    
    for size in test_sizes {
        let state_a = vec![0u8; size];
        let mut state_b = state_a.clone();
        
        // Modify ~10% of the data
        for i in (0..size).step_by(10) {
            state_b[i] = 255;
        }
        
        let start = Instant::now();
        
        calculate_diff(
            &mut banks_client,
            &payer,
            &program_id,
            diff_state,
            state_a,
            state_b,
            recent_blockhash,
        ).await;
        
        let elapsed = start.elapsed();
        println!("Diff calculation for {} bytes: {:?}", size, elapsed);
        
        // Baseline: Should be under 50ms for reasonable sizes
        assert!(elapsed.as_millis() < 50, "Diff calculation exceeds baseline for size {}", size);
    }
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
        max_shards: 100,
        max_queue_size: 1000,
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

async fn process_capability(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    capability_id: &str,
    input_data: Vec<u8>,
    recent_blockhash: Hash,
) {
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
    
    banks_client.process_transaction(transaction).await.unwrap();
}

async fn schedule_execution(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    scheduler_state: Pubkey,
    shard_id: Pubkey,
    capabilities: Vec<String>,
    priority: u8,
    recent_blockhash: Hash,
) {
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

async fn process_queue(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    scheduler_state: Pubkey,
    recent_blockhash: Hash,
) {
    let accounts = vec![
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::ProcessQueue {}.data();
    
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

async fn calculate_diff(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    diff_state: Pubkey,
    state_a: Vec<u8>,
    state_b: Vec<u8>,
    recent_blockhash: Hash,
) {
    let accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::CalculateDiff {
        state_a,
        state_b,
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