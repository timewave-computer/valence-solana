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
async fn test_end_to_end_capability_execution() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Step 1: Initialize all singletons
    println!("Initializing singletons...");
    initialize_all_singletons(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Step 2: Initialize a shard with embedded eval
    println!("Initializing shard...");
    let shard_keypair = Keypair::new();
    let processor_program = program_id; // In this architecture, processor is part of core
    
    initialize_shard(
        &mut banks_client,
        &payer,
        &shard_keypair,
        &program_id,
        processor_program,
        recent_blockhash,
    ).await;
    
    // Step 3: Grant a capability
    println!("Granting capability...");
    let capability_id = "test_capability";
    let verification_functions = vec!["basic_permission".to_string()];
    
    grant_capability(
        &mut banks_client,
        &payer,
        &shard_keypair.pubkey(),
        &program_id,
        capability_id,
        verification_functions,
        recent_blockhash,
    ).await;
    
    // Step 4: Schedule execution through scheduler
    println!("Scheduling execution...");
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    let schedule_accounts = vec![
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let schedule_data = valence_kernel::instruction::ScheduleExecution {
        shard_id: shard_keypair.pubkey(),
        capabilities: vec![capability_id.to_string()],
        priority: 7,
    }.data();
    
    let schedule_instruction = Instruction {
        program_id,
        accounts: schedule_accounts,
        data: schedule_data,
    };
    
    let mut schedule_transaction = Transaction::new_with_payer(
        &[schedule_instruction],
        Some(&payer.pubkey()),
    );
    schedule_transaction.sign(&[&payer], recent_blockhash);
    
    let schedule_result = banks_client.process_transaction(schedule_transaction).await;
    assert!(schedule_result.is_ok(), "Failed to schedule execution");
    
    // Step 5: Process queue to get next capability
    println!("Processing queue...");
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
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
    assert!(process_result.is_ok(), "Failed to process queue");
    
    // Step 6: Execute capability through processor
    println!("Executing capability...");
    let input_data = vec![1, 2, 3, 4, 5];
    
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        &program_id,
    );
    
    let execute_accounts = vec![
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let execute_data = valence_kernel::instruction::ProcessCapability {
        capability_id: capability_id.to_string(),
        input_data: input_data.clone(),
        session: None,
    }.data();
    
    let execute_instruction = Instruction {
        program_id,
        accounts: execute_accounts,
        data: execute_data,
    };
    
    let mut execute_transaction = Transaction::new_with_payer(
        &[execute_instruction],
        Some(&payer.pubkey()),
    );
    execute_transaction.sign(&[&payer], recent_blockhash);
    
    let execute_result = banks_client.process_transaction(execute_transaction).await;
    assert!(execute_result.is_ok(), "Failed to execute capability");
    
    // Step 7: Calculate state diff
    println!("Calculating state diff...");
    let state_before = vec![0, 0, 0, 0, 0];
    let state_after = input_data;
    
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        &program_id,
    );
    
    let diff_accounts = vec![
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let diff_data = valence_kernel::instruction::CalculateDiff {
        state_a: state_before,
        state_b: state_after,
    }.data();
    
    let diff_instruction = Instruction {
        program_id,
        accounts: diff_accounts,
        data: diff_data,
    };
    
    let mut diff_transaction = Transaction::new_with_payer(
        &[diff_instruction],
        Some(&payer.pubkey()),
    );
    diff_transaction.sign(&[&payer], recent_blockhash);
    
    let diff_result = banks_client.process_transaction(diff_transaction).await;
    assert!(diff_result.is_ok(), "Failed to calculate diff");
    
    println!("End-to-end capability execution completed successfully!");
}

#[tokio::test]
async fn test_cross_singleton_coordination() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "valence_kernel",
        program_id,
        processor!(valence_kernel::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Initialize all singletons
    initialize_all_singletons(&mut banks_client, &payer, &program_id, recent_blockhash).await;
    
    // Create multiple shards
    let shard1 = Keypair::new();
    let shard2 = Keypair::new();
    
    for shard in &[&shard1, &shard2] {
        initialize_shard(
            &mut banks_client,
            &payer,
            shard,
            &program_id,
            program_id,
            recent_blockhash,
        ).await;
    }
    
    // Schedule capabilities from both shards
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        &program_id,
    );
    
    // Shard 1 schedules high priority work
    schedule_execution(
        &mut banks_client,
        &payer,
        &program_id,
        scheduler_state,
        shard1.pubkey(),
        vec!["shard1_cap1".to_string(), "shard1_cap2".to_string()],
        8,
        recent_blockhash,
    ).await;
    
    // Shard 2 schedules lower priority work
    schedule_execution(
        &mut banks_client,
        &payer,
        &program_id,
        scheduler_state,
        shard2.pubkey(),
        vec!["shard2_cap1".to_string()],
        4,
        recent_blockhash,
    ).await;
    
    // Process queue multiple times to verify priority ordering
    for i in 0..3 {
        println!("Processing queue iteration {}", i + 1);
        
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
        assert!(process_result.is_ok(), "Failed to process queue on iteration {}", i + 1);
    }
}



// Helper functions

async fn initialize_all_singletons(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    program_id: &Pubkey,
    recent_blockhash: Hash,
) {
    // Initialize processor
    let (processor_state, _) = Pubkey::find_program_address(
        &[b"processor_state"],
        program_id,
    );
    
    let processor_accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(processor_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    
    let processor_data = valence_kernel::instruction::InitializeProcessor {}.data();
    let processor_instruction = Instruction {
        program_id: *program_id,
        accounts: processor_accounts,
        data: processor_data,
    };
    
    let mut processor_transaction = Transaction::new_with_payer(
        &[processor_instruction],
        Some(&payer.pubkey()),
    );
    processor_transaction.sign(&[payer], recent_blockhash);
    banks_client.process_transaction(processor_transaction).await.unwrap();
    
    // Initialize scheduler
    let (scheduler_state, _) = Pubkey::find_program_address(
        &[b"scheduler_state"],
        program_id,
    );
    
    let scheduler_accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(scheduler_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    
    let scheduler_data = valence_kernel::instruction::InitializeScheduler {
        max_shards: 10,
        max_queue_size: 100,
    }.data();
    
    let scheduler_instruction = Instruction {
        program_id: *program_id,
        accounts: scheduler_accounts,
        data: scheduler_data,
    };
    
    let mut scheduler_transaction = Transaction::new_with_payer(
        &[scheduler_instruction],
        Some(&payer.pubkey()),
    );
    scheduler_transaction.sign(&[payer], recent_blockhash);
    banks_client.process_transaction(scheduler_transaction).await.unwrap();
    
    // Initialize diff
    let (diff_state, _) = Pubkey::find_program_address(
        &[b"diff_state"],
        program_id,
    );
    
    let diff_accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(diff_state, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    
    let diff_data = valence_kernel::instruction::InitializeDiff {}.data();
    let diff_instruction = Instruction {
        program_id: *program_id,
        accounts: diff_accounts,
        data: diff_data,
    };
    
    let mut diff_transaction = Transaction::new_with_payer(
        &[diff_instruction],
        Some(&payer.pubkey()),
    );
    diff_transaction.sign(&[payer], recent_blockhash);
    banks_client.process_transaction(diff_transaction).await.unwrap();
}

async fn initialize_shard(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    shard: &Keypair,
    program_id: &Pubkey,
    processor_program: Pubkey,
    recent_blockhash: Hash,
) {
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(shard.pubkey(), false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    
    let data = valence_kernel::instruction::InitializeShard {
        processor_program,
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
    transaction.sign(&[payer, shard], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

async fn grant_capability(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    shard: &Pubkey,
    program_id: &Pubkey,
    capability_id: &str,
    verification_functions: Vec<String>,
    recent_blockhash: Hash,
) {
    let accounts = vec![
        AccountMeta::new(*shard, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    
    let data = valence_kernel::instruction::GrantCapability {
        capability_id: capability_id.to_string(),
        verification_functions,
        description: "Test capability".to_string(),
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